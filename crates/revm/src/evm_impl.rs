use crate::{
    db::Database,
    handler::Handler,
    inspector_instruction,
    interpreter::{
        analysis::to_analysed,
        gas,
        gas::initial_tx_gas,
        opcode::{make_boxed_instruction_table, make_instruction_table, InstructionTables},
        return_ok, CallContext, CallInputs, CallScheme, Contract, CreateInputs, Gas, Host,
        InstructionResult, Interpreter, SelfDestructResult, SharedMemory, Transfer, MAX_CODE_SIZE,
    },
    journaled_state::{JournalCheckpoint, JournaledState},
    precompile::{self, Precompile, Precompiles},
    primitives::{
        keccak256, Address, AnalysisKind, Bytecode, Bytes, EVMError, EVMResult, Env,
        InvalidTransaction, Log, Output, Spec, SpecId::*, TransactTo, B256, U256,
    },
    EVMData, Inspector,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use auto_impl::auto_impl;
use core::{fmt, marker::PhantomData};
use revm_interpreter::InterpreterResult;

#[cfg(feature = "optimism")]
use crate::optimism;

/// EVM call stack limit.
pub const CALL_STACK_LIMIT: u64 = 1024;

pub struct EVMImpl<'a, GSPEC: Spec, DB: Database> {
    pub data: EVMData<'a, DB>,
    pub inspector: Option<&'a mut dyn Inspector<DB>>,
    pub instruction_table: InstructionTables<'a, Self>,
    pub handler: Handler<DB>,
    _phantomdata: PhantomData<GSPEC>,
}

impl<GSPEC, DB> fmt::Debug for EVMImpl<'_, GSPEC, DB>
where
    GSPEC: Spec,
    DB: Database + fmt::Debug,
    DB::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EVMImpl")
            .field("data", &self.data)
            .finish_non_exhaustive()
    }
}

/// EVM transaction interface.
#[auto_impl(&mut, Box)]
pub trait Transact<DBError> {
    /// Run checks that could make transaction fail before call/create.
    fn preverify_transaction(&mut self) -> Result<(), EVMError<DBError>>;

    /// Skip pre-verification steps and execute the transaction.
    fn transact_preverified(&mut self) -> EVMResult<DBError>;

    /// Execute transaction by running pre-verification steps and then transaction itself.
    fn transact(&mut self) -> EVMResult<DBError>;
}

#[cfg(feature = "optimism")]
impl<'a, GSPEC: Spec, DB: Database> EVMImpl<'a, GSPEC, DB> {
    /// If the transaction is not a deposit transaction, subtract the L1 data fee from the
    /// caller's balance directly after minting the requested amount of ETH.
    fn remove_l1_cost(
        is_deposit: bool,
        tx_caller: Address,
        l1_cost: U256,
        db: &mut DB,
        journal: &mut JournaledState,
    ) -> Result<(), EVMError<DB::Error>> {
        if is_deposit {
            return Ok(());
        }
        let acc = journal
            .load_account(tx_caller, db)
            .map_err(EVMError::Database)?
            .0;
        if l1_cost.gt(&acc.info.balance) {
            let u64_cost = if U256::from(u64::MAX).lt(&l1_cost) {
                u64::MAX
            } else {
                l1_cost.as_limbs()[0]
            };
            return Err(EVMError::Transaction(
                InvalidTransaction::LackOfFundForMaxFee {
                    fee: u64_cost,
                    balance: acc.info.balance,
                },
            ));
        }
        acc.info.balance = acc.info.balance.saturating_sub(l1_cost);
        Ok(())
    }

    /// If the transaction is a deposit with a `mint` value, add the mint value
    /// in wei to the caller's balance. This should be persisted to the database
    /// prior to the rest of execution.
    fn commit_mint_value(
        tx_caller: Address,
        tx_mint: Option<u128>,
        db: &mut DB,
        journal: &mut JournaledState,
    ) -> Result<(), EVMError<DB::Error>> {
        if let Some(mint) = tx_mint {
            journal
                .load_account(tx_caller, db)
                .map_err(EVMError::Database)?
                .0
                .info
                .balance += U256::from(mint);
            journal.checkpoint();
        }
        Ok(())
    }
}

