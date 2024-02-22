use crate::{
    gas::Gas,
    handler::Handler,
    primitives::{
        keccak256,
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
    types::CallCreateResult,
    EVMData,
};
use core::marker::PhantomData;
use fluentbase_core::account::Account;
use fluentbase_sdk::{LowLevelAPI, LowLevelSDK};
use fluentbase_types::{ExitCode, STATE_DEPLOY, STATE_MAIN};
use revm_primitives::{CreateScheme, RWASM_MAX_CODE_SIZE};
use rwasm_codegen::{Compiler, CompilerConfig, CompilerError, FuncOrExport};

/// EVM call stack limit.
pub const CALL_STACK_LIMIT: u64 = 1024;

pub struct EVMImpl<'a, GSPEC: Spec> {
    pub data: EVMData<'a>,
    pub handler: Handler,
    depth: u64,
    _pd: PhantomData<GSPEC>,
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

impl<'a, GSPEC: Spec + 'static> Transact<ExitCode> for EVMImpl<'a, GSPEC> {
    #[inline]
    fn preverify_transaction(&mut self) -> Result<(), EVMError<ExitCode>> {
        self.preverify_transaction_inner()
    }

    #[inline]
    fn transact_preverified(&mut self) -> EVMResult<ExitCode> {
        let output = self.transact_preverified_inner();
        self.handler.end(&mut self.data, output)
    }

    #[inline]
    fn transact(&mut self) -> EVMResult<ExitCode> {
        let output = self
            .preverify_transaction_inner()
            .and_then(|()| self.transact_preverified_inner());
        self.handler.end(&mut self.data, output)
    }
}

