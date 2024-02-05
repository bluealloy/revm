use crate::{
    db::Database,
    gas::Gas,
    handler::Handler,
    journaled_state::{JournalCheckpoint, JournaledState},
    primitives::{
        keccak256,
        Address,
        Bytecode,
        Bytes,
        EVMError,
        EVMResult,
        Env,
        Output,
        Spec,
        SpecId::*,
        TransactTo,
        U256,
    },
    EVMData,
};
use core::marker::PhantomData;
use fluentbase_sdk::{LowLevelAPI, LowLevelSDK};
use fluentbase_types::{AccountDb, ExitCode, STATE_DEPLOY, STATE_MAIN};
use revm_primitives::{CreateScheme, B256, RWASM_MAX_CODE_SIZE};
use rwasm_codegen::{Compiler, CompilerConfig, CompilerError, FuncOrExport};

/// EVM call stack limit.
pub const CALL_STACK_LIMIT: u64 = 1024;

pub struct EVMImpl<'a, GSPEC: Spec, DB: Database> {
    pub data: EVMData<'a, DB>,
    pub handler: Handler<DB>,
    _pd: PhantomData<GSPEC>,
}

struct CreateInputs {
    caller: Address,
    value: U256,
    init_code: Bytes,
    salt: Option<U256>,
    gas_limit: u64,
}

struct CallInputsTransfer {
    source: Address,
    target: Address,
    value: U256,
}

pub enum CallScheme {
    /// `CALL`
    Call,
    /// `CALLCODE`
    CallCode,
    /// `DELEGATECALL`
    DelegateCall,
    /// `STATICCALL`
    StaticCall,
}

struct CallInputsContext {
    caller: Address,
    address: Address,
    code_address: Address,
    apparent_value: U256,
    scheme: CallScheme,
}

struct CallInputs {
    contract: Address,
    gas_limit: u64,
    transfer: CallInputsTransfer,
    input: Bytes,
    context: CallInputsContext,
    is_static: bool,
}

struct PreparedCreate {
    created_address: Address,
    gas: Gas,
    checkpoint: JournalCheckpoint,
    bytecode: Bytes,
    caller: Address,
    value: U256,
}

struct CallCreateResult {
    result: ExitCode,
    created_address: Option<Address>,
    gas: Gas,
    return_value: Bytes,
}

impl CallCreateResult {
    fn from_error(result: ExitCode, gas: Gas) -> Self {
        Self {
            result,
            created_address: None,
            gas,
            return_value: Bytes::new(),
        }
    }
}

struct PreparedCall {
    gas: Gas,
    checkpoint: JournalCheckpoint,
    bytecode: Bytes,
    code_hash: B256,
    input: Bytes,
}

/// EVM transaction interface.
#[auto_impl::auto_impl(&mut, Box)]
pub trait Transact<DBError> {
    /// Run checks that could make transaction fail before call/create.
    fn preverify_transaction(&mut self) -> Result<(), EVMError<DBError>>;

    /// Skip pre-verification steps and execute the transaction.
    fn transact_preverified(&mut self) -> EVMResult<DBError>;

    /// Execute transaction by running pre-verification steps and then transaction itself.
    fn transact(&mut self) -> EVMResult<DBError>;
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

impl<'a, GSPEC: Spec + 'static, DB: Database> EVMImpl<'a, GSPEC, DB> {
    pub fn new(db: &'a mut DB, env: &'a mut Env) -> Self {
        let journaled_state = JournaledState::new(GSPEC::SPEC_ID);
        Self {
            data: EVMData {
                env,
                journaled_state,
                db,
                error: None,
            },
            handler: Handler::mainnet::<GSPEC>(),
            _pd: PhantomData {},
        }
    }

