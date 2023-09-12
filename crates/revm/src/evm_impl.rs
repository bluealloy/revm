use crate::interpreter::{
    analysis::to_analysed, gas, instruction_result::SuccessOrHalt, return_ok, return_revert,
    CallContext, CallInputs, CallScheme, Contract, CreateInputs, CreateScheme, Gas, Host,
    InstructionResult, Interpreter, SelfDestructResult, Transfer, CALL_STACK_LIMIT,
};
use crate::journaled_state::{is_precompile, JournalCheckpoint};
use crate::primitives::{
    create2_address, create_address, keccak256, Account, AnalysisKind, Bytecode, Bytes, EVMError,
    EVMResult, Env, ExecutionResult, HashMap, InvalidTransaction, Log, Output, ResultAndState,
    Spec,
    SpecId::{self, *},
    TransactTo, B160, B256, U256,
};
use crate::{db::Database, journaled_state::JournaledState, precompile, Inspector};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::{cmp::min, marker::PhantomData};
use revm_interpreter::gas::initial_tx_gas;
use revm_interpreter::MAX_CODE_SIZE;
use revm_precompile::{Precompile, Precompiles};

pub struct EVMData<'a, DB: Database> {
    pub env: &'a mut Env,
    pub journaled_state: JournaledState,
    pub db: &'a mut DB,
    pub error: Option<DB::Error>,
    pub precompiles: Precompiles,
}

pub struct EVMImpl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> {
    data: EVMData<'a, DB>,
    inspector: &'a mut dyn Inspector<DB>,
    _phantomdata: PhantomData<GSPEC>,
}

struct PreparedCreate {
    gas: Gas,
    created_address: B160,
    checkpoint: JournalCheckpoint,
    contract: Box<Contract>,
}

struct CreateResult {
    result: InstructionResult,
    created_address: Option<B160>,
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
    /// Do checks that could make transaction fail before call/create
    fn preverify_transaction(&mut self) -> Result<(), EVMError<DBError>>;

    /// Skip preverification steps and do transaction
    fn transact_preverified(&mut self) -> EVMResult<DBError>;