impl<'a, GSPEC: Spec + 'static> EVMImpl<'a, GSPEC> {
    pub fn new(env: &'a mut Env) -> Self {
        Self {
            data: EVMData { env },
            handler: Handler::mainnet::<GSPEC>(),
            _pd: PhantomData {},
            depth: 0,
        }
    }

    /// Pre verify transaction.
    pub fn preverify_transaction_inner(&mut self) -> Result<(), EVMError<ExitCode>> {
        // Important: validate block before tx.
        self.data.env.validate_block_env::<GSPEC>()?;
        self.data.env.validate_tx::<GSPEC>()?;

        // load acc
        // TODO: "warmup and verify tx caller?"
        // let tx_caller = env.tx.caller;
        // let (caller_account, _) = self
        //     .data
        //     .journaled_state
        //     .load_account(tx_caller, self.data.db)
        //     .map_err(EVMError::Database)?;
        //
        // self.data
        //     .env
        //     .validate_tx_against_state(caller_account)
        //     .map_err(Into::into)
        Ok(())
    }

    /// Transact preverified transaction.
    pub fn transact_preverified_inner(&mut self) -> EVMResult<ExitCode> {
        let env = &self.data.env;
        let tx_caller = env.tx.caller;
        let tx_value = env.tx.value;
        let tx_data = env.tx.data.clone();
        let tx_gas_limit = env.tx.gas_limit;
        let block_coinbase = env.block.coinbase;

        // load coinbase
        // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
        if GSPEC::enabled(SHANGHAI) {
            // TODO: "warmup coinbase"
        }
        // TODO: "warmup access list"

        let mut caller_account = Account::new_from_jzkt(&tx_caller);

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
        caller_account.sub_balance_saturating(gas_cost);

        let transact_gas_limit = tx_gas_limit;

        // call inner handling of call/create
        let (call_result, ret_gas, output) = match self.data.env.tx.transact_to {
            TransactTo::Call(address) => {
                caller_account.inc_nonce();
                let mut callee_account = Account::new_from_jzkt(&address);
                let result = self.call_inner(
                    &mut caller_account,
                    &mut callee_account,
                    tx_value,
                    tx_data,
                    transact_gas_limit,
                );
                (result.result, result.gas, Output::Call(result.return_value))
            }
            TransactTo::Create(scheme) => {
                let salt = match scheme {
                    CreateScheme::Create2 { salt } => Some(salt),
                    CreateScheme::Create => None,
                };
                let result = self.create_inner(
                    &mut caller_account,
                    tx_value,
                    tx_data,
                    transact_gas_limit,
                    salt,
                );
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
        let effective_gas_price = data.env.effective_gas_price();
        caller_account.add_balance_saturating(
            effective_gas_price * U256::from(gas.remaining() + gas.refunded() as u64),
        );

        // Reward beneficiary
        if !data.env.cfg.is_beneficiary_reward_disabled() {
            let mut coinbase_account = Account::new_from_jzkt(&block_coinbase);
            let effective_gas_price = data.env.effective_gas_price();
            // EIP-1559 discard basefee for coinbase transfer. Basefee amount of gas is discarded.
            let coinbase_gas_price = if GSPEC::enabled(LONDON) {
                effective_gas_price.saturating_sub(data.env.block.basefee)
            } else {
                effective_gas_price
            };
            coinbase_account.add_balance_saturating(
                coinbase_gas_price * U256::from(gas.spend() - gas.refunded() as u64),
            );
        }

        // main return
        handler.main_return(data, call_result, output, &gas)
    }

    fn translate_wasm_to_rwasm(
        input: &Bytes,
        func_name: &'static str,
    ) -> Result<Bytes, CompilerError> {
        #[cfg(feature = "runtime")]
        {
            use fluentbase_runtime::Runtime;
            let import_linker = Runtime::<()>::new_shared_linker();
            let mut compiler = Compiler::new_with_linker(
                input.as_ref(),
                CompilerConfig::default(),
                Some(&import_linker),
            )?;
            compiler.translate(FuncOrExport::Export(func_name))?;
            let output = compiler.finalize()?;
            return Ok(Bytes::from(output));
        };
        #[cfg(not(feature = "runtime"))]
        {
            use rwasm_codegen::{ImportLinker, ImportLinkerV1AlphaDefaults};
            let mut import_linker = ImportLinker::default();
            ImportLinkerV1AlphaDefaults::register_import_funcs(&mut import_linker, None);
            let mut compiler = Compiler::new_with_linker(
                input.as_ref(),
                CompilerConfig::default(),
                Some(&import_linker),
            )?;
            compiler.translate(FuncOrExport::Export(func_name))?;
            let output = compiler.finalize()?;
            return Ok(Bytes::from(output));
        };
    }

    /// EVM create opcode for both initial crate and CREATE and CREATE2 opcodes.
    fn create_inner(
        &mut self,
        caller_account: &mut Account,
        value: U256,
        input: Bytes,
        gas_limit: u64,
        salt: Option<U256>,
    ) -> CallCreateResult {
        let gas = Gas::new(gas_limit);
        if self.depth > CALL_STACK_LIMIT {
            return CallCreateResult::from_error(ExitCode::CallDepthOverflow, gas);
        } else if caller_account.balance < value {
            return CallCreateResult::from_error(ExitCode::InsufficientBalance, gas);
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce = caller_account.inc_nonce();

        // Create address
        let code_hash = keccak256(&input);
        let created_address = match salt {
            Some(salt) => caller_account
                .address
                .create2(salt.to_be_bytes(), &code_hash),
            None => caller_account.address.create(old_nonce),
        };
        // TODO: "Load account so it needs to be marked as warm for access list"

        // create account, transfer funds and make the journal checkpoint.
        let mut contract_account = Account::new_from_jzkt(&created_address);
        let checkpoint = match Account::create_account_checkpoint(
            caller_account,
            &mut contract_account,
            value,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(err) => return CallCreateResult::from_error(err, gas),
        };

        let bytecode = match Self::translate_wasm_to_rwasm(&input, "deploy") {
            Ok(bytecode) => bytecode,
            Err(_) => return CallCreateResult::from_error(ExitCode::CompilationError, gas),
        };

        // Create new interpreter and execute init code
        let (exit_reason, bytes, gas) =
            self.run_interpreter(&bytecode, &Bytes::new(), STATE_DEPLOY, gas);

        if exit_reason != ExitCode::Ok {
            Account::rollback(checkpoint);
            return CallCreateResult {
                result: exit_reason,
                created_address: Some(created_address),
                gas,
                return_value: bytes,
            };
        }
        if bytes.len() > RWASM_MAX_CODE_SIZE {
            Account::rollback(checkpoint);
            return CallCreateResult {
                result: ExitCode::ContractSizeLimit,
                created_address: Some(created_address),
                gas,
                return_value: bytes,
            };
        }

        let bytes = match Self::translate_wasm_to_rwasm(&bytes, "main") {
            Err(_) => {
                Account::rollback(checkpoint);
                return CallCreateResult {
                    result: ExitCode::CompilationError,
                    created_address: Some(created_address),
                    gas,
                    return_value: bytes,
                };
            }
            Ok(result) => result,
        };

        contract_account.change_bytecode(&bytes);

        CallCreateResult {
            result: ExitCode::Ok,
            created_address: Some(created_address),
            gas,
            return_value: bytes,
        }
    }

    #[inline(always)]
    pub fn run_interpreter(
        &mut self,
        bytecode: &Bytes,
        input: &Bytes,
        state: u32,
        mut fuel: Gas,
    ) -> (ExitCode, Bytes, Gas) {
        let mut fuel_remaining = fuel.remaining() as u32;
        let err_code = LowLevelSDK::sys_exec(
            bytecode.as_ptr(),
            bytecode.len() as u32,
            input.as_ptr(),
            input.len() as u32,
            core::ptr::null_mut(),
            0,
            &mut fuel_remaining as *mut u32,
            state,
        );
        let output_size = LowLevelSDK::sys_output_size();
        let mut output_buffer = vec![0u8; output_size as usize];
        LowLevelSDK::sys_read_output(output_buffer.as_mut_ptr(), 0, output_size);
        let fuel_spent = fuel.remaining() as u32 - fuel_remaining;
        fuel.record_cost(fuel_spent as u64);
        (err_code.into(), output_buffer.into(), fuel)
    }

    /// Main contract call of the EVM.
    fn call_inner(
        &mut self,
        caller_account: &mut Account,
        callee_account: &mut Account,
        value: U256,
        input: Bytes,
        gas_limit: u64,
    ) -> CallCreateResult {
        let gas = Gas::new(gas_limit);

        // load contract account and it's bytecode
        let bytecode = callee_account.load_bytecode();

        // check call stack limit
        if self.depth > CALL_STACK_LIMIT {
            return CallCreateResult::from_error(ExitCode::CallDepthOverflow, gas);
        }

        // Create subroutine checkpoint
        let checkpoint = Account::checkpoint();

        if let Err(e) = Account::transfer(caller_account, callee_account, value) {
            Account::rollback(checkpoint);
            return CallCreateResult {
                result: e,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            };
        }

        let ret = if !bytecode.is_empty() {
            let (exit_reason, bytes, gas) =
                self.run_interpreter(&bytecode, &input, STATE_MAIN, gas);
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
                gas,
                return_value: Bytes::new(),
            }
        };

        // revert changes or not.
        if ret.result == ExitCode::Ok {
            Account::checkpoint();
        } else {
            Account::rollback(checkpoint);
        }

        ret
    }
}