    /// Pre verify transaction.
    pub fn preverify_transaction_inner(&mut self) -> Result<(), EVMError<DB::Error>> {
        let env = self.data.env();

        // Important: validate block before tx.
        env.validate_block_env::<GSPEC>()?;
        env.validate_tx::<GSPEC>()?;

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

        let (caller_account, _) = journal
            .load_account(tx_caller, self.data.db)
            .map_err(EVMError::Database)?;

        // Subtract gas costs from the caller's account.
        // We need to saturate the gas cost to prevent underflow in case that
        // `disable_balance_check` is enabled.
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

        let transact_gas_limit = tx_gas_limit;

        // call inner handling of call/create
        let (call_result, ret_gas, output) = match self.data.env.tx.transact_to {
            TransactTo::Call(address) => {
                // Nonce is already checked
                caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);

                let result = self.call_inner(CallInputs {
                    contract: address,
                    transfer: CallInputsTransfer {
                        source: tx_caller,
                        target: address,
                        value: tx_value,
                    },
                    input: tx_data,
                    gas_limit: transact_gas_limit,
                    context: CallInputsContext {
                        caller: tx_caller,
                        address,
                        code_address: address,
                        apparent_value: tx_value,
                        scheme: CallScheme::Call,
                    },
                    is_static: false,
                });
                (result.result, result.gas, Output::Call(result.return_value))
            }
            TransactTo::Create(scheme) => {
                let salt = match scheme {
                    CreateScheme::Create2 { salt } => Some(salt),
                    CreateScheme::Create => None,
                };
                let result = self.create_inner(&mut CreateInputs {
                    caller: tx_caller,
                    value: tx_value,
                    init_code: tx_data,
                    salt,
                    gas_limit: transact_gas_limit,
                });
                (
                    result.result,
                    result.gas,
                    Output::Create(result.return_value, result.created_address),
                )
            }
        };

        let handler = &self.handler;
        let data = &mut self.data;

        // handle output of call/create calls.
        let mut gas = handler.call_return(data.env, call_result, ret_gas);

        // set refund. Refund amount depends on hardfork.
        gas.set_refund(handler.calculate_gas_refund(data.env, &gas) as i64);

        // Reimburse the caller
        handler.reimburse_caller(data, &gas)?;

        // Reward beneficiary
        if !data.env.cfg.is_beneficiary_reward_disabled() {
            handler.reward_beneficiary(data, &gas)?;
        }

