#[cfg(feature = "rwasm")]
use crate::journal_db_wrapper::JournalDbWrapper;
#[cfg(feature = "rwasm")]
use crate::primitives::hex;
use crate::{
    builder::{EvmBuilder, HandlerStage, SetGenericStage},
    db::{Database, DatabaseCommit, EmptyDB},
    handler::Handler,
    interpreter::{
        analysis::validate_eof,
        CallInputs,
        CreateInputs,
        EOFCreateInputs,
        EOFCreateOutcome,
        Gas,
        Host,
        InstructionResult,
        InterpreterAction,
        InterpreterResult,
        SharedMemory,
    },
    primitives::{
        specification::SpecId,
        BlockEnv,
        Bytes,
        CfgEnv,
        EVMError,
        EVMResult,
        EnvWithHandlerCfg,
        ExecutionResult,
        HandlerCfg,
        InvalidTransaction,
        ResultAndState,
        TransactTo,
        TxEnv,
        U256,
    },
    Context,
    ContextWithHandlerCfg,
    Frame,
    FrameOrResult,
    FrameResult,
};
use core::{cell::RefCell, fmt};
use fluentbase_core::{
    loader::{_loader_call, _loader_create},
};
use fluentbase_sdk::{types::{EvmCallMethodInput, EvmCreateMethodInput}, ContractInput, AccountManager};
use fluentbase_types::{Address, ExitCode};
use revm_interpreter::{CallOutcome, CreateOutcome};
use std::vec::Vec;
use fluentbase_core::helpers::evm_error_from_exit_code;

/// EVM call stack limit.
pub const CALL_STACK_LIMIT: u64 = 1024;

