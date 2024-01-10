use crate::{
    builder::{EvmBuilder, HandlerStage, SetGenericStage},
    db::{Database, DatabaseCommit, EmptyDB},
    handler::Handler,
    interpreter::{
        opcode::InstructionTables, Host, Interpreter, InterpreterAction, InterpreterResult,
        SelfDestructResult, SharedMemory,
    },
    primitives::{
        specification::SpecId, Address, Bytecode, Bytes, EVMError, EVMResult, Env, ExecutionResult,
        Log, Output, ResultAndState, TransactTo, B256, U256,
    },
    CallStackFrame, Context, FrameOrResult,
};
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

/// EVM call stack limit.
pub const CALL_STACK_LIMIT: u64 = 1024;

/// EVM instance containing both internal EVM context and external context
/// and the handler that dictates the logic of EVM (or hardfork specification).
pub struct Evm<'a, EXT, DB: Database> {
    /// Context of execution, containing both EVM and external context.
    pub context: Context<EXT, DB>,
    /// Handler of EVM that contains all the logic. Handler contains specification id
    /// and it different depending on the specified fork.
    pub handler: Handler<'a, Self, EXT, DB>,
}

impl<EXT, DB> fmt::Debug for Evm<'_, EXT, DB>
where
    EXT: fmt::Debug,
    DB: Database + fmt::Debug,
    DB::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Evm")
            .field("evm context", &self.context.evm)
            .finish_non_exhaustive()
    }
}

impl<EXT, DB: Database + DatabaseCommit> Evm<'_, EXT, DB> {
    /// Commit the changes to the database.
    pub fn transact_commit(&mut self) -> Result<ExecutionResult, EVMError<DB::Error>> {
        let ResultAndState { result, state } = self.transact()?;
        self.context.evm.db.commit(state);
        Ok(result)
    }
}

impl<'a> Evm<'a, (), EmptyDB> {
    /// Returns evm builder with empty database and empty external context.
    pub fn builder() -> EvmBuilder<'a, SetGenericStage, (), EmptyDB> {
        EvmBuilder::default()
    }
}

impl<'a, EXT, DB: Database> Evm<'a, EXT, DB> {
    /// Create new EVM.
    pub fn new(
        mut context: Context<EXT, DB>,
        handler: Handler<'a, Self, EXT, DB>,
    ) -> Evm<'a, EXT, DB> {
        context.evm.journaled_state.set_spec_id(handler.spec_id);
        Evm { context, handler }
    }

    /// Allow for evm setting to be modified by feeding current evm
    /// into the builder for modifications.
    pub fn modify(self) -> EvmBuilder<'a, HandlerStage, EXT, DB> {
        EvmBuilder::new(self)
    }
}

