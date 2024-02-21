use crate::{
    db::{Database, EmptyDB},
    interpreter::{
        analysis::to_analysed, gas, return_ok, CallInputs, Contract, CreateInputs, Gas,
        InstructionResult, Interpreter, InterpreterResult, MAX_CODE_SIZE,
    },
    journaled_state::JournaledState,
    precompile::{Precompile, Precompiles},
    primitives::{
        keccak256, Address, AnalysisKind, Bytecode, Bytes, CreateScheme, EVMError, Env, HandlerCfg,
        HashSet, Spec, SpecId, SpecId::*, B256, U256,
    },
    FrameOrResult, JournalCheckpoint, CALL_STACK_LIMIT,
};
use std::boxed::Box;

/// Main Context structure that contains both EvmContext and External context.
pub struct Context<EXT, DB: Database> {
    /// Evm Context.
    pub evm: EvmContext<DB>,
    /// External contexts.
    pub external: EXT,
}

impl<EXT: Clone, DB: Database + Clone> Clone for Context<EXT, DB>
where
    DB::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            evm: self.evm.clone(),
            external: self.external.clone(),
        }
    }
}

impl Default for Context<(), EmptyDB> {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl Context<(), EmptyDB> {
    /// Creates empty context. This is useful for testing.
    pub fn new_empty() -> Context<(), EmptyDB> {
        Context {
            evm: EvmContext::new(EmptyDB::new()),
            external: (),
        }
    }
}

impl<DB: Database> Context<(), DB> {
    /// Creates new context with database.
    pub fn new_with_db(db: DB) -> Context<(), DB> {
        Context {
            evm: EvmContext::new_with_env(db, Box::default()),
            external: (),
        }
    }
}

impl<EXT, DB: Database> Context<EXT, DB> {
    /// Creates new context with external and database.
    pub fn new(evm: EvmContext<DB>, external: EXT) -> Context<EXT, DB> {
        Context { evm, external }
    }
}

/// Context with handler configuration.
pub struct ContextWithHandlerCfg<EXT, DB: Database> {
    /// Context of execution.
    pub context: Context<EXT, DB>,
    /// Handler configuration.
    pub cfg: HandlerCfg,
}

impl<EXT, DB: Database> ContextWithHandlerCfg<EXT, DB> {
    /// Creates new context with handler configuration.
    pub fn new(context: Context<EXT, DB>, cfg: HandlerCfg) -> Self {
        Self { cfg, context }
    }
}

/// EVM contexts contains data that EVM needs for execution.
#[derive(Debug)]
pub struct EvmContext<DB: Database> {
    /// EVM Environment contains all the information about config, block and transaction that
    /// evm needs.
    pub env: Box<Env>,
    /// EVM State with journaling support.
    pub journaled_state: JournaledState,
    /// Database to load data from.
    pub db: DB,
    /// Error that happened during execution.
    pub error: Option<DB::Error>,
    /// Precompiles that are available for evm.
    pub precompiles: Precompiles,
    /// Used as temporary value holder to store L1 block info.
    #[cfg(feature = "optimism")]
    pub l1_block_info: Option<crate::optimism::L1BlockInfo>,
}

impl<DB: Database + Clone> Clone for EvmContext<DB>
where
    DB::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            env: self.env.clone(),
            journaled_state: self.journaled_state.clone(),
            db: self.db.clone(),
            error: self.error.clone(),
            precompiles: self.precompiles.clone(),
            #[cfg(feature = "optimism")]
            l1_block_info: self.l1_block_info.clone(),
        }
    }
}

impl<DB: Database> EvmContext<DB> {
    pub fn with_db<ODB: Database>(self, db: ODB) -> EvmContext<ODB> {
        EvmContext {
            env: self.env,
            journaled_state: self.journaled_state,
            db,
            error: None,
            precompiles: self.precompiles,
            #[cfg(feature = "optimism")]
            l1_block_info: self.l1_block_info,
        }
    }

