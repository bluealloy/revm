use revm_interpreter::CallValue;
use revm_precompile::PrecompileErrors;

use super::inner_evm_context::InnerEvmContext;
use crate::{
    db::Database,
    interpreter::{
        analysis::validate_eof, return_ok, CallInputs, Contract, CreateInputs, EOFCreateInputs,
        EOFCreateKind, Gas, InstructionResult, Interpreter, InterpreterResult,
    },
    primitives::{
        keccak256, Address, Bytecode, Bytes, CreateScheme, EVMError, Env, Eof,
        SpecId::{self, *},
        B256, EOF_MAGIC_BYTES, U256,
    },
    ContextPrecompiles, FrameOrResult, CALL_STACK_LIMIT,
};
use core::{
    fmt,
    ops::{Deref, DerefMut},
};
use std::{boxed::Box, sync::Arc};

/// EVM context that contains the inner EVM context and precompiles.
pub struct EvmContext<DB: Database> {
    /// Inner EVM context.
    pub inner: InnerEvmContext<DB>,
    /// Precompiles that are available for evm.
    pub precompiles: ContextPrecompiles<DB>,
}

impl<DB: Database + Clone> Clone for EvmContext<DB>
where
    DB::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            precompiles: ContextPrecompiles::default(),
        }
    }
}

impl<DB> fmt::Debug for EvmContext<DB>
where
    DB: Database + fmt::Debug,
    DB::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvmContext")
            .field("inner", &self.inner)
            .field("precompiles", &self.inner)
            .finish_non_exhaustive()
    }
}