/// EVM instance containing both internal EVM context and external context
/// and the handler that dictates the logic of EVM (or hardfork specification).
pub struct Evm<'a, EXT, DB: Database> {
    /// Context of execution, containing both EVM and external context.
    pub context: Context<EXT, DB>,
    /// Handler is a component of the of EVM that contains all the logic. Handler contains
    /// specification id and it different depending on the specified fork.
    pub handler: Handler<'a, Context<EXT, DB>, EXT, DB>,
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
        handler: Handler<'a, Context<EXT, DB>, EXT, DB>,
    ) -> Evm<'a, EXT, DB> {
        context.evm.journaled_state.set_spec_id(handler.cfg.spec_id);
        Evm { context, handler }
    }

    /// Allow for evm setting to be modified by feeding current evm
    /// into the builder for modifications.
    pub fn modify(self) -> EvmBuilder<'a, HandlerStage, EXT, DB> {
        EvmBuilder::new(self)
    }

    /// Runs main call loop.
    #[inline]
    #[cfg(not(feature = "rwasm"))]
    pub fn run_the_loop(&mut self, first_frame: Frame) -> Result<FrameResult, EVMError<DB::Error>> {
        let mut call_stack: Vec<Frame> = Vec::with_capacity(1025);
        call_stack.push(first_frame);

        #[cfg(feature = "memory_limit")]
        let mut shared_memory =
            SharedMemory::new_with_memory_limit(self.context.evm.env.cfg.memory_limit);
        #[cfg(not(feature = "memory_limit"))]
        let mut shared_memory = SharedMemory::new();

        shared_memory.new_context();

        // Peek the last stack frame.
        let mut stack_frame = call_stack.last_mut().unwrap();

        loop {
            // Execute the frame.
            let next_action =
                self.handler
                    .execute_frame(stack_frame, &mut shared_memory, &mut self.context)?;

            // Take error and break the loop, if any.
            // This error can be set in the Interpreter when it interacts with the context.
            self.context.evm.take_error()?;

            let exec = &mut self.handler.execution;
            let frame_or_result = match next_action {
                InterpreterAction::Call { inputs } => exec.call(&mut self.context, inputs)?,
                InterpreterAction::Create { inputs } => exec.create(&mut self.context, inputs)?,
                InterpreterAction::EOFCreate { inputs } => {
                    exec.eofcreate(&mut self.context, inputs)?
                }
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
                            FrameResult::Call(exec.call_return(ctx, frame, result)?)
                        }
                        Frame::Create(frame) => {
                            // return_create
                            FrameResult::Create(exec.create_return(ctx, frame, result)?)
                        }
                        Frame::EOFCreate(frame) => {
                            // return_eofcreate
                            FrameResult::EOFCreate(exec.eofcreate_return(ctx, frame, result)?)
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
                        // Break the loop if there are no more frames.
                        return Ok(result);
                    };
                    stack_frame = top_frame;
                    let ctx = &mut self.context;
                    // Insert result to the top frame.
                    match result {
                        FrameResult::Call(outcome) => {
                            // return_call
                            exec.insert_call_outcome(ctx, stack_frame, &mut shared_memory, outcome)?
                        }
                        FrameResult::Create(outcome) => {
                            // return_create
                            exec.insert_create_outcome(ctx, stack_frame, outcome)?
                        }
                        FrameResult::EOFCreate(outcome) => {
                            // return_eofcreate
                            exec.insert_eofcreate_outcome(ctx, stack_frame, outcome)?
                        }
                    }
                }
            }
        }
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
        let output = self.preverify_transaction_inner().map(|_| ());
        self.clear();
        output
    }

    /// Calls clear handle of post execution to clear the state for next execution.
    fn clear(&mut self) {
        self.handler.post_execution().clear(&mut self.context);
    }

    /// Transact pre-verified transaction
    ///
    /// This function will not validate the transaction.
    #[inline]
    pub fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        let initial_gas_spend = self
            .handler
            .validation()
            .initial_tx_gas(&self.context.evm.env)
            .map_err(|e| {
                self.clear();
                e
            })?;
        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Pre verify transaction inner.
    #[inline]
    fn preverify_transaction_inner(&mut self) -> Result<u64, EVMError<DB::Error>> {
        self.handler.validation().env(&self.context.evm.env)?;
        let initial_gas_spend = self
            .handler
            .validation()
            .initial_tx_gas(&self.context.evm.env)?;
        self.handler
            .validation()
            .tx_against_state(&mut self.context)?;
        Ok(initial_gas_spend)
    }

    /// Transact transaction
    ///
    /// This function will validate the transaction.
    #[inline]
    pub fn transact(&mut self) -> EVMResult<DB::Error> {
        let initial_gas_spend = self.preverify_transaction_inner().map_err(|e| {
            self.clear();
            e
        })?;

        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Returns the reference of handler configuration
    #[inline]
    pub fn handler_cfg(&self) -> &HandlerCfg {
        &self.handler.cfg
    }

    /// Returns the reference of Env configuration
    #[inline]
    pub fn cfg(&self) -> &CfgEnv {
        &self.context.env().cfg
    }

    /// Returns the mutable reference of Env configuration
    #[inline]
    pub fn cfg_mut(&mut self) -> &mut CfgEnv {
        &mut self.context.evm.env.cfg
    }

    /// Returns the reference of transaction
    #[inline]
    pub fn tx(&self) -> &TxEnv {
        &self.context.evm.env.tx
    }

    /// Returns the mutable reference of transaction
    #[inline]
    pub fn tx_mut(&mut self) -> &mut TxEnv {
        &mut self.context.evm.env.tx
    }

    /// Returns the reference of database
    #[inline]
    pub fn db(&self) -> &DB {
        &self.context.evm.db
    }

    /// Returns the mutable reference of database
    #[inline]
    pub fn db_mut(&mut self) -> &mut DB {
        &mut self.context.evm.db
    }

    /// Returns the reference of block
    #[inline]
    pub fn block(&self) -> &BlockEnv {
        &self.context.evm.env.block
    }

    /// Returns the mutable reference of block
    #[inline]
    pub fn block_mut(&mut self) -> &mut BlockEnv {
        &mut self.context.evm.env.block
    }

    /// Modify spec id, this will create new EVM that matches this spec id.
    pub fn modify_spec_id(&mut self, spec_id: SpecId) {
        self.handler.modify_spec_id(spec_id);
    }

    /// Returns internal database and external struct.
    #[inline]
    pub fn into_context(self) -> Context<EXT, DB> {
        self.context
    }

    /// Returns database and [`EnvWithHandlerCfg`].
    #[inline]
    pub fn into_db_and_env_with_handler_cfg(self) -> (DB, EnvWithHandlerCfg) {
        (
            self.context.evm.inner.db,
            EnvWithHandlerCfg {
                env: self.context.evm.inner.env,
                handler_cfg: self.handler.cfg,
            },
        )
    }

    /// Returns [Context] and [HandlerCfg].
    #[inline]
    pub fn into_context_with_handler_cfg(self) -> ContextWithHandlerCfg<EXT, DB> {
        ContextWithHandlerCfg::new(self.context, self.handler.cfg)
    }

    fn input_from_env(
        &self,
        gas: &Gas,
        caller_address: Address,
        callee_address: Address,
        value: U256,
    ) -> ContractInput {
        ContractInput {
            contract_gas_limit: gas.remaining(),
            contract_address: callee_address,
            contract_caller: caller_address,
            contract_value: value,
            contract_is_static: false,
            block_chain_id: self.context.evm.env.cfg.chain_id,
            block_coinbase: self.context.evm.env.block.coinbase,
            block_timestamp: self.context.evm.env.block.timestamp.as_limbs()[0],
            block_number: self.context.evm.env.block.number.as_limbs()[0],
            block_difficulty: self.context.evm.env.block.difficulty.as_limbs()[0],
            block_prevrandao: self.context.evm.env.block.prevrandao.unwrap_or_default(),
            block_gas_limit: self.context.evm.env.block.gas_limit.as_limbs()[0],
            block_base_fee: self.context.evm.env.block.basefee,
            tx_gas_limit: self.context.evm.env.tx.gas_limit,
            tx_nonce: self.context.evm.env.tx.nonce.unwrap_or_default(),
            tx_gas_price: self.context.evm.env.tx.gas_price,
            tx_gas_priority_fee: self.context.evm.env.tx.gas_priority_fee,
            tx_caller: self.context.evm.env.tx.caller,
            tx_access_list: self.context.evm.env.tx.access_list.clone(),
            tx_blob_hashes: self.context.evm.env.tx.blob_hashes.clone(),
            tx_max_fee_per_blob_gas: self.context.evm.env.tx.max_fee_per_blob_gas,
        }
    }

    /// Transact pre-verified transaction.
    fn transact_preverified_inner(&mut self, initial_gas_spend: u64) -> EVMResult<DB::Error> {
        let spec_id = self.spec_id();
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

        let mut result = {
            #[cfg(feature = "rwasm")]
            {
                // Load EVM storage account
                // let (evm_storage, _) = ctx.evm.load_account(EVM_STORAGE_ADDRESS)?;
                // evm_storage.info.nonce = 1;
                // ctx.evm.touch(&EVM_STORAGE_ADDRESS);
                let tx_gas_limit = ctx.evm.env.tx.gas_limit;

                match ctx.evm.env.tx.transact_to {
                    TransactTo::Call(address) => {
                        let value = ctx.evm.env.tx.value;
                        let caller = ctx.evm.env.tx.caller;
                        let data = ctx.evm.env.tx.data.clone();
                        let result =
                            self.call_inner(caller, address, value, data, tx_gas_limit, gas_limit)?;
                        FrameResult::Call(result)
                    }
                    TransactTo::Create => {
                        let value = ctx.evm.env.tx.value;
                        let caller = ctx.evm.env.tx.caller;
                        let data = ctx.evm.env.tx.data.clone();
                        let result = self.create_inner(caller, value, data, gas_limit)?;
                        FrameResult::Create(result)
                    }
                }
            }
            #[cfg(not(feature = "rwasm"))]
            {
                let exec = self.handler.execution();
                // call inner handling of call/create
                let first_frame_or_result = match ctx.evm.env.tx.transact_to {
                    TransactTo::Call(_) => exec.call(
                        ctx,
                        CallInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap(),
                    )?,
                    TransactTo::Create => {
                        // if first byte of data is magic 0xEF00, then it is EOFCreate.
                        if spec_id.is_enabled_in(SpecId::PRAGUE)
                            && ctx
                                .env()
                                .tx
                                .data
                                .get(0..=1)
                                .filter(|&t| t == [0xEF, 00])
                                .is_some()
                        {
                            // TODO Should we just check 0xEF it seems excessive to switch to legacy
                            // only if it 0xEF00?

                            // get nonce from tx (if set) or from account (if not).
                            // Nonce for call is bumped in deduct_caller while
                            // for CREATE it is not (it is done inside exec handlers).
                            let nonce = ctx.evm.env.tx.nonce.unwrap_or_else(|| {
                                let caller = ctx.evm.env.tx.caller;
                                ctx.evm
                                    .load_account(caller)
                                    .map(|(a, _)| a.info.nonce)
                                    .unwrap_or_default()
                            });

                            // Create EOFCreateInput from transaction initdata.
                            let eofcreate = EOFCreateInputs::new_tx_boxed(&ctx.evm.env.tx, nonce)
                                .ok()
                                .and_then(|eofcreate| {
                                    // validate EOF initcode
                                    validate_eof(&eofcreate.eof_init_code).ok()?;
                                    Some(eofcreate)
                                });

                            if let Some(eofcreate) = eofcreate {
                                exec.eofcreate(ctx, eofcreate)?
                            } else {
                                // Return result, as code is invalid.
                                FrameOrResult::Result(FrameResult::EOFCreate(
                                    EOFCreateOutcome::new(
                                        InterpreterResult::new(
                                            InstructionResult::Stop,
                                            Bytes::new(),
                                            Gas::new(gas_limit),
                                        ),
                                        ctx.env().tx.caller.create(nonce),
                                    ),
                                ))
                            }
                        } else {
                            // Safe to unwrap because we are sure that it is create tx.
                            exec.create(
                                ctx,
                                CreateInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap(),
                            )?
                        }
                    }
                };

                // Starts the main running loop.
                let mut result = match first_frame_or_result {
                    FrameOrResult::Frame(first_frame) => self.run_the_loop(first_frame)?,
                    FrameOrResult::Result(result) => result,
                };
                result
            }
        };

        let ctx = &mut self.context;

        // handle output of call/create calls.
        self.handler
            .execution()
            .last_frame_return(ctx, &mut result)?;

        let post_exec = self.handler.post_execution();
        // Reimburse the caller
        post_exec.reimburse_caller(ctx, result.gas())?;
        // Reward beneficiary
        post_exec.reward_beneficiary(ctx, result.gas())?;
        // Returns output of transaction.
        post_exec.output(ctx, result)
    }

    /// EVM create opcode for both initial crate and CREATE and CREATE2 opcodes.
    #[cfg(feature = "rwasm")]
    fn create_inner(
        &mut self,
        caller_address: Address,
        value: U256,
        input: Bytes,
        gas_limit: u64,
    ) -> Result<CreateOutcome, EVMError<DB::Error>> {
        let return_result = |instruction_result: InstructionResult, gas: Gas| CreateOutcome {
            result: InterpreterResult {
                result: instruction_result,
                output: Default::default(),
                gas,
            },
            address: None,
        };

        let mut gas = Gas::new(gas_limit);

        if self.context.evm.journaled_state.depth as u64 > CALL_STACK_LIMIT {
            return Ok(return_result(InstructionResult::CallTooDeep, gas));
        }

        let (caller_account, _) = self.context.evm.load_account(caller_address)?;
        // .expect("external database error");
        if caller_account.info.balance < value {
            return Ok(return_result(InstructionResult::OutOfFunds, gas));
        }

        let method_data = EvmCreateMethodInput {
            bytecode: input,
            value,
            gas_limit: gas.remaining(),
            salt: None,
            depth: 0,
        };

        let contract_input = self.input_from_env(&mut gas, caller_address, Address::ZERO, value);
        let am = JournalDbWrapper::new(RefCell::new(&mut self.context.evm));
        let create_output = _loader_create(&contract_input, &am, method_data);

        // let (output_buffer, exit_code) = self.exec_rwasm_binary(
        //     &mut gas,
        //     caller_account,
        //     &mut middleware_account,
        //     None,
        //     core_input.into(),
        //     value,
        // );

        // let create_output = if exit_code == ExitCode::Ok {
        //     let mut buffer_decoder = BufferDecoder::new(output_buffer.as_ref());
        //     let mut create_output = EvmCreateMethodOutput::default();
        //     EvmCreateMethodOutput::decode_body(&mut buffer_decoder, 0, &mut create_output);
        //     create_output
        // } else {
        //     EvmCreateMethodOutput::from_exit_code(exit_code).with_gas(gas.remaining())
        // };

        // let created_address = if exit_code == ExitCode::Ok {
        //     if output_buffer.len() != 20 {
        //         return return_result(ExitCode::CreateError, gas);
        //     }
        //     assert_eq!(
        //         output_buffer.len(),
        //         20,
        //         "output buffer is not 20 bytes after create/create2"
        //     );
        //     Some(Address::from_slice(output_buffer.as_ref()))
        // } else {
        //     None
        // };

        let mut gas = Gas::new(create_output.gas);
        gas.record_refund(create_output.gas_refund);

        Ok(CreateOutcome {
            result: InterpreterResult {
                result: evm_error_from_exit_code(create_output.exit_code.into()),
                output: Bytes::new(),
                gas,
            },
            address: create_output.address,
        })
    }

    /// Main contract call of the EVM.
    #[cfg(feature = "rwasm")]
    fn call_inner(
        &mut self,
        caller_address: Address,
        callee_address: Address,
        value: U256,
        input: Bytes,
        tx_gas_limit: u64,
        gas_limit: u64,
    ) -> Result<CallOutcome, EVMError<DB::Error>> {
        let mut gas = Gas::new(tx_gas_limit);

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if value == U256::ZERO {
            self.context.evm.load_account(callee_address)?;
            self.context.evm.journaled_state.touch(&callee_address);
        }

        let method_input = EvmCallMethodInput {
            callee: callee_address,
            value,
            input,
            gas_limit,
            depth: 0,
        };
        let contract_input = self.input_from_env(&mut gas, caller_address, callee_address, value);
        let am = JournalDbWrapper::new(RefCell::new(&mut self.context.evm));
        let call_output = _loader_call(&contract_input, &am, method_input);

        #[cfg(feature = "debug_print")]
        {
            println!("executed ECL call:");
            println!(" - caller: 0x{}", hex::encode(caller_address));
            println!(" - callee: 0x{}", hex::encode(callee_address));
            println!(" - value: 0x{}", hex::encode(&value.to_be_bytes::<32>()));
            println!(
                " - call_output.gas_remaining: {}",
                call_output.gas_remaining
            );
            println!(" - call_output.gas_refund: {}", call_output.gas_refund);
            println!(
                " - fuel consumed: {}",
                gas.remaining() as i64 - call_output.gas_remaining as i64
            );
            println!(" - gas.limit: {}", gas.limit() as i64);
            println!(" - gas.remaining: {}", gas.remaining() as i64);
            println!(" - gas.spent: {}", gas.spent() as i64);
            println!(" - gas.refunded: {}", gas.refunded());
            println!(" - exit code: {}", call_output.exit_code);
            if call_output.output.iter().all(|c| c.is_ascii()) {
                println!(
                    " - output message: {}",
                    from_utf8(&call_output.output).unwrap()
                );
            } else {
                println!(
                    " - output message: {}",
                    format!("0x{}", hex::encode(&call_output.output))
                );
            }
        }

        let mut gas = Gas::new(tx_gas_limit);
        if !gas.record_cost(tx_gas_limit - call_output.gas_remaining) {
            return Err(InvalidTransaction::CallGasCostMoreThanGasLimit.into());
        };
        gas.record_refund(call_output.gas_refund);

        Ok(CallOutcome {
            result: InterpreterResult {
                result: evm_error_from_exit_code(call_output.exit_code.into()),
                output: call_output.output,
                gas,
            },
            memory_offset: Default::default(),
        })
    }
}
