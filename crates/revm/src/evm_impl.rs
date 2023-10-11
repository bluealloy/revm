use crate::handler::Handler;
use crate::interpreter::{
    analysis::to_analysed, gas, return_ok, CallContext, CallInputs, CallScheme, Contract,
    CreateInputs, CreateScheme, Gas, Host, InstructionResult, Interpreter, SelfDestructResult,
    SuccessOrHalt, Transfer,
};
use crate::journaled_state::{is_precompile, JournalCheckpoint};
use crate::primitives::{
    keccak256, Address, AnalysisKind, Bytecode, Bytes, EVMError, EVMResult, Env, ExecutionResult,
    InvalidTransaction, Log, Output, ResultAndState, Spec, SpecId::*, TransactTo, B256, U256,
};
use crate::{db::Database, journaled_state::JournaledState, precompile, Inspector};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;
use revm_interpreter::gas::initial_tx_gas;
use revm_interpreter::{SharedMemory, MAX_CODE_SIZE};
use revm_precompile::{Precompile, Precompiles};

#[cfg(feature = "optimism")]
use crate::optimism;

/// EVM call stack limit.
pub const CALL_STACK_LIMIT: u64 = 1024;

pub struct EVMData<'a, DB: Database> {
    pub env: &'a mut Env,
    pub journaled_state: JournaledState,
    pub db: &'a mut DB,
    pub error: Option<DB::Error>,
    pub precompiles: Precompiles,
    /// Used as temporary value holder to store L1 block info.
    #[cfg(feature = "optimism")]
    pub l1_block_info: Option<optimism::L1BlockInfo>,
}

pub struct EVMImpl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> {
    data: EVMData<'a, DB>,
    inspector: &'a mut dyn Inspector<DB>,
    handler: Handler<DB>,
    _phantomdata: PhantomData<GSPEC>,
}

struct PreparedCreate {
    gas: Gas,
    created_address: Address,
    checkpoint: JournalCheckpoint,
    contract: Box<Contract>,
}

struct CreateResult {
    result: InstructionResult,
    created_address: Option<Address>,
    gas: Gas,
    return_value: Bytes,
}

struct PreparedCall {
    gas: Gas,
    checkpoint: JournalCheckpoint,
    contract: Box<Contract>,
}

struct CallResult {
    result: InstructionResult,
    gas: Gas,
    return_value: Bytes,
}

pub trait Transact<DBError> {
    /// Run checks that could make transaction fail before call/create.
    fn preverify_transaction(&mut self) -> Result<(), EVMError<DBError>>;

    /// Skip pre-verification steps and execute the transaction.
    fn transact_preverified(&mut self) -> EVMResult<DBError>;

    /// Execute transaction by running pre-verification steps and then transaction itself.
    #[inline]
    fn transact(&mut self) -> EVMResult<DBError> {
        self.preverify_transaction()
            .and_then(|_| self.transact_preverified())
    }
}

impl<'a, DB: Database> EVMData<'a, DB> {
    /// Load access list for berlin hardfork.
    ///
    /// Loading of accounts/storages is needed to make them warm.
    #[inline]
    fn load_access_list(&mut self) -> Result<(), EVMError<DB::Error>> {
        for (address, slots) in self.env.tx.access_list.iter() {
            self.journaled_state
                .initial_account_load(*address, slots, self.db)
                .map_err(EVMError::Database)?;
        }
        Ok(())
    }
}