    pub fn new(db: DB) -> Self {
        Self {
            env: Box::default(),
            journaled_state: JournaledState::new(SpecId::LATEST, HashSet::new()),
            db,
            error: None,
            precompiles: Precompiles::default(),
            #[cfg(feature = "optimism")]
            l1_block_info: None,
        }
    }

    /// New context with database and environment.
    pub fn new_with_env(db: DB, env: Box<Env>) -> Self {
        Self {
            env,
            journaled_state: JournaledState::new(SpecId::LATEST, HashSet::new()),
            db,
            error: None,
            precompiles: Precompiles::default(),
            #[cfg(feature = "optimism")]
            l1_block_info: None,
        }
    }

    /// Returns the configured EVM spec ID.
    pub const fn spec_id(&self) -> SpecId {
        self.journaled_state.spec
    }

    /// Sets precompiles
    #[inline]
    pub fn set_precompiles(&mut self, precompiles: Precompiles) {
        self.journaled_state.warm_preloaded_addresses =
            precompiles.addresses().copied().collect::<HashSet<_>>();
        self.precompiles = precompiles;
    }

    /// Load access list for berlin hard fork.
    ///
    /// Loading of accounts/storages is needed to make them warm.
    #[inline]
    pub fn load_access_list(&mut self) -> Result<(), EVMError<DB::Error>> {
        for (address, slots) in self.env.tx.access_list.iter() {
            self.journaled_state
                .initial_account_load(*address, slots, &mut self.db)
                .map_err(EVMError::Database)?;
        }
        Ok(())
    }

    /// Return environment.
    #[inline]
    pub fn env(&mut self) -> &mut Env {
        &mut self.env
    }

