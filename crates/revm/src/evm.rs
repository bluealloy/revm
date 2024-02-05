use crate::{
    builder::{EvmBuilder, HandlerStage, SetGenericStage},
    db::{Database, DatabaseCommit, EmptyDB},
    handler::Handler,
    interpreter::{
        opcode::InstructionTables, Host, Interpreter, InterpreterAction, SelfDestructResult,
        SharedMemory,
    },
    primitives::{
        specification::SpecId, Address, Bytecode, EVMError, EVMResult, Env, EnvWithHandlerCfg,
        ExecutionResult, HandlerCfg, Log, ResultAndState, TransactTo, B256, U256,
    },
    Context, Frame, FrameOrResult, FrameResult,
};
use alloc::vec::Vec;
use core::fmt;
use revm_interpreter::{CallInputs, CreateInputs};

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
        context.evm.journaled_state.set_spec_id(handler.cfg.spec_id);
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
        self.handler.cfg.spec_id
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

    pub fn handler_cfg(&self) -> &HandlerCfg {
        &self.handler.cfg
    }

    /// Returns [`EnvWithHandlerCfg`].
    pub fn into_db_and_env_with_spec_id(self) -> (DB, EnvWithHandlerCfg) {
        (
            self.context.evm.db,
            EnvWithHandlerCfg {
                env: self.context.evm.env,
                handler_cfg: self.handler.cfg,
            },
        )
    }

    /// Starts the main loop and returns outcome of the execution.
    pub fn start_the_loop(&mut self, first_frame: Frame) -> FrameResult {
        // take instruction talbe
        let table = self
            .handler
            .take_instruction_table()
            .expect("Instruction table should be present");

        // run main loop
        let frame_result = match &table {
            InstructionTables::Plain(table) => self.run_the_loop(table, first_frame),
            InstructionTables::Boxed(table) => self.run_the_loop(table, first_frame),
        };

        // return back instruction table
        self.handler.set_instruction_table(table);

        frame_result
    }

    /// Runs main call loop.
    #[inline]
    pub fn run_the_loop<FN>(
        &mut self,
        instruction_table: &[FN; 256],
        first_frame: Frame,
    ) -> FrameResult
    where
        FN: Fn(&mut Interpreter, &mut Self),
    {
        let mut call_stack: Vec<Frame> = Vec::with_capacity(1025);
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
            let interpreter = &mut stack_frame.frame_data_mut().interpreter;
            let next_action = interpreter.run(shared_memory, instruction_table, self);
            // take shared memory back.
            shared_memory = interpreter.take_memory();

            let exec = &mut self.handler.execution;
            let frame_or_result = match next_action {
                InterpreterAction::Call {
                    inputs,
                    return_memory_offset,
                } => exec.call(&mut self.context, inputs, return_memory_offset),
                InterpreterAction::Create { inputs } => exec.create(&mut self.context, inputs),
                InterpreterAction::Return { result } => {
                    // free memory context.
                    shared_memory.free_context();

                    // pop last frame from the stack and consume it to create FrameResult.
                    let returned_frame = call_stack
                        .pop()
                        .expect("We just returned from Interpreter frame");

                    let ctx = &mut self.context;
                    FrameOrResult::Result(match returned_frame {
                        Frame::Call(frame) => {
                            // return_call
                            FrameResult::Call(exec.call_return(ctx, frame, result))
                        }
                        Frame::Create(frame) => {
                            // return_create
                            FrameResult::Create(exec.create_return(ctx, frame, result))
                        }
                    })
                }
                InterpreterAction::None => unreachable!("InterpreterAction::None is not expected"),
            };

            // handle result
            match frame_or_result {
                FrameOrResult::Frame(frame) => {
                    shared_memory.new_context();
                    call_stack.push(frame);
                    stack_frame = call_stack.last_mut().unwrap();
                }
                FrameOrResult::Result(result) => {
                    let Some(top_frame) = call_stack.last_mut() else {
                        // Break the look if there are no more frames.
                        return result;
                    };
                    stack_frame = top_frame;
                    let ctx = &mut self.context;
                    // Insert result to the top frame.
                    match result {
                        FrameResult::Call(outcome) => {
                            // return_call
                            exec.insert_call_outcome(ctx, stack_frame, &mut shared_memory, outcome)
                        }
                        FrameResult::Create(outcome) => {
                            // return_create
                            exec.insert_create_outcome(ctx, stack_frame, outcome)
                        }
                    }
                }
            }
        }
    }

    /// Transact pre-verified transaction.
    fn transact_preverified_inner(&mut self, initial_gas_spend: u64) -> EVMResult<DB::Error> {
        let ctx = &mut self.context;
        let pre_exec = self.handler.pre_execution();

        // load access list and beneficiary if needed.
        pre_exec.load_accounts(ctx)?;

        // load precompiles
        let precompiles = pre_exec.load_precompiles();
        ctx.evm.set_precompiles(precompiles);

        // deduce caller balance with its limit.
        pre_exec.deduct_caller(ctx)?;

        let gas_limit = ctx.evm.env.tx.gas_limit - initial_gas_spend;

        let exec = self.handler.execution();
        // call inner handling of call/create
        let first_frame_or_result = match ctx.evm.env.tx.transact_to {
            TransactTo::Call(_) => exec.call(
                ctx,
                CallInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap(),
                0..0,
            ),
            TransactTo::Create(_) => exec.create(
                ctx,
                CreateInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap(),
            ),
        };

        // Starts the main running loop.
        let mut result = match first_frame_or_result {
            FrameOrResult::Frame(first_frame) => self.start_the_loop(first_frame),
            FrameOrResult::Result(result) => result,
        };

        let ctx = &mut self.context;

        // handle output of call/create calls.
        self.handler.execution().last_frame_return(ctx, &mut result);

        let post_exec = self.handler.post_execution();
        // Reimburse the caller
        post_exec.reimburse_caller(ctx, result.gas())?;
        // Reward beneficiary
        post_exec.reward_beneficiary(ctx, result.gas())?;
        // Returns output of transaction.
        post_exec.output(ctx, result)
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

    fn log(&mut self, log: Log) {
        self.context.evm.journaled_state.log(log);
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