#[cfg(feature = "optimism")]
impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> EVMImpl<'a, GSPEC, DB, INSPECT> {
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

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> Transact<DB::Error>
    for EVMImpl<'a, GSPEC, DB, INSPECT>
{
    fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
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

    fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        let env = &self.data.env;
        let tx_caller = env.tx.caller;
        let tx_value = env.tx.value;
        let tx_data = env.tx.data.clone();
        let tx_gas_limit = env.tx.gas_limit;

        #[cfg(feature = "optimism")]
        let tx_l1_cost = {
            let is_deposit = env.tx.optimism.source_hash.is_some();

            let l1_block_info =
                optimism::L1BlockInfo::try_fetch(self.data.db, self.data.env.cfg.optimism)
                    .map_err(EVMError::Database)?;

            // Perform this calculation optimistically to avoid cloning the enveloped tx.
            let tx_l1_cost = l1_block_info.as_ref().map(|l1_block_info| {
                env.tx
                    .optimism
                    .enveloped_tx
                    .as_ref()
                    .map(|enveloped_tx| {
                        l1_block_info.calculate_tx_l1_cost::<GSPEC>(enveloped_tx, is_deposit)
                    })
                    .unwrap_or(U256::ZERO)
            });
            // storage l1 block info for later use.
            self.data.l1_block_info = l1_block_info;

            //
            let Some(tx_l1_cost) = tx_l1_cost else {
                panic!("[OPTIMISM] L1 Block Info could not be loaded from the DB.")
            };

            tx_l1_cost
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
            EVMImpl::<GSPEC, DB, INSPECT>::commit_mint_value(
                tx_caller,
                self.data.env.tx.optimism.mint,
                self.data.db,
                journal,
            )?;

            let is_deposit = self.data.env.tx.optimism.source_hash.is_some();
            EVMImpl::<GSPEC, DB, INSPECT>::remove_l1_cost(
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

        #[cfg(feature = "memory_limit")]
        let mut shared_memory = SharedMemory::new_with_memory_limit(self.data.env.cfg.memory_limit);
        #[cfg(not(feature = "memory_limit"))]
        let mut shared_memory = SharedMemory::new();

        // call inner handling of call/create
        let (call_result, ret_gas, output) = match self.data.env.tx.transact_to {
            TransactTo::Call(address) => {
                // Nonce is already checked
                caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);

                let (exit, gas, bytes) = self.call(
                    &mut CallInputs {
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
                    },
                    &mut shared_memory,
                );
                (exit, gas, Output::Call(bytes))
            }
            TransactTo::Create(scheme) => {
                let (exit, address, ret_gas, bytes) = self.create(
                    &mut CreateInputs {
                        caller: tx_caller,
                        scheme,
                        value: tx_value,
                        init_code: tx_data,
                        gas_limit: transact_gas_limit,
                    },
                    &mut shared_memory,
                );
                (exit, ret_gas, Output::Create(bytes, address))
            }
        };

        let handler = &self.handler;
        let data = &mut self.data;

        // handle output of call/create calls.
        let gas = handler.call_return(data.env, call_result, ret_gas);

        let gas_refunded = handler.calculate_gas_refund(data.env, &gas);

        // Reimburse the caller
        handler.reimburse_caller(data, &gas, gas_refunded)?;

        // Reward beneficiary
        handler.reward_beneficiary(data, &gas, gas_refunded)?;

        // used gas with refund calculated.
        let final_gas_used = gas.spend() - gas_refunded;

        // reset journal and return present state.
        let (state, logs) = self.data.journaled_state.finalize();

        let result = match call_result.into() {
            SuccessOrHalt::Success(reason) => ExecutionResult::Success {
                reason,
                gas_used: final_gas_used,
                gas_refunded,
                logs,
                output,
            },
            SuccessOrHalt::Revert => ExecutionResult::Revert {
                gas_used: final_gas_used,
                output: match output {
                    Output::Call(return_value) => return_value,
                    Output::Create(return_value, _) => return_value,
                },
            },
            SuccessOrHalt::Halt(reason) => {
                // Post-regolith, if the transaction is a deposit transaction and the
                // output is a contract creation, increment the account nonce even if
                // the transaction halts.
                #[cfg(feature = "optimism")]
                {
                    let is_deposit = self.data.env.tx.optimism.source_hash.is_some();
                    let is_creation = matches!(output, Output::Create(_, _));
                    let regolith_enabled = GSPEC::enabled(REGOLITH);
                    let optimism_regolith = self.data.env.cfg.optimism && regolith_enabled;
                    if is_deposit && is_creation && optimism_regolith {
                        let (acc, _) = self
                            .data
                            .journaled_state
                            .load_account(tx_caller, self.data.db)
                            .map_err(EVMError::Database)?;
                        acc.info.nonce = acc.info.nonce.checked_add(1).unwrap_or(u64::MAX);
                    }
                }
                ExecutionResult::Halt {
                    reason,
                    gas_used: final_gas_used,
                }
            }
            SuccessOrHalt::FatalExternalError => {
                return Err(EVMError::Database(self.data.error.take().unwrap()));
            }
            SuccessOrHalt::InternalContinue => {
                panic!("Internal return flags should remain internal {call_result:?}")
            }
        };

        Ok(ResultAndState { result, state })
    }
}

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> EVMImpl<'a, GSPEC, DB, INSPECT> {
    pub fn new(
        db: &'a mut DB,
        env: &'a mut Env,
        inspector: &'a mut dyn Inspector<DB>,
        precompiles: Precompiles,
    ) -> Self {
        let journaled_state = JournaledState::new(precompiles.len(), GSPEC::SPEC_ID);
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
            handler: Handler::mainnet::<GSPEC>(),
            _phantomdata: PhantomData {},
        }
    }

    #[inline(never)]
    fn prepare_create(&mut self, inputs: &CreateInputs) -> Result<PreparedCreate, CreateResult> {
        let gas = Gas::new(inputs.gas_limit);

        // Check depth of calls
        if self.data.journaled_state.depth() > CALL_STACK_LIMIT {
            return Err(CreateResult {
                result: InstructionResult::CallTooDeep,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            });
        }

        // Fetch balance of caller.
        let Some((caller_balance, _)) = self.balance(inputs.caller) else {
            return Err(CreateResult {
                result: InstructionResult::FatalExternalError,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            });
        };

        // Check if caller has enough balance to send to the crated contract.
        if caller_balance < inputs.value {
            return Err(CreateResult {
                result: InstructionResult::OutOfFund,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            });
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce;
        if let Some(nonce) = self.data.journaled_state.inc_nonce(inputs.caller) {
            old_nonce = nonce - 1;
        } else {
            return Err(CreateResult {
                result: InstructionResult::Return,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            });
        }

        // Create address
        let code_hash = keccak256(&inputs.init_code);
        let created_address = match inputs.scheme {
            CreateScheme::Create => inputs.caller.create(old_nonce),
            CreateScheme::Create2 { salt } => inputs.caller.create2(B256::from(salt), code_hash),
        };

        // Load account so it needs to be marked as warm for access list.
        if self
            .data
            .journaled_state
            .load_account(created_address, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .is_err()
        {
            return Err(CreateResult {
                result: InstructionResult::FatalExternalError,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            });
        }

        // create account, transfer funds and make the journal checkpoint.
        let checkpoint = match self
            .data
            .journaled_state
            .create_account_checkpoint::<GSPEC>(inputs.caller, created_address, inputs.value)
        {
            Ok(checkpoint) => checkpoint,
            Err(e) => {
                return Err(CreateResult {
                    result: e,
                    created_address: None,
                    gas,
                    return_value: Bytes::new(),
                });
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

        Ok(PreparedCreate {
            gas,
            created_address,
            checkpoint,
            contract,
        })
    }

    /// EVM create opcode for both initial crate and CREATE and CREATE2 opcodes.
    fn create_inner(
        &mut self,
        inputs: &CreateInputs,
        shared_memory: &mut SharedMemory,
    ) -> CreateResult {
        // Prepare crate.
        let prepared_create = match self.prepare_create(inputs) {
            Ok(o) => o,
            Err(e) => return e,
        };

        // Create new interpreter and execute initcode
        let (exit_reason, mut bytes, mut gas) = self.run_interpreter(
            prepared_create.contract,
            prepared_create.gas.limit(),
            false,
            shared_memory,
        );

        // Host error if present on execution
        match exit_reason {
            return_ok!() => {
                // if ok, check contract creation limit and calculate gas deduction on output len.
                //
                // EIP-3541: Reject new contract code starting with the 0xEF byte
                if GSPEC::enabled(LONDON) && !bytes.is_empty() && bytes.first() == Some(&0xEF) {
                    self.data
                        .journaled_state
                        .checkpoint_revert(prepared_create.checkpoint);
                    return CreateResult {
                        result: InstructionResult::CreateContractStartingWithEF,
                        created_address: Some(prepared_create.created_address),
                        gas,
                        return_value: bytes,
                    };
                }

                // EIP-170: Contract code size limit
                // By default limit is 0x6000 (~25kb)
                if GSPEC::enabled(SPURIOUS_DRAGON)
                    && bytes.len()
                        > self
                            .data
                            .env
                            .cfg
                            .limit_contract_code_size
                            .unwrap_or(MAX_CODE_SIZE)
                {
                    self.data
                        .journaled_state
                        .checkpoint_revert(prepared_create.checkpoint);
                    return CreateResult {
                        result: InstructionResult::CreateContractSizeLimit,
                        created_address: Some(prepared_create.created_address),
                        gas,
                        return_value: bytes,
                    };
                }
                if crate::USE_GAS {
                    let gas_for_code = bytes.len() as u64 * gas::CODEDEPOSIT;
                    if !gas.record_cost(gas_for_code) {
                        // record code deposit gas cost and check if we are out of gas.
                        // EIP-2 point 3: If contract creation does not have enough gas to pay for the
                        // final gas fee for adding the contract code to the state, the contract
                        //  creation fails (i.e. goes out-of-gas) rather than leaving an empty contract.
                        if GSPEC::enabled(HOMESTEAD) {
                            self.data
                                .journaled_state
                                .checkpoint_revert(prepared_create.checkpoint);
                            return CreateResult {
                                result: InstructionResult::OutOfGas,
                                created_address: Some(prepared_create.created_address),
                                gas,
                                return_value: bytes,
                            };
                        } else {
                            bytes = Bytes::new();
                        }
                    }
                }
                // if we have enough gas
                self.data.journaled_state.checkpoint_commit();
                // Do analysis of bytecode straight away.
                let bytecode = match self.data.env.cfg.perf_analyse_created_bytecodes {
                    AnalysisKind::Raw => Bytecode::new_raw(bytes.clone()),
                    AnalysisKind::Check => Bytecode::new_raw(bytes.clone()).to_checked(),
                    AnalysisKind::Analyse => to_analysed(Bytecode::new_raw(bytes.clone())),
                };
                self.data
                    .journaled_state
                    .set_code(prepared_create.created_address, bytecode);
                CreateResult {
                    result: InstructionResult::Return,
                    created_address: Some(prepared_create.created_address),
                    gas,
                    return_value: bytes,
                }
            }
            _ => {
                self.data
                    .journaled_state
                    .checkpoint_revert(prepared_create.checkpoint);
                CreateResult {
                    result: exit_reason,
                    created_address: Some(prepared_create.created_address),
                    gas,
                    return_value: bytes,
                }
            }
        }
    }

    /// Create a Interpreter and run it.
    /// Returns the exit reason, return value and gas from interpreter
    pub fn run_interpreter(
        &mut self,
        contract: Box<Contract>,
        gas_limit: u64,
        is_static: bool,
        shared_memory: &mut SharedMemory,
    ) -> (InstructionResult, Bytes, Gas) {
        let mut interpreter = Box::new(Interpreter::new(
            contract,
            gas_limit,
            is_static,
            shared_memory,
        ));

        interpreter.shared_memory.new_context_memory();

        if INSPECT {
            self.inspector
                .initialize_interp(&mut interpreter, &mut self.data);
        }
        let exit_reason = if INSPECT {
            interpreter.run_inspect::<Self, GSPEC>(self)
        } else {
            interpreter.run::<Self, GSPEC>(self)
        };

        let (return_value, gas) = (interpreter.return_value(), *interpreter.gas());

        interpreter.shared_memory.free_context_memory();

        (exit_reason, return_value, gas)
    }

    /// Call precompile contract
    fn call_precompile(&mut self, inputs: &CallInputs, mut gas: Gas) -> CallResult {
        let input_data = &inputs.input;
        let contract = inputs.contract;

        let precompile = self
            .data
            .precompiles
            .get(&contract)
            .expect("Check for precompile should be already done");
        let out = match precompile {
            Precompile::Standard(fun) => fun(input_data, gas.limit()),
            Precompile::Env(fun) => fun(input_data, gas.limit(), self.env()),
        };
        match out {
            Ok((gas_used, data)) => {
                if !crate::USE_GAS || gas.record_cost(gas_used) {
                    CallResult {
                        result: InstructionResult::Return,
                        gas,
                        return_value: Bytes::from(data),
                    }
                } else {
                    CallResult {
                        result: InstructionResult::PrecompileOOG,
                        gas,
                        return_value: Bytes::new(),
                    }
                }
            }
            Err(e) => {
                let result = if precompile::Error::OutOfGas == e {
                    InstructionResult::PrecompileOOG
                } else {
                    InstructionResult::PrecompileError
                };
                CallResult {
                    result,
                    gas,
                    return_value: Bytes::new(),
                }
            }
        }
    }

    #[inline(never)]
    fn prepare_call(&mut self, inputs: &CallInputs) -> Result<PreparedCall, CallResult> {
        let gas = Gas::new(inputs.gas_limit);
        let account = match self
            .data
            .journaled_state
            .load_code(inputs.contract, self.data.db)
        {
            Ok((account, _)) => account,
            Err(e) => {
                self.data.error = Some(e);
                return Err(CallResult {
                    result: InstructionResult::FatalExternalError,
                    gas,
                    return_value: Bytes::new(),
                });
            }
        };
        let code_hash = account.info.code_hash();
        let bytecode = account.info.code.clone().unwrap_or_default();

        // Check depth
        if self.data.journaled_state.depth() > CALL_STACK_LIMIT {
            return Err(CallResult {
                result: InstructionResult::CallTooDeep,
                gas,
                return_value: Bytes::new(),
            });
        }

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
            self.data.journaled_state.checkpoint_revert(checkpoint);
            return Err(CallResult {
                result: e,
                gas,
                return_value: Bytes::new(),
            });
        }

        let contract = Box::new(Contract::new_with_context(
            inputs.input.clone(),
            bytecode,
            code_hash,
            &inputs.context,
        ));

        Ok(PreparedCall {
            gas,
            checkpoint,
            contract,
        })
    }

    /// Main contract call of the EVM.
    fn call_inner(&mut self, inputs: &CallInputs, shared_memory: &mut SharedMemory) -> CallResult {
        // Prepare call
        let prepared_call = match self.prepare_call(inputs) {
            Ok(o) => o,
            Err(e) => return e,
        };

        let ret = if is_precompile(inputs.contract, self.data.precompiles.len()) {
            self.call_precompile(inputs, prepared_call.gas)
        } else if !prepared_call.contract.bytecode.is_empty() {
            // Create interpreter and execute subcall
            let (exit_reason, bytes, gas) = self.run_interpreter(
                prepared_call.contract,
                prepared_call.gas.limit(),
                inputs.is_static,
                shared_memory,
            );
            CallResult {
                result: exit_reason,
                gas,
                return_value: bytes,
            }
        } else {
            CallResult {
                result: InstructionResult::Stop,
                gas: prepared_call.gas,
                return_value: Bytes::new(),
            }
        };

        // revert changes or not.
        if matches!(ret.result, return_ok!()) {
            self.data.journaled_state.checkpoint_commit();
        } else {
            self.data
                .journaled_state
                .checkpoint_revert(prepared_call.checkpoint);
        }

        ret
    }
}

impl<'a, GSPEC: Spec, DB: Database + 'a, const INSPECT: bool> Host
    for EVMImpl<'a, GSPEC, DB, INSPECT>
{
    fn step(&mut self, interp: &mut Interpreter) -> InstructionResult {
        self.inspector.step(interp, &mut self.data)
    }

    fn step_end(&mut self, interp: &mut Interpreter, ret: InstructionResult) -> InstructionResult {
        self.inspector.step_end(interp, &mut self.data, ret)
    }

    fn env(&mut self) -> &mut Env {
        self.data.env
    }

    fn block_hash(&mut self, number: U256) -> Option<B256> {
        self.data
            .db
            .block_hash(number)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn load_account(&mut self, address: Address) -> Option<(bool, bool)> {
        self.data
            .journaled_state
            .load_account_exist(address, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn balance(&mut self, address: Address) -> Option<(U256, bool)> {
        let db = &mut self.data.db;
        let journal = &mut self.data.journaled_state;
        let error = &mut self.data.error;
        journal
            .load_account(address, db)
            .map_err(|e| *error = Some(e))
            .ok()
            .map(|(acc, is_cold)| (acc.info.balance, is_cold))
    }

    fn code(&mut self, address: Address) -> Option<(Bytecode, bool)> {
        let journal = &mut self.data.journaled_state;
        let db = &mut self.data.db;
        let error = &mut self.data.error;

        let (acc, is_cold) = journal
            .load_code(address, db)
            .map_err(|e| *error = Some(e))
            .ok()?;
        Some((acc.info.code.clone().unwrap(), is_cold))
    }

    /// Get code hash of address.
    fn code_hash(&mut self, address: Address) -> Option<(B256, bool)> {
        let journal = &mut self.data.journaled_state;
        let db = &mut self.data.db;
        let error = &mut self.data.error;

        let (acc, is_cold) = journal
            .load_code(address, db)
            .map_err(|e| *error = Some(e))
            .ok()?;
        if acc.is_empty() {
            return Some((B256::ZERO, is_cold));
        }

        Some((acc.info.code_hash, is_cold))
    }

    fn sload(&mut self, address: Address, index: U256) -> Option<(U256, bool)> {
        // account is always warm. reference on that statement https://eips.ethereum.org/EIPS/eip-2929 see `Note 2:`
        self.data
            .journaled_state
            .sload(address, index, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        self.data
            .journaled_state
            .sstore(address, index, value, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.data.journaled_state.tload(address, index)
    }

    fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.data.journaled_state.tstore(address, index, value)
    }

    fn log(&mut self, address: Address, topics: Vec<B256>, data: Bytes) {
        if INSPECT {
            self.inspector.log(&mut self.data, &address, &topics, &data);
        }
        let log = Log {
            address,
            topics,
            data,
        };
        self.data.journaled_state.log(log);
    }

    fn selfdestruct(&mut self, address: Address, target: Address) -> Option<SelfDestructResult> {
        if INSPECT {
            let acc = self.data.journaled_state.state.get(&address).unwrap();
            self.inspector
                .selfdestruct(address, target, acc.info.balance);
        }
        self.data
            .journaled_state
            .selfdestruct(address, target, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn create(
        &mut self,
        inputs: &mut CreateInputs,
        shared_memory: &mut SharedMemory,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        // Call inspector
        if INSPECT {
            let (ret, address, gas, out) = self.inspector.create(&mut self.data, inputs);
            if ret != InstructionResult::Continue {
                return self
                    .inspector
                    .create_end(&mut self.data, inputs, ret, address, gas, out);
            }
        }
        let ret = self.create_inner(inputs, shared_memory);
        if INSPECT {
            self.inspector.create_end(
                &mut self.data,
                inputs,
                ret.result,
                ret.created_address,
                ret.gas,
                ret.return_value,
            )
        } else {
            (ret.result, ret.created_address, ret.gas, ret.return_value)
        }
    }

    fn call(
        &mut self,
        inputs: &mut CallInputs,
        shared_memory: &mut SharedMemory,
    ) -> (InstructionResult, Gas, Bytes) {
        if INSPECT {
            let (ret, gas, out) = self.inspector.call(&mut self.data, inputs);
            if ret != InstructionResult::Continue {
                return self
                    .inspector
                    .call_end(&mut self.data, inputs, gas, ret, out);
            }
        }
        let ret = self.call_inner(inputs, shared_memory);
        if INSPECT {
            self.inspector.call_end(
                &mut self.data,
                inputs,
                ret.gas,
                ret.result,
                ret.return_value,
            )
        } else {
            (ret.result, ret.gas, ret.return_value)
        }
    }
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
        let mut journal = JournaledState::new(0, SpecId::BERLIN);
        journal
            .initial_account_load(caller, &[U256::from(100)], &mut db)
            .unwrap();
        assert!(
            EVMImpl::<BedrockSpec, InMemoryDB, false>::commit_mint_value(
                caller,
                mint_value,
                &mut db,
                &mut journal
            )
            .is_ok(),
        );

        // Check the account balance is updated.
        let (account, _) = journal.load_account(caller, &mut db).unwrap();
        assert_eq!(account.info.balance, U256::from(101));

        // No mint value should be a no-op.
        assert!(
            EVMImpl::<BedrockSpec, InMemoryDB, false>::commit_mint_value(
                caller,
                None,
                &mut db,
                &mut journal
            )
            .is_ok(),
        );
        let (account, _) = journal.load_account(caller, &mut db).unwrap();
        assert_eq!(account.info.balance, U256::from(101));
    }

    #[test]
    fn test_remove_l1_cost_non_deposit() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        let mut journal = JournaledState::new(0, SpecId::BERLIN);
        let slots = &[U256::from(100)];
        journal
            .initial_account_load(caller, slots, &mut db)
            .unwrap();
        assert!(EVMImpl::<BedrockSpec, InMemoryDB, false>::remove_l1_cost(
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
        let mut journal = JournaledState::new(0, SpecId::BERLIN);
        journal
            .initial_account_load(caller, &[U256::from(100)], &mut db)
            .unwrap();
        assert!(EVMImpl::<BedrockSpec, InMemoryDB, false>::remove_l1_cost(
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
        let mut journal = JournaledState::new(0, SpecId::BERLIN);
        journal
            .initial_account_load(caller, &[U256::from(100)], &mut db)
            .unwrap();
        assert_eq!(
            EVMImpl::<BedrockSpec, InMemoryDB, false>::remove_l1_cost(
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