impl<'a, GSPEC: Spec + 'static, DB: Database> Transact<DB::Error> for EVMImpl<'a, GSPEC, DB> {
    #[inline]
    fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        self.preverify_transaction_inner()
    }

    #[inline]
    fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        let output = self.transact_preverified_inner();
        self.handler.end(&mut self.data, output)
    }

    #[inline]
    fn transact(&mut self) -> EVMResult<DB::Error> {
        let output = self
            .preverify_transaction_inner()
            .and_then(|()| self.transact_preverified_inner());
        self.handler.end(&mut self.data, output)
    }
}

/// Call frame.
#[derive(Debug)]
pub struct CallFrame<'a> {
    /// True if it is create false if it is call.
    /// TODO make a enum for this.
    is_create: bool,
    /// Journal checkpoint
    checkpoint: JournalCheckpoint,
    /// temporary. If it is create it should have address.
    created_address: Option<Address>,
    /// Interpreter
    interpreter: Interpreter<'a>,
}

impl<'a, GSPEC: Spec + 'static, DB: Database> EVMImpl<'a, GSPEC, DB> {
    #[inline]
    pub fn main_loop<FN>(
        &mut self,
        instruction_table: &[FN; 256],
        first_frame: CallFrame<'_>,
    ) -> InterpreterResult
    where
        FN: Fn(&mut Interpreter<'_>, &mut Self),
    {
        // bool true is create, false is a call.
        let mut frames: Vec<CallFrame<'_>> = Vec::with_capacity(1026);
        frames.push(first_frame);

        #[cfg(feature = "memory_limit")]
        let mut shared_memory = SharedMemory::new_with_memory_limit(self.data.env.cfg.memory_limit);
        #[cfg(not(feature = "memory_limit"))]
        let mut shared_memory = SharedMemory::new();

        shared_memory.new_context();
        let host = self;

        let mut shared_memory_ref = &mut shared_memory;

        let mut call = frames.first_mut().unwrap();

        loop {
            // run interpreter
            let action = call
                .interpreter
                .run(shared_memory_ref, instruction_table, host);
            // take shared memory back.
            shared_memory_ref = call.interpreter.shared_memory.take().unwrap();
            println!("action: {action:?}");
            match action {
                // run sub call
                revm_interpreter::InterpreterAction::SubCall { mut inputs, .. } => {
                    // Call inspector if it is some.
                    if let Some(inspector) = host.inspector.as_mut() {
                        if let Some(result) = inspector.call(&mut host.data, &mut inputs) {
                            call.interpreter
                                .insert_call_output(shared_memory_ref, result);
                            continue;
                        }
                    }
                    match host.make_call_frame(&inputs) {
                        Ok(new_frame) => {
                            //shared_memory_ref.new_context();
                            frames.push(new_frame);
                            shared_memory_ref.new_context();
                        }
                        Err(mut result) => {
                            //println!("Result returned right away: {:#?}", result);
                            if let Some(inspector) = host.inspector.as_mut() {
                                result = inspector.call_end(&mut host.data, result);
                            }
                            call.interpreter
                                .insert_call_output(shared_memory_ref, result);
                        }
                    };
                }
                // run sub create
                revm_interpreter::InterpreterAction::Create { mut inputs } => {
                    // Call inspector if it is some.
                    if let Some(inspector) = host.inspector.as_mut() {
                        if let Some((result, address)) =
                            inspector.create(&mut host.data, &mut inputs)
                        {
                            call.interpreter.insert_create_output(result, address);
                            continue;
                        }
                    }

                    match host.make_create_frame(&inputs) {
                        Ok(new_frame) => {
                            shared_memory_ref.new_context();
                            frames.push(new_frame);
                        }
                        Err(mut result) => {
                            let mut address = None;
                            if let Some(inspector) = host.inspector.as_mut() {
                                let ret = inspector.create_end(
                                    &mut host.data,
                                    result,
                                    call.created_address,
                                );
                                result = ret.0;
                                address = ret.1;
                            }
                            // insert result of the failed creation of create frame.
                            call.interpreter.insert_create_output(result, address);
                        }
                    };
                }
                revm_interpreter::InterpreterAction::Return { mut result } => {
                    if let Some(inspector) = host.inspector.as_mut() {
                        result = if call.is_create {
                            let (result, address) =
                                inspector.create_end(&mut host.data, result, call.created_address);
                            call.created_address = address;
                            result
                        } else {
                            inspector.call_end(&mut host.data, result)
                        }
                    }

                    let address = call.created_address;
                    let checkpoint = call.checkpoint;
                    let is_create = call.is_create;

                    // pop last interpreter frame.
                    frames.pop();
                    shared_memory_ref.free_context();

                    // break from loop sa this is last frame.
                    if frames.is_empty() {
                        if is_create {
                            let (result, _) =
                                host.create_return(result, checkpoint, address.unwrap());
                            return result;
                        } else {
                            // revert changes or not.
                            if matches!(result.result, return_ok!()) {
                                host.data.journaled_state.checkpoint_commit();
                            } else {
                                host.data.journaled_state.checkpoint_revert(checkpoint);
                            }
                            return result;
                        }
                    }
                    let previous_call = frames.last_mut().unwrap();

                    if is_create {
                        let (result, address) =
                            host.create_return(result, checkpoint, address.unwrap());
                        previous_call
                            .interpreter
                            .insert_create_output(result, Some(address))
                    } else {
                        // revert changes or not.
                        if matches!(result.result, return_ok!()) {
                            host.data.journaled_state.checkpoint_commit();
                        } else {
                            host.data.journaled_state.checkpoint_revert(checkpoint);
                        }

                        previous_call
                            .interpreter
                            .insert_call_output(shared_memory_ref, result)
                    }
                    call = previous_call;
                    continue;
                }
            }
            // Host error if present on execution
            call = frames.last_mut().unwrap();
        }
    }

    pub fn make_call_frame<'b, 'c>(
        &'b mut self,
        inputs: &CallInputs,
    ) -> Result<CallFrame<'c>, InterpreterResult> {
        let gas = Gas::new(inputs.gas_limit);

        let return_result = |instruction_result: InstructionResult| {
            Err(InterpreterResult {
                result: instruction_result,
                gas,
                output: Bytes::new(),
            })
        };

        // Check depth
        if self.data.journaled_state.depth() > CALL_STACK_LIMIT {
            return return_result(InstructionResult::CallTooDeep);
        }

        let account = match self
            .data
            .journaled_state
            .load_code(inputs.contract, self.data.db)
        {
            Ok((account, _)) => account,
            Err(e) => {
                self.data.error = Some(e);
                return return_result(InstructionResult::FatalExternalError);
            }
        };
        let code_hash = account.info.code_hash();
        let bytecode = account.info.code.clone().unwrap_or_default();

        // Create subroutine checkpoint
        let checkpoint = self.data.journaled_state.checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if inputs.transfer.value == U256::ZERO {
            self.load_account(inputs.context.address);
            self.data.journaled_state.touch(&inputs.context.address);
        }

        // Transfer value from caller to called account
        if let Err(e) = self.data.journaled_state.transfer(
            &inputs.transfer.source,
            &inputs.transfer.target,
            inputs.transfer.value,
            self.data.db,
        ) {
            //println!("transfer error");
            self.data.journaled_state.checkpoint_revert(checkpoint);
            return return_result(e);
        }

        if let Some(precompile) = self.data.precompiles.get(&inputs.contract) {
            //println!("Call precompile");
            let result = self.call_precompile(precompile, &inputs, gas);
            if matches!(result.result, return_ok!()) {
                self.data.journaled_state.checkpoint_commit();
            } else {
                self.data.journaled_state.checkpoint_revert(checkpoint);
            }
            Err(result)
        } else if !bytecode.is_empty() {
            let contract = Box::new(Contract::new_with_context(
                inputs.input.clone(),
                bytecode,
                code_hash,
                &inputs.context,
            ));
            // Create interpreter and execute subcall and push new frame.
            Ok(CallFrame {
                is_create: false,
                checkpoint: checkpoint,
                created_address: None,
                interpreter: Interpreter::new(contract, gas.limit(), inputs.is_static),
            })
        } else {
            self.data.journaled_state.checkpoint_commit();
            return_result(InstructionResult::Stop)
        }
    }

    pub fn make_create_frame<'b, 'c>(
        &'b mut self,
        inputs: &CreateInputs,
    ) -> Result<CallFrame<'c>, InterpreterResult> {
        // Prepare crate.
        let gas = Gas::new(inputs.gas_limit);

        let return_error = |e| {
            Err(InterpreterResult {
                result: e,
                gas,
                output: Bytes::new(),
            })
        };

        // Check depth
        if self.data.journaled_state.depth() > CALL_STACK_LIMIT {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Fetch balance of caller.
        let Some((caller_balance, _)) = self.balance(inputs.caller) else {
            return return_error(InstructionResult::FatalExternalError);
        };

        // Check if caller has enough balance to send to the created contract.
        if caller_balance < inputs.value {
            return return_error(InstructionResult::OutOfFund);
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce;
        if let Some(nonce) = self.data.journaled_state.inc_nonce(inputs.caller) {
            old_nonce = nonce - 1;
        } else {
            return return_error(InstructionResult::Return);
        }

        // Create address
        let code_hash = keccak256(&inputs.init_code);
        let created_address = inputs.created_address_with_hash(old_nonce, &code_hash);

        // Load account so it needs to be marked as warm for access list.
        if self
            .data
            .journaled_state
            .load_account(created_address, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .is_err()
        {
            return return_error(InstructionResult::FatalExternalError);
        }

        // create account, transfer funds and make the journal checkpoint.
        let checkpoint = match self
            .data
            .journaled_state
            .create_account_checkpoint::<GSPEC>(inputs.caller, created_address, inputs.value)
        {
            Ok(checkpoint) => checkpoint,
            Err(e) => {
                return return_error(e);
            }
        };

        let bytecode = Bytecode::new_raw(inputs.init_code.clone());

        let contract = Box::new(Contract::new(
            Bytes::new(),
            bytecode,
            code_hash,
            created_address,
            inputs.caller,
            inputs.value,
        ));

        Ok(CallFrame {
            is_create: true,
            checkpoint: checkpoint,
            created_address: Some(created_address),
            interpreter: Interpreter::new(contract, gas.limit(), false),
        })
    }
}

impl<'a, GSPEC: Spec + 'static, DB: Database> EVMImpl<'a, GSPEC, DB> {
    pub fn new(
        db: &'a mut DB,
        env: &'a mut Env,
        inspector: Option<&'a mut dyn Inspector<DB>>,
        precompiles: Precompiles,
    ) -> Self {
        let journaled_state = JournaledState::new(
            GSPEC::SPEC_ID,
            precompiles
                .addresses()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
        );
        // If T is present it should be a generic T that modifies handler.
        let instruction_table = if inspector.is_some() {
            let instruction_table = make_boxed_instruction_table::<Self, GSPEC, _>(
                make_instruction_table::<Self, GSPEC>(),
                inspector_instruction,
            );
            InstructionTables::Boxed(Arc::new(instruction_table))
        } else {
            InstructionTables::Plain(Arc::new(make_instruction_table::<Self, GSPEC>()))
        };
        #[cfg(feature = "optimism")]
        let mut handler = if env.cfg.optimism {
            Handler::optimism::<GSPEC>()
        } else {
            Handler::mainnet::<GSPEC>()
        };
        #[cfg(not(feature = "optimism"))]
        let mut handler = Handler::mainnet::<GSPEC>();

        if env.cfg.is_beneficiary_reward_disabled() {
            // do nothing
            handler.reward_beneficiary = |_, _| Ok(());
        }

        Self {
            data: EVMData {
                env,
                journaled_state,
                db,
                error: None,
                precompiles,
                #[cfg(feature = "optimism")]
                l1_block_info: None,
            },
            inspector,
            instruction_table,
            handler,
            _phantomdata: PhantomData {},
        }
    }

    /// Pre verify transaction.
    pub fn preverify_transaction_inner(&mut self) -> Result<(), EVMError<DB::Error>> {
        let env = self.env();

        // Important: validate block before tx.
        env.validate_block_env::<GSPEC>()?;
        env.validate_tx::<GSPEC>()?;

        let initial_gas_spend = initial_tx_gas::<GSPEC>(
            &env.tx.data,
            env.tx.transact_to.is_create(),
            &env.tx.access_list,
        );

        // Additional check to see if limit is big enough to cover initial gas.
        if initial_gas_spend > env.tx.gas_limit {
            return Err(InvalidTransaction::CallGasCostMoreThanGasLimit.into());
        }

        // load acc
        let tx_caller = env.tx.caller;
        let (caller_account, _) = self
            .data
            .journaled_state
            .load_account(tx_caller, self.data.db)
            .map_err(EVMError::Database)?;

        self.data
            .env
            .validate_tx_against_state(caller_account)
            .map_err(Into::into)
    }

    /// Transact preverified transaction.
    pub fn transact_preverified_inner(&mut self) -> EVMResult<DB::Error> {
        let env = &self.data.env;
        let tx_caller = env.tx.caller;
        let tx_value = env.tx.value;
        let tx_data = env.tx.data.clone();
        let tx_gas_limit = env.tx.gas_limit;

        // the L1-cost fee is only computed for Optimism non-deposit transactions.
        #[cfg(feature = "optimism")]
        let tx_l1_cost = if env.cfg.optimism && env.tx.optimism.source_hash.is_none() {
            let l1_block_info =
                optimism::L1BlockInfo::try_fetch(self.data.db).map_err(EVMError::Database)?;

            let Some(enveloped_tx) = &env.tx.optimism.enveloped_tx else {
                panic!("[OPTIMISM] Failed to load enveloped transaction.");
            };
            let tx_l1_cost = l1_block_info.calculate_tx_l1_cost::<GSPEC>(enveloped_tx);

            // storage l1 block info for later use.
            self.data.l1_block_info = Some(l1_block_info);

            tx_l1_cost
        } else {
            U256::ZERO
        };

        let initial_gas_spend = initial_tx_gas::<GSPEC>(
            &tx_data,
            env.tx.transact_to.is_create(),
            &env.tx.access_list,
        );

        // load coinbase
        // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
        if GSPEC::enabled(SHANGHAI) {
            self.data
                .journaled_state
                .initial_account_load(self.data.env.block.coinbase, &[], self.data.db)
                .map_err(EVMError::Database)?;
        }

        self.data.load_access_list()?;

        // load acc
        let journal = &mut self.data.journaled_state;

        #[cfg(feature = "optimism")]
        if self.data.env.cfg.optimism {
            EVMImpl::<GSPEC, DB>::commit_mint_value(
                tx_caller,
                self.data.env.tx.optimism.mint,
                self.data.db,
                journal,
            )?;

            let is_deposit = self.data.env.tx.optimism.source_hash.is_some();
            EVMImpl::<GSPEC, DB>::remove_l1_cost(
                is_deposit,
                tx_caller,
                tx_l1_cost,
                self.data.db,
                journal,
            )?;
        }

        let (caller_account, _) = journal
            .load_account(tx_caller, self.data.db)
            .map_err(EVMError::Database)?;

        // Subtract gas costs from the caller's account.
        // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
        let mut gas_cost =
            U256::from(tx_gas_limit).saturating_mul(self.data.env.effective_gas_price());

        // EIP-4844
        if GSPEC::enabled(CANCUN) {
            let data_fee = self.data.env.calc_data_fee().expect("already checked");
            gas_cost = gas_cost.saturating_add(data_fee);
        }

        caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);

        // touch account so we know it is changed.
        caller_account.mark_touch();

        let transact_gas_limit = tx_gas_limit - initial_gas_spend;

        // call inner handling of call/create
        let first_frame = match self.data.env.tx.transact_to {
            TransactTo::Call(address) => {
                // Nonce is already checked
                caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);

                self.make_call_frame(&mut CallInputs {
                    contract: address,
                    transfer: Transfer {
                        source: tx_caller,
                        target: address,
                        value: tx_value,
                    },
                    input: tx_data,
                    gas_limit: transact_gas_limit,
                    context: CallContext {
                        caller: tx_caller,
                        address,
                        code_address: address,
                        apparent_value: tx_value,
                        scheme: CallScheme::Call,
                    },
                    is_static: false,
                })
            }
            TransactTo::Create(scheme) => self.make_create_frame(&mut CreateInputs {
                caller: tx_caller,
                scheme,
                value: tx_value,
                init_code: tx_data,
                gas_limit: transact_gas_limit,
            }),
        };
        let mut created_address = None;
        // start main loop if frame is created correctly
        let interpreter_result = match first_frame {
            Ok(first_frame) => {
                created_address = first_frame.created_address;
                let table = self.instruction_table.clone();
                match table {
                    InstructionTables::Plain(table) => self.main_loop(&table, first_frame),
                    InstructionTables::Boxed(table) => self.main_loop(&table, first_frame),
                }
            }
            Err(interpreter_result) => interpreter_result,
        };

        let handler = &self.handler;
        let data = &mut self.data;

        // handle output of call/create calls.
        let mut gas =
            handler.call_return(data.env, interpreter_result.result, interpreter_result.gas);

        // set refund. Refund amount depends on hardfork.
        gas.set_refund(handler.calculate_gas_refund(data.env, &gas) as i64);

        // Reimburse the caller
        handler.reimburse_caller(data, &gas)?;

        // Reward beneficiary
        handler.reward_beneficiary(data, &gas)?;

        // output of execution
        let output = match data.env.tx.transact_to {
            TransactTo::Call(_) => Output::Call(interpreter_result.output),
            TransactTo::Create(_) => Output::Create(interpreter_result.output, created_address),
        };

        // main return
        handler.main_return(data, interpreter_result.result, output, &gas)
    }

    #[inline]
    fn create_return(
        &mut self,
        mut interpreter_result: InterpreterResult,
        journal_checkpoint: JournalCheckpoint,
        created_address: Address,
    ) -> (InterpreterResult, Address) {
        // Host error if present on execution
        match interpreter_result.result {
            return_ok!() => {
                // if ok, check contract creation limit and calculate gas deduction on output len.
                //
                // EIP-3541: Reject new contract code starting with the 0xEF byte
                if GSPEC::enabled(LONDON)
                    && !interpreter_result.output.is_empty()
                    && interpreter_result.output.first() == Some(&0xEF)
                {
                    self.data
                        .journaled_state
                        .checkpoint_revert(journal_checkpoint);
                    interpreter_result.result = InstructionResult::CreateContractStartingWithEF;
                    return (interpreter_result, created_address);
                }

                // EIP-170: Contract code size limit
                // By default limit is 0x6000 (~25kb)
                if GSPEC::enabled(SPURIOUS_DRAGON)
                    && interpreter_result.output.len()
                        > self
                            .data
                            .env
                            .cfg
                            .limit_contract_code_size
                            .unwrap_or(MAX_CODE_SIZE)
                {
                    self.data
                        .journaled_state
                        .checkpoint_revert(journal_checkpoint);
                    interpreter_result.result = InstructionResult::CreateContractSizeLimit;
                    return (interpreter_result, created_address);
                }
                if crate::USE_GAS {
                    let gas_for_code = interpreter_result.output.len() as u64 * gas::CODEDEPOSIT;
                    if !interpreter_result.gas.record_cost(gas_for_code) {
                        // record code deposit gas cost and check if we are out of gas.
                        // EIP-2 point 3: If contract creation does not have enough gas to pay for the
                        // final gas fee for adding the contract code to the state, the contract
                        //  creation fails (i.e. goes out-of-gas) rather than leaving an empty contract.
                        if GSPEC::enabled(HOMESTEAD) {
                            self.data
                                .journaled_state
                                .checkpoint_revert(journal_checkpoint);
                            interpreter_result.result = InstructionResult::OutOfGas;
                            return (interpreter_result, created_address);
                        } else {
                            interpreter_result.output = Bytes::new();
                        }
                    }
                }
                // if we have enough gas
                self.data.journaled_state.checkpoint_commit();
                // Do analysis of bytecode straight away.
                let bytecode = match self.data.env.cfg.perf_analyse_created_bytecodes {
                    AnalysisKind::Raw => Bytecode::new_raw(interpreter_result.output.clone()),
                    AnalysisKind::Check => {
                        Bytecode::new_raw(interpreter_result.output.clone()).to_checked()
                    }
                    AnalysisKind::Analyse => {
                        to_analysed(Bytecode::new_raw(interpreter_result.output.clone()))
                    }
                };
                self.data
                    .journaled_state
                    .set_code(created_address, bytecode);
                interpreter_result.result = InstructionResult::Return;
                (interpreter_result, created_address)
            }
            _ => {
                self.data
                    .journaled_state
                    .checkpoint_revert(journal_checkpoint);
                (interpreter_result, created_address)
            }
        }
    }

    /// Call precompile contract
    fn call_precompile(
        &mut self,
        precompile: Precompile,
        inputs: &CallInputs,
        gas: Gas,
    ) -> InterpreterResult {
        let input_data = &inputs.input;

        let out = match precompile {
            Precompile::Standard(fun) => fun(input_data, gas.limit()),
            Precompile::Env(fun) => fun(input_data, gas.limit(), self.env()),
        };

        let mut result = InterpreterResult {
            result: InstructionResult::Return,
            gas,
            output: Bytes::new(),
        };

        match out {
            Ok((gas_used, data)) => {
                if !crate::USE_GAS || result.gas.record_cost(gas_used) {
                    result.result = InstructionResult::Return;
                    result.output = Bytes::from(data);
                } else {
                    result.result = InstructionResult::PrecompileOOG;
                }
            }
            Err(e) => {
                result.result = if precompile::Error::OutOfGas == e {
                    InstructionResult::PrecompileOOG
                } else {
                    InstructionResult::PrecompileError
                };
            }
        }
        result
    }
}