    /// Do transaction.
    /// InstructionResult InstructionResult, Output for call or Address if we are creating
    /// contract, gas spend, gas refunded, State that needs to be applied.
    fn transact(&mut self) -> EVMResult<DBError>;
}

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> EVMImpl<'a, GSPEC, DB, INSPECT> {
    /// Load access list for berlin hardfork.
    ///
    /// Loading of accounts/storages is needed to make them hot.
    #[inline]
    fn load_access_list(&mut self) -> Result<(), EVMError<DB::Error>> {
        for (address, slots) in self.data.env.tx.access_list.iter() {
            self.data
                .journaled_state
                .initial_account_load(*address, slots, self.data.db)
                .map_err(EVMError::Database)?;
        }
        Ok(())
    }
}

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> Transact<DB::Error>
    for EVMImpl<'a, GSPEC, DB, INSPECT>
{
    fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        let env = self.env();

        env.validate_block_env::<GSPEC, DB::Error>()?;
        env.validate_tx::<GSPEC>()?;

        let tx_caller = env.tx.caller;
        let tx_data = &env.tx.data;
        let tx_is_create = env.tx.transact_to.is_create();

        let initial_gas_spend = initial_tx_gas::<GSPEC>(tx_data, tx_is_create, &env.tx.access_list);

        // Additonal check to see if limit is big enought to cover initial gas.
        if env.tx.gas_limit < initial_gas_spend {
            return Err(InvalidTransaction::CallGasCostMoreThanGasLimit.into());
        }

        // load acc
        let journal = &mut self.data.journaled_state;
        let (caller_account, _) = journal
            .load_account(tx_caller, self.data.db)
            .map_err(EVMError::Database)?;

        self.data.env.validate_tx_against_state(caller_account)?;

        Ok(())
    }

    fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        let env = &self.data.env;
        let tx_caller = env.tx.caller;
        let tx_value = env.tx.value;
        let tx_data = env.tx.data.clone();
        let tx_gas_limit = env.tx.gas_limit;
        let tx_is_create = env.tx.transact_to.is_create();
        let effective_gas_price = env.effective_gas_price();

        let initial_gas_spend =
            initial_tx_gas::<GSPEC>(&tx_data, tx_is_create, &env.tx.access_list);

        // load coinbase
        // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
        if GSPEC::enabled(SHANGHAI) {
            self.data
                .journaled_state
                .initial_account_load(self.data.env.block.coinbase, &[], self.data.db)
                .map_err(EVMError::Database)?;
        }
        self.load_access_list()?;

        // load acc
        let journal = &mut self.data.journaled_state;
        let (caller_account, _) = journal
            .load_account(tx_caller, self.data.db)
            .map_err(EVMError::Database)?;

        // Reduce gas_limit*gas_price amount of caller account.
        // unwrap_or can only occur if disable_balance_check is enabled
        caller_account.info.balance = caller_account
            .info
            .balance
            .checked_sub(U256::from(tx_gas_limit).saturating_mul(effective_gas_price))
            .unwrap_or(U256::ZERO);

        // touch account so we know it is changed.
        caller_account.mark_touch();

        let transact_gas_limit = tx_gas_limit - initial_gas_spend;

        // call inner handling of call/create
        let (exit_reason, ret_gas, output) = match self.data.env.tx.transact_to {
            TransactTo::Call(address) => {
                // Nonce is already checked
                caller_account.info.nonce =
                    caller_account.info.nonce.checked_add(1).unwrap_or(u64::MAX);

                let (exit, gas, bytes) = self.call(&mut CallInputs {
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
                });
                (exit, gas, Output::Call(bytes))
            }
            TransactTo::Create(scheme) => {
                let (exit, address, ret_gas, bytes) = self.create(&mut CreateInputs {
                    caller: tx_caller,
                    scheme,
                    value: tx_value,
                    init_code: tx_data,
                    gas_limit: transact_gas_limit,
                });
                (exit, ret_gas, Output::Create(bytes, address))
            }
        };

        // set gas with gas limit and spend it all. Gas is going to be reimbursed when
        // transaction is returned successfully.
        let mut gas = Gas::new(tx_gas_limit);
        gas.record_cost(tx_gas_limit);

        if crate::USE_GAS {
            match exit_reason {
                return_ok!() => {
                    gas.erase_cost(ret_gas.remaining());
                    gas.record_refund(ret_gas.refunded());
                }
                return_revert!() => {
                    gas.erase_cost(ret_gas.remaining());
                }
                _ => {}
            }
        }

        let (state, logs, gas_used, gas_refunded) = self.finalize::<GSPEC>(&gas);

        let result = match exit_reason.into() {
            SuccessOrHalt::Success(reason) => ExecutionResult::Success {
                reason,
                gas_used,
                gas_refunded,
                logs,
                output,
            },
            SuccessOrHalt::Revert => ExecutionResult::Revert {
                gas_used,
                output: match output {
                    Output::Call(return_value) => return_value,
                    Output::Create(return_value, _) => return_value,
                },
            },
            SuccessOrHalt::Halt(reason) => ExecutionResult::Halt { reason, gas_used },
            SuccessOrHalt::FatalExternalError => {
                return Err(EVMError::Database(self.data.error.take().unwrap()));
            }
            SuccessOrHalt::InternalContinue => {
                panic!("Internal return flags should remain internal {exit_reason:?}")
            }
        };

        Ok(ResultAndState { result, state })
    }

    fn transact(&mut self) -> EVMResult<DB::Error> {
        self.preverify_transaction()
            .and_then(|_| self.transact_preverified())
    }
}

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> EVMImpl<'a, GSPEC, DB, INSPECT> {
    pub fn new(
        db: &'a mut DB,
        env: &'a mut Env,
        inspector: &'a mut dyn Inspector<DB>,
        precompiles: Precompiles,
    ) -> Self {
        let journaled_state = if GSPEC::enabled(SpecId::SPURIOUS_DRAGON) {
            JournaledState::new(precompiles.len())
        } else {
            JournaledState::new_legacy(precompiles.len())
        };
        Self {
            data: EVMData {
                env,
                journaled_state,
                db,
                error: None,
                precompiles,
            },
            inspector,
            _phantomdata: PhantomData {},
        }
    }

    fn finalize<SPEC: Spec>(&mut self, gas: &Gas) -> (HashMap<B160, Account>, Vec<Log>, u64, u64) {
        let caller = self.data.env.tx.caller;
        let coinbase = self.data.env.block.coinbase;
        let (gas_used, gas_refunded) =
            if crate::USE_GAS {
                let effective_gas_price = self.data.env.effective_gas_price();
                let basefee = self.data.env.block.basefee;

                let gas_refunded = if self.env().cfg.is_gas_refund_disabled() {
                    0
                } else {
                    // EIP-3529: Reduction in refunds
                    let max_refund_quotient = if SPEC::enabled(LONDON) { 5 } else { 2 };
                    min(gas.refunded() as u64, gas.spend() / max_refund_quotient)
                };

                // return balance of not spend gas.
                let Ok((caller_account, _)) =
                    self.data.journaled_state.load_account(caller, self.data.db)
                else {
                    panic!("caller account not found");
                };

                caller_account.info.balance = caller_account.info.balance.saturating_add(
                    effective_gas_price * U256::from(gas.remaining() + gas_refunded),
                );

                // transfer fee to coinbase/beneficiary.
                if !self.data.env.cfg.disable_coinbase_tip {
                    // EIP-1559 discard basefee for coinbase transfer. Basefee amount of gas is discarded.
                    let coinbase_gas_price = if SPEC::enabled(LONDON) {
                        effective_gas_price.saturating_sub(basefee)
                    } else {
                        effective_gas_price
                    };

                    let Ok((coinbase_account, _)) = self
                        .data
                        .journaled_state
                        .load_account(coinbase, self.data.db)
                    else {
                        panic!("coinbase account not found");
                    };
                    coinbase_account.mark_touch();
                    coinbase_account.info.balance = coinbase_account.info.balance.saturating_add(
                        coinbase_gas_price * U256::from(gas.spend() - gas_refunded),
                    );
                }

                (gas.spend() - gas_refunded, gas_refunded)
            } else {
                // touch coinbase
                let _ = self
                    .data
                    .journaled_state
                    .load_account(coinbase, self.data.db);
                self.data.journaled_state.touch(&coinbase);
                (0, 0)
            };
        let (new_state, logs) = self.data.journaled_state.finalize();
        (new_state, logs, gas_used, gas_refunded)
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
            CreateScheme::Create => create_address(inputs.caller, old_nonce),
            CreateScheme::Create2 { salt } => create2_address(inputs.caller, code_hash, salt),
        };

        // Load account so it needs to be marked as hot for access list.
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
    fn create_inner(&mut self, inputs: &CreateInputs) -> CreateResult {
        // Prepare crate.
        let prepared_create = match self.prepare_create(inputs) {
            Ok(o) => o,
            Err(e) => return e,
        };

        // Create new interpreter and execute initcode
        let (exit_reason, mut interpreter) =
            self.run_interpreter(prepared_create.contract, prepared_create.gas.limit(), false);

        // Host error if present on execution
        match exit_reason {
            return_ok!() => {
                // if ok, check contract creation limit and calculate gas deduction on output len.
                let mut bytes = interpreter.return_value();

                // EIP-3541: Reject new contract code starting with the 0xEF byte
                if GSPEC::enabled(LONDON) && !bytes.is_empty() && bytes.first() == Some(&0xEF) {
                    self.data
                        .journaled_state
                        .checkpoint_revert(prepared_create.checkpoint);
                    return CreateResult {
                        result: InstructionResult::CreateContractStartingWithEF,
                        created_address: Some(prepared_create.created_address),
                        gas: interpreter.gas,
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
                        gas: interpreter.gas,
                        return_value: bytes,
                    };
                }
                if crate::USE_GAS {
                    let gas_for_code = bytes.len() as u64 * gas::CODEDEPOSIT;
                    if !interpreter.gas.record_cost(gas_for_code) {
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
                                gas: interpreter.gas,
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
                    gas: interpreter.gas,
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
                    gas: interpreter.gas,
                    return_value: interpreter.return_value(),
                }
            }
        }
    }

    /// Create a Interpreter and run it.
    /// Returns the exit reason and created interpreter as it contains return values and gas spend.
    pub fn run_interpreter(
        &mut self,
        contract: Box<Contract>,
        gas_limit: u64,
        is_static: bool,
    ) -> (InstructionResult, Box<Interpreter>) {
        // Create inspector
        #[cfg(feature = "memory_limit")]
        let mut interpreter = Box::new(Interpreter::new_with_memory_limit(
            contract,
            gas_limit,
            is_static,
            self.data.env.cfg.memory_limit,
        ));

        #[cfg(not(feature = "memory_limit"))]
        let mut interpreter = Box::new(Interpreter::new(contract, gas_limit, is_static));

        if INSPECT {
            self.inspector
                .initialize_interp(&mut interpreter, &mut self.data);
        }
        let exit_reason = if INSPECT {
            interpreter.run_inspect::<Self, GSPEC>(self)
        } else {
            interpreter.run::<Self, GSPEC>(self)
        };

        (exit_reason, interpreter)
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
            Precompile::Custom(fun) => fun(input_data, gas.limit()),
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
    fn call_inner(&mut self, inputs: &CallInputs) -> CallResult {
        // Prepare call
        let prepared_call = match self.prepare_call(inputs) {
            Ok(o) => o,
            Err(e) => return e,
        };

        let ret = if is_precompile(inputs.contract, self.data.precompiles.len()) {
            self.call_precompile(inputs, prepared_call.gas)
        } else if !prepared_call.contract.bytecode.is_empty() {
            // Create interpreter and execute subcall
            let (exit_reason, interpreter) = self.run_interpreter(
                prepared_call.contract,
                prepared_call.gas.limit(),
                inputs.is_static,
            );
            CallResult {
                result: exit_reason,
                gas: interpreter.gas,
                return_value: interpreter.return_value(),
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

    fn load_account(&mut self, address: B160) -> Option<(bool, bool)> {
        self.data
            .journaled_state
            .load_account_exist(address, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn balance(&mut self, address: B160) -> Option<(U256, bool)> {
        let db = &mut self.data.db;
        let journal = &mut self.data.journaled_state;
        let error = &mut self.data.error;
        journal
            .load_account(address, db)
            .map_err(|e| *error = Some(e))
            .ok()
            .map(|(acc, is_cold)| (acc.info.balance, is_cold))
    }

    fn code(&mut self, address: B160) -> Option<(Bytecode, bool)> {
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
    fn code_hash(&mut self, address: B160) -> Option<(B256, bool)> {
        let journal = &mut self.data.journaled_state;
        let db = &mut self.data.db;
        let error = &mut self.data.error;

        let (acc, is_cold) = journal
            .load_code(address, db)
            .map_err(|e| *error = Some(e))
            .ok()?;
        if acc.is_empty() {
            return Some((B256::zero(), is_cold));
        }

        Some((acc.info.code_hash, is_cold))
    }

    fn sload(&mut self, address: B160, index: U256) -> Option<(U256, bool)> {
        // account is always hot. reference on that statement https://eips.ethereum.org/EIPS/eip-2929 see `Note 2:`
        self.data
            .journaled_state
            .sload(address, index, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn sstore(
        &mut self,
        address: B160,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        self.data
            .journaled_state
            .sstore(address, index, value, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn tload(&mut self, address: B160, index: U256) -> U256 {
        self.data.journaled_state.tload(address, index)
    }

    fn tstore(&mut self, address: B160, index: U256, value: U256) {
        self.data.journaled_state.tstore(address, index, value)
    }

    fn log(&mut self, address: B160, topics: Vec<B256>, data: Bytes) {
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

    fn selfdestruct(&mut self, address: B160, target: B160) -> Option<SelfDestructResult> {
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
    ) -> (InstructionResult, Option<B160>, Gas, Bytes) {
        // Call inspector
        if INSPECT {
            let (ret, address, gas, out) = self.inspector.create(&mut self.data, inputs);
            if ret != InstructionResult::Continue {
                return self
                    .inspector
                    .create_end(&mut self.data, inputs, ret, address, gas, out);
            }
        }
        let ret = self.create_inner(inputs);
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

    fn call(&mut self, inputs: &mut CallInputs) -> (InstructionResult, Gas, Bytes) {
        if INSPECT {
            let (ret, gas, out) = self.inspector.call(&mut self.data, inputs);
            if ret != InstructionResult::Continue {
                return self
                    .inspector
                    .call_end(&mut self.data, inputs, gas, ret, out);
            }
        }
        let ret = self.call_inner(inputs);
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