impl<DB: Database> Deref for EvmContext<DB> {
    type Target = InnerEvmContext<DB>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<DB: Database> DerefMut for EvmContext<DB> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<DB: Database> EvmContext<DB> {
    /// Create new context with database.
    pub fn new(db: DB) -> Self {
        Self {
            inner: InnerEvmContext::new(db),
            precompiles: ContextPrecompiles::default(),
        }
    }

    /// Creates a new context with the given environment and database.
    #[inline]
    pub fn new_with_env(db: DB, env: Box<Env>) -> Self {
        Self {
            inner: InnerEvmContext::new_with_env(db, env),
            precompiles: ContextPrecompiles::default(),
        }
    }

    /// Sets the database.
    ///
    /// Note that this will ignore the previous `error` if set.
    #[inline]
    pub fn with_db<ODB: Database>(self, db: ODB) -> EvmContext<ODB> {
        EvmContext {
            inner: self.inner.with_db(db),
            precompiles: ContextPrecompiles::default(),
        }
    }

    /// Sets precompiles
    #[inline]
    pub fn set_precompiles(&mut self, precompiles: ContextPrecompiles<DB>) {
        // set warm loaded addresses.
        self.journaled_state
            .warm_preloaded_addresses
            .extend(precompiles.addresses_set());
        self.precompiles = precompiles;
    }

    /// Call precompile contract
    #[inline]
    fn call_precompile(
        &mut self,
        address: &Address,
        input_data: &Bytes,
        gas: Gas,
    ) -> Result<Option<InterpreterResult>, EVMError<DB::Error>> {
        let Some(outcome) =
            self.precompiles
                .call(address, input_data, gas.limit(), &mut self.inner)
        else {
            return Ok(None);
        };

        let mut result = InterpreterResult {
            result: InstructionResult::Return,
            gas,
            output: Bytes::new(),
        };

        match outcome {
            Ok(output) => {
                if result.gas.record_cost(output.gas_used) {
                    result.result = InstructionResult::Return;
                    result.output = output.bytes;
                } else {
                    result.result = InstructionResult::PrecompileOOG;
                }
            }
            Err(PrecompileErrors::Error(e)) => {
                result.result = if e.is_oog() {
                    InstructionResult::PrecompileOOG
                } else {
                    InstructionResult::PrecompileError
                };
            }
            Err(PrecompileErrors::Fatal { msg }) => return Err(EVMError::Precompile(msg)),
        }
        Ok(Some(result))
    }

    /// Make call frame
    #[inline]
    pub fn make_call_frame(
        &mut self,
        inputs: &CallInputs,
    ) -> Result<FrameOrResult, EVMError<DB::Error>> {
        let gas = Gas::new(inputs.gas_limit);

        let return_result = |instruction_result: InstructionResult| {
            Ok(FrameOrResult::new_call_result(
                InterpreterResult {
                    result: instruction_result,
                    gas,
                    output: Bytes::new(),
                },
                inputs.return_memory_offset.clone(),
            ))
        };

        // Check depth
        if self.journaled_state.depth() > CALL_STACK_LIMIT {
            return return_result(InstructionResult::CallTooDeep);
        }

        // Make account warm and loaded
        let _ = self
            .inner
            .journaled_state
            .load_account(inputs.bytecode_address, &mut self.inner.db)?;

        // Create subroutine checkpoint
        let checkpoint = self.journaled_state.checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        match inputs.value {
            // if transfer value is zero, load account and force the touch.
            CallValue::Transfer(value) if value == U256::ZERO => {
                self.load_account(inputs.target_address)?;
                self.journaled_state.touch(&inputs.target_address);
            }
            CallValue::Transfer(value) => {
                // Transfer value from caller to called account. As value get transferred
                // target gets touched.
                if let Some(result) = self.inner.journaled_state.transfer(
                    &inputs.caller,
                    &inputs.target_address,
                    value,
                    &mut self.inner.db,
                )? {
                    self.journaled_state.checkpoint_revert(checkpoint);
                    return return_result(result);
                }
            }
            _ => {}
        };

        if let Some(result) = self.call_precompile(&inputs.bytecode_address, &inputs.input, gas)? {
            if matches!(result.result, return_ok!()) {
                self.journaled_state.checkpoint_commit();
            } else {
                self.journaled_state.checkpoint_revert(checkpoint);
            }
            Ok(FrameOrResult::new_call_result(
                result,
                inputs.return_memory_offset.clone(),
            ))
        } else {
            let (account, _) = self
                .inner
                .journaled_state
                .load_code(inputs.bytecode_address, &mut self.inner.db)?;

            let code_hash = account.info.code_hash();
            let bytecode = account.info.code.clone().unwrap_or_default();

            // ExtDelegateCall is not allowed to call non-EOF contracts.
            if inputs.scheme.is_ext_delegate_call()
                && !bytecode.bytes_slice().starts_with(&EOF_MAGIC_BYTES)
            {
                return return_result(InstructionResult::InvalidExtDelegateCallTarget);
            }

            if bytecode.is_empty() {
                self.journaled_state.checkpoint_commit();
                return return_result(InstructionResult::Stop);
            }

            let contract =
                Contract::new_with_context(inputs.input.clone(), bytecode, Some(code_hash), inputs);
            // Create interpreter and executes call and push new CallStackFrame.
            Ok(FrameOrResult::new_call_frame(
                inputs.return_memory_offset.clone(),
                checkpoint,
                Interpreter::new(contract, gas.limit(), inputs.is_static),
            ))
        }
    }

    /// Make create frame.
    #[inline]
    pub fn make_create_frame(
        &mut self,
        spec_id: SpecId,
        inputs: &CreateInputs,
    ) -> Result<FrameOrResult, EVMError<DB::Error>> {
        let return_error = |e| {
            Ok(FrameOrResult::new_create_result(
                InterpreterResult {
                    result: e,
                    gas: Gas::new(inputs.gas_limit),
                    output: Bytes::new(),
                },
                None,
            ))
        };

        // Check depth
        if self.journaled_state.depth() > CALL_STACK_LIMIT {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Prague EOF
        if spec_id.is_enabled_in(PRAGUE_EOF) && inputs.init_code.starts_with(&EOF_MAGIC_BYTES) {
            return return_error(InstructionResult::CreateInitCodeStartingEF00);
        }

        // Fetch balance of caller.
        let (caller_balance, _) = self.balance(inputs.caller)?;

        // Check if caller has enough balance to send to the created contract.
        if caller_balance < inputs.value {
            return return_error(InstructionResult::OutOfFunds);
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce;
        if let Some(nonce) = self.journaled_state.inc_nonce(inputs.caller) {
            old_nonce = nonce - 1;
        } else {
            return return_error(InstructionResult::Return);
        }

        // Create address
        let mut init_code_hash = B256::ZERO;
        let created_address = match inputs.scheme {
            CreateScheme::Create => inputs.caller.create(old_nonce),
            CreateScheme::Create2 { salt } => {
                init_code_hash = keccak256(&inputs.init_code);
                inputs.caller.create2(salt.to_be_bytes(), init_code_hash)
            }
        };

        // created address is not allowed to be a precompile.
        if self.precompiles.contains(&created_address) {
            return return_error(InstructionResult::CreateCollision);
        }

        // warm load account.
        self.load_account(created_address)?;

        // create account, transfer funds and make the journal checkpoint.
        let checkpoint = match self.journaled_state.create_account_checkpoint(
            inputs.caller,
            created_address,
            inputs.value,
            spec_id,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(e) => {
                return return_error(e);
            }
        };

        let bytecode = Bytecode::new_legacy(inputs.init_code.clone());

        let contract = Contract::new(
            Bytes::new(),
            bytecode,
            Some(init_code_hash),
            created_address,
            None,
            inputs.caller,
            inputs.value,
        );

        Ok(FrameOrResult::new_create_frame(
            created_address,
            checkpoint,
            Interpreter::new(contract, inputs.gas_limit, false),
        ))
    }

    /// Make create frame.
    #[inline]
    pub fn make_eofcreate_frame(
        &mut self,
        spec_id: SpecId,
        inputs: &EOFCreateInputs,
    ) -> Result<FrameOrResult, EVMError<DB::Error>> {
        let return_error = |e| {
            Ok(FrameOrResult::new_eofcreate_result(
                InterpreterResult {
                    result: e,
                    gas: Gas::new(inputs.gas_limit),
                    output: Bytes::new(),
                },
                None,
            ))
        };

        let (input, initcode, created_address) = match &inputs.kind {
            EOFCreateKind::Opcode {
                initcode,
                input,
                created_address,
            } => (input.clone(), initcode.clone(), Some(*created_address)),
            EOFCreateKind::Tx { initdata } => {
                // decode eof and init code.
                let Ok((eof, input)) = Eof::decode_dangling(initdata.clone()) else {
                    return return_error(InstructionResult::InvalidEOFInitCode);
                };

                if validate_eof(&eof).is_err() {
                    // TODO (EOF) new error type.
                    return return_error(InstructionResult::InvalidEOFInitCode);
                }

                // Use nonce from tx (if set) to calculate address.
                // If not set, use the nonce from the account.
                let nonce = self
                    .env
                    .tx
                    .nonce
                    .map(|nonce| self.env.tx.caller.create(nonce));

                (input, eof, nonce)
            }
        };

        // Check depth
        if self.journaled_state.depth() > CALL_STACK_LIMIT {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Fetch balance of caller.
        let (caller_balance, _) = self.balance(inputs.caller)?;

        // Check if caller has enough balance to send to the created contract.
        if caller_balance < inputs.value {
            return return_error(InstructionResult::OutOfFunds);
        }

        // Increase nonce of caller and check if it overflows
        let Some(nonce) = self.journaled_state.inc_nonce(inputs.caller) else {
            // can't happen on mainnet.
            return return_error(InstructionResult::Return);
        };
        let old_nonce = nonce - 1;

        let created_address = created_address.unwrap_or_else(|| inputs.caller.create(old_nonce));

        // created address is not allowed to be a precompile.
        if self.precompiles.contains(&created_address) {
            return return_error(InstructionResult::CreateCollision);
        }

        // Load account so it needs to be marked as warm for access list.
        self.load_account(created_address)?;

        // create account, transfer funds and make the journal checkpoint.
        let checkpoint = match self.journaled_state.create_account_checkpoint(
            inputs.caller,
            created_address,
            inputs.value,
            spec_id,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(e) => {
                return return_error(e);
            }
        };

        let contract = Contract::new(
            input.clone(),
            // fine to clone as it is Bytes.
            Bytecode::Eof(Arc::new(initcode.clone())),
            None,
            created_address,
            None,
            inputs.caller,
            inputs.value,
        );

        let mut interpreter = Interpreter::new(contract, inputs.gas_limit, false);
        // EOF init will enable RETURNCONTRACT opcode.
        interpreter.set_is_eof_init();

        Ok(FrameOrResult::new_eofcreate_frame(
            created_address,
            checkpoint,
            interpreter,
        ))
    }
}

/// Test utilities for the [`EvmContext`].
#[cfg(any(test, feature = "test-utils"))]
pub(crate) mod test_utils {
    use super::*;
    use crate::{
        db::{CacheDB, EmptyDB},
        journaled_state::JournaledState,
        primitives::{address, HashSet, SpecId, B256},
    };

    /// Mock caller address.
    pub const MOCK_CALLER: Address = address!("0000000000000000000000000000000000000000");

    /// Creates `CallInputs` that calls a provided contract address from the mock caller.
    pub fn create_mock_call_inputs(to: Address) -> CallInputs {
        CallInputs {
            input: Bytes::new(),
            gas_limit: 0,
            bytecode_address: to,
            target_address: to,
            caller: MOCK_CALLER,
            value: CallValue::Transfer(U256::ZERO),
            scheme: revm_interpreter::CallScheme::Call,
            is_eof: false,
            is_static: false,
            return_memory_offset: 0..0,
        }
    }

    /// Creates an evm context with a cache db backend.
    /// Additionally loads the mock caller account into the db,
    /// and sets the balance to the provided U256 value.
    pub fn create_cache_db_evm_context_with_balance(
        env: Box<Env>,
        mut db: CacheDB<EmptyDB>,
        balance: U256,
    ) -> EvmContext<CacheDB<EmptyDB>> {
        db.insert_account_info(
            test_utils::MOCK_CALLER,
            crate::primitives::AccountInfo {
                nonce: 0,
                balance,
                code_hash: B256::default(),
                code: None,
            },
        );
        create_cache_db_evm_context(env, db)
    }

    /// Creates a cached db evm context.
    pub fn create_cache_db_evm_context(
        env: Box<Env>,
        db: CacheDB<EmptyDB>,
    ) -> EvmContext<CacheDB<EmptyDB>> {
        EvmContext {
            inner: InnerEvmContext {
                env,
                journaled_state: JournaledState::new(SpecId::CANCUN, HashSet::new()),
                db,
                error: Ok(()),
                valid_authorizations: Vec::new(),
                #[cfg(feature = "optimism")]
                l1_block_info: None,
            },
            precompiles: ContextPrecompiles::default(),
        }
    }

    /// Returns a new `EvmContext` with an empty journaled state.
    pub fn create_empty_evm_context(env: Box<Env>, db: EmptyDB) -> EvmContext<EmptyDB> {
        EvmContext {
            inner: InnerEvmContext {
                env,
                journaled_state: JournaledState::new(SpecId::CANCUN, HashSet::new()),
                db,
                error: Ok(()),
                valid_authorizations: Default::default(),
                #[cfg(feature = "optimism")]
                l1_block_info: None,
            },
            precompiles: ContextPrecompiles::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{CacheDB, EmptyDB},
        primitives::{address, Bytecode},
        Frame, JournalEntry,
    };
    use std::boxed::Box;
    use test_utils::*;

    // Tests that the `EVMContext::make_call_frame` function returns an error if the
    // call stack is too deep.
    #[test]
    fn test_make_call_frame_stack_too_deep() {
        let env = Env::default();
        let db = EmptyDB::default();
        let mut context = test_utils::create_empty_evm_context(Box::new(env), db);
        context.journaled_state.depth = CALL_STACK_LIMIT as usize + 1;
        let contract = address!("dead10000000000000000000000000000001dead");
        let call_inputs = test_utils::create_mock_call_inputs(contract);
        let res = context.make_call_frame(&call_inputs);
        let Ok(FrameOrResult::Result(err)) = res else {
            panic!("Expected FrameOrResult::Result");
        };
        assert_eq!(
            err.interpreter_result().result,
            InstructionResult::CallTooDeep
        );
    }

    // Tests that the `EVMContext::make_call_frame` function returns an error if the
    // transfer fails on the journaled state. It also verifies that the revert was
    // checkpointed on the journaled state correctly.
    #[test]
    fn test_make_call_frame_transfer_revert() {
        let env = Env::default();
        let db = EmptyDB::default();
        let mut evm_context = test_utils::create_empty_evm_context(Box::new(env), db);
        let contract = address!("dead10000000000000000000000000000001dead");
        let mut call_inputs = test_utils::create_mock_call_inputs(contract);
        call_inputs.value = CallValue::Transfer(U256::from(1));
        let res = evm_context.make_call_frame(&call_inputs);
        let Ok(FrameOrResult::Result(result)) = res else {
            panic!("Expected FrameOrResult::Result");
        };
        assert_eq!(
            result.interpreter_result().result,
            InstructionResult::OutOfFunds
        );
        let checkpointed = vec![vec![JournalEntry::AccountWarmed { address: contract }]];
        assert_eq!(evm_context.journaled_state.journal, checkpointed);
        assert_eq!(evm_context.journaled_state.depth, 0);
    }

    #[test]
    fn test_make_call_frame_missing_code_context() {
        let env = Env::default();
        let cdb = CacheDB::new(EmptyDB::default());
        let bal = U256::from(3_000_000_000_u128);
        let mut context = create_cache_db_evm_context_with_balance(Box::new(env), cdb, bal);
        let contract = address!("dead10000000000000000000000000000001dead");
        let call_inputs = test_utils::create_mock_call_inputs(contract);
        let res = context.make_call_frame(&call_inputs);
        let Ok(FrameOrResult::Result(result)) = res else {
            panic!("Expected FrameOrResult::Result");
        };
        assert_eq!(result.interpreter_result().result, InstructionResult::Stop);
    }

    #[test]
    fn test_make_call_frame_succeeds() {
        let env = Env::default();
        let mut cdb = CacheDB::new(EmptyDB::default());
        let bal = U256::from(3_000_000_000_u128);
        let by = Bytecode::new_raw(Bytes::from(vec![0x60, 0x00, 0x60, 0x00]));
        let contract = address!("dead10000000000000000000000000000001dead");
        cdb.insert_account_info(
            contract,
            crate::primitives::AccountInfo {
                nonce: 0,
                balance: bal,
                code_hash: by.clone().hash_slow(),
                code: Some(by),
            },
        );
        let mut evm_context = create_cache_db_evm_context_with_balance(Box::new(env), cdb, bal);
        let call_inputs = test_utils::create_mock_call_inputs(contract);
        let res = evm_context.make_call_frame(&call_inputs);
        let Ok(FrameOrResult::Frame(Frame::Call(call_frame))) = res else {
            panic!("Expected FrameOrResult::Frame(Frame::Call(..))");
        };
        assert_eq!(call_frame.return_memory_range, 0..0,);
    }
}