impl<'a, GSPEC: Spec + 'static, DB: Database> Host for EVMImpl<'a, GSPEC, DB> {
    fn env(&mut self) -> &mut Env {
        self.data.env()
    }

    fn block_hash(&mut self, number: U256) -> Option<B256> {
        self.data.block_hash(number)
    }

    fn load_account(&mut self, address: Address) -> Option<(bool, bool)> {
        self.data.load_account(address)
    }

    fn balance(&mut self, address: Address) -> Option<(U256, bool)> {
        self.data.balance(address)
    }

    fn code(&mut self, address: Address) -> Option<(Bytecode, bool)> {
        self.data.code(address)
    }

    /// Get code hash of address.
    fn code_hash(&mut self, address: Address) -> Option<(B256, bool)> {
        self.data.code_hash(address)
    }

    fn sload(&mut self, address: Address, index: U256) -> Option<(U256, bool)> {
        self.data.sload(address, index)
    }

    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        self.data.sstore(address, index, value)
    }

    fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.data.tload(address, index)
    }

    fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.data.tstore(address, index, value)
    }

    fn log(&mut self, address: Address, topics: Vec<B256>, data: Bytes) {
        if let Some(inspector) = self.inspector.as_mut() {
            inspector.log(&mut self.data, &address, &topics, &data);
        }
        let log = Log {
            address,
            topics,
            data,
        };
        self.data.journaled_state.log(log);
    }

    fn selfdestruct(&mut self, address: Address, target: Address) -> Option<SelfDestructResult> {
        if let Some(inspector) = self.inspector.as_mut() {
            let acc = self.data.journaled_state.state.get(&address).unwrap();
            inspector.selfdestruct(address, target, acc.info.balance);
        }
        self.data
            .journaled_state
            .selfdestruct(address, target, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    // fn create(
    //     &mut self,
    //     inputs: &mut CreateInputs,
    //     shared_memory: &mut SharedMemory,
    // ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
    //     // Call inspector
    //     if let Some(inspector) = self.inspector.as_mut() {
    //         let (ret, address, gas, out) = inspector.create(&mut self.data, inputs);
    //         if ret != InstructionResult::Continue {
    //             return inspector.create_end(&mut self.data, inputs, ret, address, gas, out);
    //         }
    //     }
    //     let ret = self.create_inner(inputs, shared_memory);
    //     if let Some(inspector) = self.inspector.as_mut() {
    //         inspector.create_end(
    //             &mut self.data,
    //             inputs,
    //             ret.result,
    //             ret.created_address,
    //             ret.gas,
    //             ret.return_value,
    //         )
    //     } else {
    //         (ret.result, ret.created_address, ret.gas, ret.return_value)
    //     }
    // }

    // fn call(
    //     &mut self,
    //     inputs: &mut CallInputs,
    //     shared_memory: &mut SharedMemory,
    // ) -> (InstructionResult, Gas, Bytes) {
    //     if let Some(inspector) = self.inspector.as_mut() {
    //         let (ret, gas, out) = inspector.call(&mut self.data, inputs);
    //         if ret != InstructionResult::Continue {
    //             return inspector.call_end(&mut self.data, inputs, gas, ret, out);
    //         }
    //     }
    //     let ret = self.call_inner(inputs, shared_memory);
    //     if let Some(inspector) = self.inspector.as_mut() {
    //         inspector.call_end(&mut self.data, inputs, ret.gas, ret.result, ret.output)
    //     } else {
    //         (ret.result, ret.gas, ret.output)
    //     }
    // }
}

#[cfg(feature = "optimism")]
#[cfg(test)]
mod tests {
    use super::*;

    use crate::db::InMemoryDB;
    use crate::primitives::{specification::BedrockSpec, state::AccountInfo, SpecId};

    #[test]
    fn test_commit_mint_value() {
        let caller = Address::ZERO;
        let mint_value = Some(1u128);
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                nonce: 0,
                balance: U256::from(100),
                code_hash: B256::ZERO,
                code: None,
            },
        );
        let mut journal = JournaledState::new(SpecId::BERLIN, vec![]);
        journal
            .initial_account_load(caller, &[U256::from(100)], &mut db)
            .unwrap();
        assert!(EVMImpl::<BedrockSpec, InMemoryDB>::commit_mint_value(
            caller,
            mint_value,
            &mut db,
            &mut journal
        )
        .is_ok(),);

        // Check the account balance is updated.
        let (account, _) = journal.load_account(caller, &mut db).unwrap();
        assert_eq!(account.info.balance, U256::from(101));

        // No mint value should be a no-op.
        assert!(EVMImpl::<BedrockSpec, InMemoryDB>::commit_mint_value(
            caller,
            None,
            &mut db,
            &mut journal
        )
        .is_ok(),);
        let (account, _) = journal.load_account(caller, &mut db).unwrap();
        assert_eq!(account.info.balance, U256::from(101));
    }

    #[test]
    fn test_remove_l1_cost_non_deposit() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        let mut journal = JournaledState::new(SpecId::BERLIN, vec![]);
        let slots = &[U256::from(100)];
        journal
            .initial_account_load(caller, slots, &mut db)
            .unwrap();
        assert!(EVMImpl::<BedrockSpec, InMemoryDB>::remove_l1_cost(
            true,
            caller,
            U256::ZERO,
            &mut db,
            &mut journal
        )
        .is_ok(),);
    }

    #[test]
    fn test_remove_l1_cost() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                nonce: 0,
                balance: U256::from(100),
                code_hash: B256::ZERO,
                code: None,
            },
        );
        let mut journal = JournaledState::new(SpecId::BERLIN, vec![]);
        journal
            .initial_account_load(caller, &[U256::from(100)], &mut db)
            .unwrap();
        assert!(EVMImpl::<BedrockSpec, InMemoryDB>::remove_l1_cost(
            false,
            caller,
            U256::from(1),
            &mut db,
            &mut journal
        )
        .is_ok(),);

        // Check the account balance is updated.
        let (account, _) = journal.load_account(caller, &mut db).unwrap();
        assert_eq!(account.info.balance, U256::from(99));
    }

    #[test]
    fn test_remove_l1_cost_lack_of_funds() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                nonce: 0,
                balance: U256::from(100),
                code_hash: B256::ZERO,
                code: None,
            },
        );
        let mut journal = JournaledState::new(SpecId::BERLIN, vec![]);
        journal
            .initial_account_load(caller, &[U256::from(100)], &mut db)
            .unwrap();
        assert_eq!(
            EVMImpl::<BedrockSpec, InMemoryDB>::remove_l1_cost(
                false,
                caller,
                U256::from(101),
                &mut db,
                &mut journal
            ),
            Err(EVMError::Transaction(
                InvalidTransaction::LackOfFundForMaxFee {
                    fee: 101u64,
                    balance: U256::from(100),
                },
            ))
        );
    }
}