    /// Fetch block hash from database.
    #[inline]
    #[must_use]
    pub fn block_hash(&mut self, number: U256) -> Option<B256> {
        self.db
            .block_hash(number)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Load account and return flags (is_cold, exists)
    #[inline]
    #[must_use]
    pub fn load_account(&mut self, address: Address) -> Option<(bool, bool)> {
        self.journaled_state
            .load_account_exist(address, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Return account balance and is_cold flag.
    #[inline]
    #[must_use]
    pub fn balance(&mut self, address: Address) -> Option<(U256, bool)> {
        self.journaled_state
            .load_account(address, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
            .map(|(acc, is_cold)| (acc.info.balance, is_cold))
    }

    /// Return account code and if address is cold loaded.
    #[inline]
    #[must_use]
    pub fn code(&mut self, address: Address) -> Option<(Bytecode, bool)> {
        let (acc, is_cold) = self
            .journaled_state
            .load_code(address, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .ok()?;
        Some((acc.info.code.clone().unwrap(), is_cold))
    }

    /// Get code hash of address.
    #[inline]
    #[must_use]
    pub fn code_hash(&mut self, address: Address) -> Option<(B256, bool)> {
        let (acc, is_cold) = self
            .journaled_state
            .load_code(address, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .ok()?;
        if acc.is_empty() {
            return Some((B256::ZERO, is_cold));
        }

        Some((acc.info.code_hash, is_cold))
    }

    /// Load storage slot, if storage is not present inside the account then it will be loaded from database.
    #[inline]
    #[must_use]
    pub fn sload(&mut self, address: Address, index: U256) -> Option<(U256, bool)> {
        // account is always warm. reference on that statement https://eips.ethereum.org/EIPS/eip-2929 see `Note 2:`
        self.journaled_state
            .sload(address, index, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Storage change of storage slot, before storing `sload` will be called for that slot.
    #[inline]
    #[must_use]
    pub fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        self.journaled_state
            .sstore(address, index, value, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Returns transient storage value.
    #[inline]
    pub fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.journaled_state.tload(address, index)
    }

    /// Stores transient storage value.
    #[inline]
    pub fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.journaled_state.tstore(address, index, value)
    }

    /// Make create frame.
    #[inline]
    #[must_use]
    pub fn make_create_frame(&mut self, spec_id: SpecId, inputs: &CreateInputs) -> FrameOrResult {
        // Prepare crate.
        let gas = Gas::new(inputs.gas_limit);

        let return_error = |e| {
            FrameOrResult::new_create_result(
                InterpreterResult {
                    result: e,
                    gas,
                    output: Bytes::new(),
                },
                None,
            )
        };

        // Check depth
        if self.journaled_state.depth() > CALL_STACK_LIMIT {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Fetch balance of caller.
        let Some((caller_balance, _)) = self.balance(inputs.caller) else {
            return return_error(InstructionResult::FatalExternalError);
        };

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

        // Load account so it needs to be marked as warm for access list.
        if self
            .journaled_state
            .load_account(created_address, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .is_err()
        {
            return return_error(InstructionResult::FatalExternalError);
        }

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

        let bytecode = Bytecode::new_raw(inputs.init_code.clone());

        let contract = Box::new(Contract::new(
            Bytes::new(),
            bytecode,
            init_code_hash,
            created_address,
            inputs.caller,
            inputs.value,
        ));

        FrameOrResult::new_create_frame(
            created_address,
            checkpoint,
            Interpreter::new(contract, gas.limit(), false),
        )
    }

    /// Make call frame
    #[inline]
    #[must_use]
    pub fn make_call_frame(&mut self, inputs: &CallInputs) -> FrameOrResult {
        let gas = Gas::new(inputs.gas_limit);

        let return_result = |instruction_result: InstructionResult| {
            FrameOrResult::new_call_result(
                InterpreterResult {
                    result: instruction_result,
                    gas,
                    output: Bytes::new(),
                },
                inputs.return_memory_offset.clone(),
            )
        };

        // Check depth
        if self.journaled_state.depth() > CALL_STACK_LIMIT {
            return return_result(InstructionResult::CallTooDeep);
        }

        let account = match self
            .journaled_state
            .load_code(inputs.contract, &mut self.db)
        {
            Ok((account, _)) => account,
            Err(e) => {
                self.error = Some(e);
                return return_result(InstructionResult::FatalExternalError);
            }
        };
        let code_hash = account.info.code_hash();
        let bytecode = account.info.code.clone().unwrap_or_default();

        // Create subroutine checkpoint
        let checkpoint = self.journaled_state.checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if inputs.transfer.value == U256::ZERO {
            if self.load_account(inputs.context.address).is_none() {
                return return_result(InstructionResult::FatalExternalError);
            };
            self.journaled_state.touch(&inputs.context.address);
        }

        // Transfer value from caller to called account
        if let Err(e) = self.journaled_state.transfer(
            &inputs.transfer.source,
            &inputs.transfer.target,
            inputs.transfer.value,
            &mut self.db,
        ) {
            self.journaled_state.checkpoint_revert(checkpoint);
            return return_result(e);
        }

        if let Some(precompile) = self.precompiles.get(&inputs.contract) {
            let result = self.call_precompile(precompile, inputs, gas);
            if matches!(result.result, return_ok!()) {
                self.journaled_state.checkpoint_commit();
            } else {
                self.journaled_state.checkpoint_revert(checkpoint);
            }
            FrameOrResult::new_call_result(result, inputs.return_memory_offset.clone())
        } else if !bytecode.is_empty() {
            let contract = Box::new(Contract::new_with_context(
                inputs.input.clone(),
                bytecode,
                code_hash,
                &inputs.context,
            ));
            // Create interpreter and executes call and push new CallStackFrame.
            FrameOrResult::new_call_frame(
                inputs.return_memory_offset.clone(),
                checkpoint,
                Interpreter::new(contract, gas.limit(), inputs.is_static),
            )
        } else {
            self.journaled_state.checkpoint_commit();
            return_result(InstructionResult::Stop)
        }
    }

    /// Call precompile contract
    #[inline]
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
                if result.gas.record_cost(gas_used) {
                    result.result = InstructionResult::Return;
                    result.output = data;
                } else {
                    result.result = InstructionResult::PrecompileOOG;
                }
            }
            Err(e) => {
                result.result = if e == crate::precompile::Error::OutOfGas {
                    InstructionResult::PrecompileOOG
                } else {
                    InstructionResult::PrecompileError
                };
            }
        }
        result
    }

    /// Handles call return.
    #[inline]
    pub fn call_return(
        &mut self,
        interpreter_result: &InterpreterResult,
        journal_checkpoint: JournalCheckpoint,
    ) {
        // revert changes or not.
        if matches!(interpreter_result.result, return_ok!()) {
            self.journaled_state.checkpoint_commit();
        } else {
            self.journaled_state.checkpoint_revert(journal_checkpoint);
        }
    }

    /// Handles create return.
    #[inline]
    pub fn create_return<SPEC: Spec>(
        &mut self,
        interpreter_result: &mut InterpreterResult,
        address: Address,
        journal_checkpoint: JournalCheckpoint,
    ) {
        // if return is not ok revert and return.
        if !matches!(interpreter_result.result, return_ok!()) {
            self.journaled_state.checkpoint_revert(journal_checkpoint);
            return;
        }
        // Host error if present on execution
        // if ok, check contract creation limit and calculate gas deduction on output len.
        //
        // EIP-3541: Reject new contract code starting with the 0xEF byte
        if SPEC::enabled(LONDON)
            && !interpreter_result.output.is_empty()
            && interpreter_result.output.first() == Some(&0xEF)
        {
            self.journaled_state.checkpoint_revert(journal_checkpoint);
            interpreter_result.result = InstructionResult::CreateContractStartingWithEF;
            return;
        }

        // EIP-170: Contract code size limit
        // By default limit is 0x6000 (~25kb)
        if SPEC::enabled(SPURIOUS_DRAGON)
            && interpreter_result.output.len()
                > self
                    .env
                    .cfg
                    .limit_contract_code_size
                    .unwrap_or(MAX_CODE_SIZE)
        {
            self.journaled_state.checkpoint_revert(journal_checkpoint);
            interpreter_result.result = InstructionResult::CreateContractSizeLimit;
            return;
        }
        let gas_for_code = interpreter_result.output.len() as u64 * gas::CODEDEPOSIT;
        if !interpreter_result.gas.record_cost(gas_for_code) {
            // record code deposit gas cost and check if we are out of gas.
            // EIP-2 point 3: If contract creation does not have enough gas to pay for the
            // final gas fee for adding the contract code to the state, the contract
            //  creation fails (i.e. goes out-of-gas) rather than leaving an empty contract.
            if SPEC::enabled(HOMESTEAD) {
                self.journaled_state.checkpoint_revert(journal_checkpoint);
                interpreter_result.result = InstructionResult::OutOfGas;
                return;
            } else {
                interpreter_result.output = Bytes::new();
            }
        }
        // if we have enough gas we can commit changes.
        self.journaled_state.checkpoint_commit();

        // Do analysis of bytecode straight away.
        let bytecode = match self.env.cfg.perf_analyse_created_bytecodes {
            AnalysisKind::Raw => Bytecode::new_raw(interpreter_result.output.clone()),
            AnalysisKind::Check => {
                Bytecode::new_raw(interpreter_result.output.clone()).to_checked()
            }
            AnalysisKind::Analyse => {
                to_analysed(Bytecode::new_raw(interpreter_result.output.clone()))
            }
        };

        // set code
        self.journaled_state.set_code(address, bytecode);

        interpreter_result.result = InstructionResult::Return;
    }
}
/// Test utilities for the [`EvmContext`].
#[cfg(any(test, feature = "test-utils"))]
pub(crate) mod test_utils {
    use super::*;
    use crate::db::CacheDB;
    use crate::db::EmptyDB;
    use crate::primitives::address;
    use crate::primitives::SpecId;

    /// Mock caller address.
    pub const MOCK_CALLER: Address = address!("0000000000000000000000000000000000000000");

    /// Creates `CallInputs` that calls a provided contract address from the mock caller.
    pub fn create_mock_call_inputs(to: Address) -> CallInputs {
        CallInputs {
            contract: to,
            transfer: revm_interpreter::Transfer {
                source: MOCK_CALLER,
                target: to,
                value: U256::ZERO,
            },
            input: Bytes::new(),
            gas_limit: 0,
            context: revm_interpreter::CallContext {
                address: MOCK_CALLER,
                caller: MOCK_CALLER,
                code_address: MOCK_CALLER,
                apparent_value: U256::ZERO,
                scheme: revm_interpreter::CallScheme::Call,
            },
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
            env,
            journaled_state: JournaledState::new(SpecId::CANCUN, HashSet::new()),
            db,
            error: None,
            precompiles: Precompiles::default(),
            #[cfg(feature = "optimism")]
            l1_block_info: None,
        }
    }

    /// Returns a new `EvmContext` with an empty journaled state.
    pub fn create_empty_evm_context(env: Box<Env>, db: EmptyDB) -> EvmContext<EmptyDB> {
        EvmContext {
            env,
            journaled_state: JournaledState::new(SpecId::CANCUN, HashSet::new()),
            db,
            error: None,
            precompiles: Precompiles::default(),
            #[cfg(feature = "optimism")]
            l1_block_info: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{CacheDB, EmptyDB};
    use crate::primitives::address;
    use crate::{Frame, JournalEntry};
    use test_utils::*;

    // Tests that the `EVMContext::make_call_frame` function returns an error if the
    // call stack is too deep.
    #[test]
    fn test_make_call_frame_stack_too_deep() {
        let env = Env::default();
        let db = EmptyDB::default();
        let mut evm_context = test_utils::create_empty_evm_context(Box::new(env), db);
        evm_context.journaled_state.depth = CALL_STACK_LIMIT as usize + 1;
        let contract = address!("dead10000000000000000000000000000001dead");
        let call_inputs = test_utils::create_mock_call_inputs(contract);
        let res = evm_context.make_call_frame(&call_inputs);
        let FrameOrResult::Result(err) = res else {
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
        call_inputs.transfer.value = U256::from(1);
        let res = evm_context.make_call_frame(&call_inputs);
        let FrameOrResult::Result(result) = res else {
            panic!("Expected FrameOrResult::Result");
        };
        assert_eq!(
            result.interpreter_result().result,
            InstructionResult::OutOfFunds
        );
        let checkpointed = vec![vec![JournalEntry::AccountLoaded { address: contract }]];
        assert_eq!(evm_context.journaled_state.journal, checkpointed);
        assert_eq!(evm_context.journaled_state.depth, 0);
    }

    #[test]
    fn test_make_call_frame_missing_code_context() {
        let env = Env::default();
        let cdb = CacheDB::new(EmptyDB::default());
        let bal = U256::from(3_000_000_000_u128);
        let mut evm_context = create_cache_db_evm_context_with_balance(Box::new(env), cdb, bal);
        let contract = address!("dead10000000000000000000000000000001dead");
        let call_inputs = test_utils::create_mock_call_inputs(contract);
        let res = evm_context.make_call_frame(&call_inputs);
        let FrameOrResult::Result(result) = res else {
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
        let FrameOrResult::Frame(Frame::Call(call_frame)) = res else {
            panic!("Expected FrameOrResult::Frame(Frame::Call(..))");
        };
        assert_eq!(call_frame.return_memory_range, 0..0,);
    }
}