impl<EXT, DB: Database> Evm<'_, EXT, DB> {
    /// Returns specification (hardfork) that the EVM is instanced with.
    ///
    /// SpecId depends on the handler.
    pub fn spec_id(&self) -> SpecId {
        self.handler.spec_id
    }

    /// Pre verify transaction by checking Environment, initial gas spend and if caller
    /// has enough balance to pay for the gas.
    #[inline]
    pub fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        self.handler.validation().env(&self.context.evm.env)?;
        self.handler
            .validation()
            .initial_tx_gas(&self.context.evm.env)?;
        self.handler
            .validation()
            .tx_against_state(&mut self.context)?;
        Ok(())
    }

    /// Transact pre-verified transaction
    ///
    /// This function will not validate the transaction.
    #[inline]
    pub fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        let initial_gas_spend = self
            .handler
            .validation()
            .initial_tx_gas(&self.context.evm.env)?;
        let output = self.transact_preverified_inner(initial_gas_spend);
        self.handler.post_execution().end(&mut self.context, output)
    }

    /// Transact transaction
    ///
    /// This function will validate the transaction.
    #[inline]
    pub fn transact(&mut self) -> EVMResult<DB::Error> {
        self.handler.validation().env(&self.context.evm.env)?;
        let initial_gas_spend = self
            .handler
            .validation()
            .initial_tx_gas(&self.context.evm.env)?;
        self.handler
            .validation()
            .tx_against_state(&mut self.context)?;

        let output = self.transact_preverified_inner(initial_gas_spend);
        self.handler.post_execution().end(&mut self.context, output)
    }

    /// Modify spec id, this will create new EVM that matches this spec id.
    pub fn modify_spec_id(self, spec_id: SpecId) -> Self {
        if self.spec_id() == spec_id {
            return self;
        }
        self.modify().spec_id(spec_id).build()
    }

    /// Returns internal database and external struct.
    #[inline]
    pub fn into_context(self) -> Context<EXT, DB> {
        self.context
    }

    /// Start the main loop.
    pub fn start_the_loop(
        &mut self,
        first_stack_frame: FrameOrResult,
    ) -> (InterpreterResult, Output) {
        // Created address will be something only if it is create.
        let mut created_address = None;

        // start main loop if CallStackFrame is created correctly
        let result = match first_stack_frame {
            FrameOrResult::Frame(first_stack_frame) => {
                created_address = first_stack_frame.created_address();
                // take instruction talbe
                let table = self
                    .handler
                    .take_instruction_table()
                    .expect("Instruction table should be present");

                // run main loop
                let output = match &table {
                    InstructionTables::Plain(table) => self.run_the_loop(table, first_stack_frame),
                    InstructionTables::Boxed(table) => self.run_the_loop(table, first_stack_frame),
                };

                // return back instruction table
                self.handler.set_instruction_table(table);

                output
            }
            FrameOrResult::Result(interpreter_result) => interpreter_result,
        };

        // output of execution
        let main_output = match self.context.evm.env.tx.transact_to {
            TransactTo::Call(_) => Output::Call(result.output.clone()),
            TransactTo::Create(_) => Output::Create(result.output.clone(), created_address),
        };

        (result, main_output)
    }

    /// Runs main call loop.
    #[inline]
    pub fn run_the_loop<FN>(
        &mut self,
        instruction_table: &[FN; 256],
        first_frame: Box<CallStackFrame>,
    ) -> InterpreterResult
    where
        FN: Fn(&mut Interpreter, &mut Self),
    {
        let mut call_stack: Vec<Box<CallStackFrame>> = Vec::with_capacity(1025);
        call_stack.push(first_frame);

        #[cfg(feature = "memory_limit")]
        let mut shared_memory =
            SharedMemory::new_with_memory_limit(self.context.evm.env.cfg.memory_limit);
        #[cfg(not(feature = "memory_limit"))]
        let mut shared_memory = SharedMemory::new();

        shared_memory.new_context();

        // peek last stack frame.
        let mut stack_frame = call_stack.last_mut().unwrap();

        loop {
            // run interpreter
            let action = stack_frame
                .interpreter
                .run(shared_memory, instruction_table, self);
            // take shared memory back.
            shared_memory = stack_frame.interpreter.take_memory();

            let new_frame = match action {
                InterpreterAction::SubCall {
                    inputs,
                    return_memory_offset,
                } => self.handler.execution_loop().sub_call(
                    &mut self.context,
                    inputs,
                    stack_frame,
                    &mut shared_memory,
                    return_memory_offset,
                ),
                InterpreterAction::Create { inputs } => {
                    self.handler
                        .execution_loop()
                        .sub_create(&mut self.context, stack_frame, inputs)
                }
                InterpreterAction::Return { result } => {
                    // free memory context.
                    shared_memory.free_context();

                    let child = call_stack.pop().unwrap();
                    let parent = call_stack.last_mut();

                    if let Some(result) = self.handler.execution_loop().frame_return(
                        &mut self.context,
                        child,
                        parent,
                        &mut shared_memory,
                        result,
                    ) {
                        return result;
                    }
                    stack_frame = call_stack.last_mut().unwrap();
                    continue;
                }
            };
            if let Some(new_frame) = new_frame {
                shared_memory.new_context();
                call_stack.push(new_frame);
            }
            stack_frame = call_stack.last_mut().unwrap();
        }
    }

    /// Transact pre-verified transaction.
    fn transact_preverified_inner(&mut self, initial_gas_spend: u64) -> EVMResult<DB::Error> {
        let hndl = &mut self.handler;
        let ctx = &mut self.context;

        // load access list and beneficiary if needed.
        hndl.pre_execution().load_accounts(ctx)?;

        // load precompiles
        let precompiles = hndl.pre_execution().load_precompiles();
        ctx.evm.set_precompiles(precompiles);

        // deduce caller balance with its limit.
        hndl.pre_execution().deduct_caller(ctx)?;
        // gas limit used in calls.
        let first_frame = hndl
            .execution_loop()
            .create_first_frame(ctx, ctx.evm.env.tx.gas_limit - initial_gas_spend);

        // Starts the main running loop.
        let (result, main_output) = self.start_the_loop(first_frame);

        let hndl = &mut self.handler;
        let ctx = &mut self.context;

        // handle output of call/create calls.
        let gas = hndl
            .execution_loop()
            .first_frame_return(&ctx.evm.env, result.result, result.gas);
        // Reimburse the caller
        hndl.post_execution().reimburse_caller(ctx, &gas)?;
        // Reward beneficiary
        hndl.post_execution().reward_beneficiary(ctx, &gas)?;
        // Returns output of transaction.
        hndl.post_execution()
            .output(ctx, result.result, main_output, &gas)
    }
}

impl<EXT, DB: Database> Host for Evm<'_, EXT, DB> {
    fn env(&mut self) -> &mut Env {
        self.context.evm.env()
    }

    fn block_hash(&mut self, number: U256) -> Option<B256> {
        self.context.evm.block_hash(number)
    }

    fn load_account(&mut self, address: Address) -> Option<(bool, bool)> {
        self.context.evm.load_account(address)
    }

    fn balance(&mut self, address: Address) -> Option<(U256, bool)> {
        self.context.evm.balance(address)
    }

    fn code(&mut self, address: Address) -> Option<(Bytecode, bool)> {
        self.context.evm.code(address)
    }

    fn code_hash(&mut self, address: Address) -> Option<(B256, bool)> {
        self.context.evm.code_hash(address)
    }

    fn sload(&mut self, address: Address, index: U256) -> Option<(U256, bool)> {
        self.context.evm.sload(address, index)
    }

    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        self.context.evm.sstore(address, index, value)
    }

    fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.context.evm.tload(address, index)
    }

    fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.context.evm.tstore(address, index, value)
    }

    fn log(&mut self, address: Address, topics: Vec<B256>, data: Bytes) {
        self.context.evm.journaled_state.log(Log {
            address,
            topics,
            data,
        });
    }

    fn selfdestruct(&mut self, address: Address, target: Address) -> Option<SelfDestructResult> {
        self.context
            .evm
            .journaled_state
            .selfdestruct(address, target, &mut self.context.evm.db)
            .map_err(|e| self.context.evm.error = Some(e))
            .ok()
    }
}