        // main return
        handler.main_return(data, call_result, output, &gas)
    }

    #[inline(never)]
    fn prepare_create(
        &mut self,
        inputs: &CreateInputs,
    ) -> Result<PreparedCreate, CallCreateResult> {
        let gas = Gas::new(inputs.gas_limit);
        // Check depth of calls
        if self.data.journaled_state.depth() > CALL_STACK_LIMIT {
            return Err(CallCreateResult::from_error(
                ExitCode::CallDepthOverflow,
                gas,
            ));
        }

        // Fetch balance of caller.
        let Some((caller_balance, _)) = self.data.balance(inputs.caller) else {
            return Err(CallCreateResult::from_error(
                ExitCode::FatalExternalError,
                gas,
            ));
        };

        // Check if caller has enough balance to send to the created contract.
        if caller_balance < inputs.value {
            return Err(CallCreateResult::from_error(
                ExitCode::InsufficientBalance,
                gas,
            ));
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce = self
            .data
            .journaled_state
            .inc_nonce(inputs.caller)
            .ok_or_else(|| CallCreateResult::from_error(ExitCode::FatalExternalError, gas))
            .map(|v| v - 1)?;

        // Create address
        let code_hash = keccak256(&inputs.init_code);
        let created_address = match inputs.salt {
            Some(salt) => inputs.caller.create2(salt.to_be_bytes(), &code_hash),
            None => inputs.caller.create(old_nonce),
        };

        // Load account so it needs to be marked as warm for access list.
        if self
            .data
            .journaled_state
            .load_account(created_address, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .is_err()
        {
            return Err(CallCreateResult::from_error(
                ExitCode::FatalExternalError,
                gas,
            ));
        }

        // create account, transfer funds and make the journal checkpoint.
        let checkpoint = match self
            .data
            .journaled_state
            .create_account_checkpoint::<GSPEC>(inputs.caller, created_address, inputs.value)
        {
            Ok(checkpoint) => checkpoint,
            Err(e) => {
                return Err(CallCreateResult::from_error(e, gas));
            }
        };

        let bytecode =
            Self::translate_wasm_to_rwasm(&inputs.init_code, "deploy").map_err(|_| {
                return CallCreateResult::from_error(ExitCode::CompilationError, gas);
            })?;

        Ok(PreparedCreate {
            created_address,
            gas,
            checkpoint,
            bytecode,
            caller: inputs.caller,
            value: inputs.value,
        })
    }

    fn translate_wasm_to_rwasm(
        input: &Bytes,
        func_name: &'static str,
    ) -> Result<Bytes, CompilerError> {
        use fluentbase_runtime::Runtime;
        let import_linker = Runtime::<()>::new_shared_linker();
        let mut compiler = Compiler::new_with_linker(
            input.as_ref(),
            CompilerConfig::default(),
            Some(&import_linker),
        )?;
        compiler.translate(FuncOrExport::Export(func_name))?;
        let output = compiler.finalize()?;
        Ok(Bytes::from(output))
    }

    /// EVM create opcode for both initial crate and CREATE and CREATE2 opcodes.
    fn create_inner(&mut self, inputs: &CreateInputs) -> CallCreateResult {
        // Prepare crate.
        let prepared_create = match self.prepare_create(inputs) {
            Ok(o) => o,
            Err(e) => return e,
        };
        // Create new interpreter and execute init code
        let (exit_reason, bytes, gas) = self.run_interpreter(
            &prepared_create.bytecode,
            &Bytes::new(),
            STATE_DEPLOY,
            prepared_create.gas,
        );

        if exit_reason != ExitCode::Ok {
            self.data
                .journaled_state
                .checkpoint_revert(prepared_create.checkpoint);
            return CallCreateResult {
                result: exit_reason,
                created_address: Some(prepared_create.created_address),
                gas,
                return_value: bytes,
            };
        }
        if bytes.len()
            > self
                .data
                .env
                .cfg
                .limit_contract_code_size
                .unwrap_or(RWASM_MAX_CODE_SIZE)
        {
            self.data
                .journaled_state
                .checkpoint_revert(prepared_create.checkpoint);
            return CallCreateResult {
                result: ExitCode::ContractSizeLimit,
                created_address: Some(prepared_create.created_address),
                gas,
                return_value: bytes,
            };
        }

        // if we have enough gas
        self.data.journaled_state.checkpoint_commit();
        self.data.journaled_state.set_code(
            prepared_create.created_address,
            Bytecode::new_raw(bytes.clone()),
        );
        CallCreateResult {
            result: ExitCode::Ok,
            created_address: Some(prepared_create.created_address),
            gas,
            return_value: bytes,
        }
    }

    pub fn run_interpreter(
        &mut self,
        bytecode: &Bytes,
        input: &Bytes,
        _state: u32,
        fuel: Gas,
    ) -> (ExitCode, Bytes, Gas) {
        let err_code = LowLevelSDK::sys_exec(
            bytecode.as_ptr(),
            bytecode.len() as u32,
            input.as_ptr(),
            input.len() as u32,
            core::ptr::null_mut(),
            0,
            fuel.remaining() as u32,
        );
        let output_size = LowLevelSDK::sys_output_size();
        let mut output_buffer = vec![0u8; output_size as usize];
        LowLevelSDK::sys_read_output(output_buffer.as_mut_ptr(), 0, output_size);
        (err_code.into(), output_buffer.into(), fuel)
    }

    fn prepare_call(&mut self, inputs: CallInputs) -> Result<PreparedCall, CallCreateResult> {
        let gas = Gas::new(inputs.gas_limit);
        let account = match self
            .data
            .journaled_state
            .load_code(inputs.contract, self.data.db)
        {
            Ok((account, _)) => account,
            Err(e) => {
                self.data.error = Some(e);
                return Err(CallCreateResult::from_error(
                    ExitCode::FatalExternalError,
                    gas,
                ));
            }
        };
        let code_hash = account.info.code_hash();
        let bytecode = account.info.code.clone().unwrap_or_default();

        // Check depth
        if self.data.journaled_state.depth() > CALL_STACK_LIMIT {
            return Err(CallCreateResult::from_error(
                ExitCode::CallDepthOverflow,
                gas,
            ));
        }

        // Create subroutine checkpoint
        let checkpoint = self.data.journaled_state.checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if inputs.transfer.value == U256::ZERO {
            self.data.load_account(inputs.context.address);
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
            return Err(CallCreateResult {
                result: e,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            });
        }

        Ok(PreparedCall {
            gas,
            checkpoint,
            bytecode: bytecode.original_bytes(),
            code_hash,
            input: inputs.input,
        })
    }

    /// Main contract call of the EVM.
    fn call_inner(&mut self, inputs: CallInputs) -> CallCreateResult {
        // Prepare call
        let prepared_call = match self.prepare_call(inputs) {
            Ok(o) => o,
            Err(e) => return e,
        };

        let ret = if !prepared_call.bytecode.is_empty() {
            let (exit_reason, bytes, gas) = self.run_interpreter(
                &prepared_call.bytecode,
                &prepared_call.input,
                STATE_MAIN,
                prepared_call.gas,
            );
            CallCreateResult {
                result: exit_reason,
                created_address: None,
                gas,
                return_value: bytes,
            }
        } else {
            CallCreateResult {
                result: ExitCode::Ok,
                created_address: None,
                gas: prepared_call.gas,
                return_value: Bytes::new(),
            }
        };

        // revert changes or not.
        if ret.result == ExitCode::Ok {
            self.data.journaled_state.checkpoint_commit();
        } else {
            self.data
                .journaled_state
                .checkpoint_revert(prepared_call.checkpoint);
        }

        ret
    }
}
