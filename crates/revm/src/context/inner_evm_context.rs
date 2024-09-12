use derive_where::derive_where;

use crate::{journaled_state::JournaledState, JournalCheckpoint};
use bytecode::{Bytecode, Eof, EOF_MAGIC_BYTES, EOF_MAGIC_HASH};
use database_interface::Database;
use interpreter::{
    analysis::to_analysed, gas, return_ok, AccountLoad, Eip7702CodeLoad, InstructionResult,
    InterpreterResult, SStoreResult, SelfDestructResult, StateLoad,
};
use primitives::{Address, Bytes, HashSet, B256, U256};
use specification::{
    eip2930::AccessListItem,
    hardfork::{
        Spec,
        SpecId::{self, *},
    },
};
use state::Account;
use std::{boxed::Box, sync::Arc};
use wiring::{
    default::{AnalysisKind, CfgEnv, EnvWiring},
    EvmWiring, Transaction,
};

/// EVM contexts contains data that EVM needs for execution.
#[derive_where(Clone, Debug; EvmWiringT::Block, EvmWiringT::ChainContext, EvmWiringT::Transaction, EvmWiringT::Database, <EvmWiringT::Database as Database>::Error)]
pub struct InnerEvmContext<EvmWiringT: EvmWiring> {
    /// EVM Environment contains all the information about config, block and transaction that
    /// evm needs.
    pub env: Box<EnvWiring<EvmWiringT>>,
    /// EVM State with journaling support.
    pub journaled_state: JournaledState,
    /// Database to load data from.
    pub db: EvmWiringT::Database,
    /// Inner context.
    pub chain: EvmWiringT::ChainContext,
    /// Error that happened during execution.
    pub error: Result<(), <EvmWiringT::Database as Database>::Error>,
}

impl<EvmWiringT> InnerEvmContext<EvmWiringT>
where
    EvmWiringT: EvmWiring<Block: Default, Transaction: Default>,
{
    pub fn new(db: EvmWiringT::Database) -> Self {
        Self {
            env: Box::default(),
            journaled_state: JournaledState::new(SpecId::LATEST, HashSet::new()),
            db,
            chain: Default::default(),
            error: Ok(()),
        }
    }
}

impl<EvmWiringT: EvmWiring> InnerEvmContext<EvmWiringT> {
    /// Creates a new context with the given environment and database.
    #[inline]
    pub fn new_with_env(db: EvmWiringT::Database, env: Box<EnvWiring<EvmWiringT>>) -> Self {
        Self {
            env,
            journaled_state: JournaledState::new(SpecId::LATEST, HashSet::new()),
            db,
            chain: Default::default(),
            error: Ok(()),
        }
    }

    /// Sets the database.
    ///
    /// Note that this will ignore the previous `error` if set.
    #[inline]
    pub fn with_db<
        OWiring: EvmWiring<Block = EvmWiringT::Block, Transaction = EvmWiringT::Transaction>,
    >(
        self,
        db: OWiring::Database,
    ) -> InnerEvmContext<OWiring> {
        InnerEvmContext {
            env: self.env,
            journaled_state: self.journaled_state,
            db,
            chain: Default::default(),
            error: Ok(()),
        }
    }

    /// Returns the configured EVM spec ID.
    #[inline]
    pub const fn spec_id(&self) -> SpecId {
        self.journaled_state.spec
    }

    /// Load access list for berlin hard fork.
    ///
    /// Loading of accounts/storages is needed to make them warm.
    #[inline]
    pub fn load_access_list(&mut self) -> Result<(), <EvmWiringT::Database as Database>::Error> {
        for AccessListItem {
            address,
            storage_keys,
        } in self.env.tx.access_list()
        {
            self.journaled_state.initial_account_load(
                *address,
                storage_keys.iter().map(|i| U256::from_be_bytes(i.0)),
                &mut self.db,
            )?;
        }
        Ok(())
    }

    /// Return environment.
    #[inline]
    pub fn env(&mut self) -> &mut EnvWiring<EvmWiringT> {
        &mut self.env
    }

    /// Returns reference to [`CfgEnv`].
    pub fn cfg(&self) -> &CfgEnv {
        &self.env.cfg
    }

    /// Returns the error by replacing it with `Ok(())`, if any.
    #[inline]
    pub fn take_error(&mut self) -> Result<(), <EvmWiringT::Database as Database>::Error> {
        core::mem::replace(&mut self.error, Ok(()))
    }

    /// Fetch block hash from database.
    #[inline]
    pub fn block_hash(
        &mut self,
        number: u64,
    ) -> Result<B256, <EvmWiringT::Database as Database>::Error> {
        self.db.block_hash(number)
    }

    /// Mark account as touched as only touched accounts will be added to state.
    #[inline]
    pub fn touch(&mut self, address: &Address) {
        self.journaled_state.touch(address);
    }

    /// Loads an account into memory. Returns `true` if it is cold accessed.
    #[inline]
    pub fn load_account(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&mut Account>, <EvmWiringT::Database as Database>::Error> {
        self.journaled_state.load_account(address, &mut self.db)
    }

    /// Load account from database to JournaledState.
    ///
    /// Return boolean pair where first is `is_cold` second bool `exists`.
    #[inline]
    pub fn load_account_delegated(
        &mut self,
        address: Address,
    ) -> Result<AccountLoad, <EvmWiringT::Database as Database>::Error> {
        self.journaled_state
            .load_account_delegated(address, &mut self.db)
    }

    /// Return account balance and is_cold flag.
    #[inline]
    pub fn balance(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<U256>, <EvmWiringT::Database as Database>::Error> {
        self.journaled_state
            .load_account(address, &mut self.db)
            .map(|acc| acc.map(|a| a.info.balance))
    }

    /// Return account code bytes and if address is cold loaded.
    ///
    /// In case of EOF account it will return `EOF_MAGIC` (0xEF00) as code.
    #[inline]
    pub fn code(
        &mut self,
        address: Address,
    ) -> Result<Eip7702CodeLoad<Bytes>, <EvmWiringT::Database as Database>::Error> {
        let a = self.journaled_state.load_code(address, &mut self.db)?;
        // SAFETY: safe to unwrap as load_code will insert code if it is empty.
        let code = a.info.code.as_ref().unwrap();
        if code.is_eof() {
            return Ok(Eip7702CodeLoad::new_not_delegated(
                EOF_MAGIC_BYTES.clone(),
                a.is_cold,
            ));
        }

        if let Bytecode::Eip7702(code) = code {
            let address = code.address();
            let is_cold = a.is_cold;

            let delegated_account = self.journaled_state.load_code(address, &mut self.db)?;

            // SAFETY: safe to unwrap as load_code will insert code if it is empty.
            let delegated_code = delegated_account.info.code.as_ref().unwrap();

            let bytes = if delegated_code.is_eof() {
                EOF_MAGIC_BYTES.clone()
            } else {
                delegated_code.original_bytes()
            };

            return Ok(Eip7702CodeLoad::new(
                StateLoad::new(bytes, is_cold),
                delegated_account.is_cold,
            ));
        }

        Ok(Eip7702CodeLoad::new_not_delegated(
            code.original_bytes(),
            a.is_cold,
        ))
    }

    /// Get code hash of address.
    ///
    /// In case of EOF account it will return `EOF_MAGIC_HASH`
    /// (the hash of `0xEF00`).
    #[inline]
    pub fn code_hash(
        &mut self,
        address: Address,
    ) -> Result<Eip7702CodeLoad<B256>, <EvmWiringT::Database as Database>::Error> {
        let acc = self.journaled_state.load_code(address, &mut self.db)?;
        if acc.is_empty() {
            return Ok(Eip7702CodeLoad::new_not_delegated(B256::ZERO, acc.is_cold));
        }
        // SAFETY: safe to unwrap as load_code will insert code if it is empty.
        let code = acc.info.code.as_ref().unwrap();

        // If bytecode is EIP-7702 then we need to load the delegated account.
        if let Bytecode::Eip7702(code) = code {
            let address = code.address();
            let is_cold = acc.is_cold;

            let delegated_account = self.journaled_state.load_code(address, &mut self.db)?;

            let hash = if delegated_account.is_empty() {
                B256::ZERO
            } else if delegated_account.info.code.as_ref().unwrap().is_eof() {
                EOF_MAGIC_HASH
            } else {
                delegated_account.info.code_hash
            };

            return Ok(Eip7702CodeLoad::new(
                StateLoad::new(hash, is_cold),
                delegated_account.is_cold,
            ));
        }

        let hash = if code.is_eof() {
            EOF_MAGIC_HASH
        } else {
            acc.info.code_hash
        };

        Ok(Eip7702CodeLoad::new_not_delegated(hash, acc.is_cold))
    }

    /// Load storage slot, if storage is not present inside the account then it will be loaded from database.
    #[inline]
    pub fn sload(
        &mut self,
        address: Address,
        index: U256,
    ) -> Result<StateLoad<U256>, <EvmWiringT::Database as Database>::Error> {
        // account is always warm. reference on that statement https://eips.ethereum.org/EIPS/eip-2929 see `Note 2:`
        self.journaled_state.sload(address, index, &mut self.db)
    }

    /// Storage change of storage slot, before storing `sload` will be called for that slot.
    #[inline]
    pub fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Result<StateLoad<SStoreResult>, <EvmWiringT::Database as Database>::Error> {
        self.journaled_state
            .sstore(address, index, value, &mut self.db)
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

    /// Selfdestructs the account.
    #[inline]
    pub fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Result<StateLoad<SelfDestructResult>, <EvmWiringT::Database as Database>::Error> {
        self.journaled_state
            .selfdestruct(address, target, &mut self.db)
    }

    /// If error is present revert changes, otherwise save EOF bytecode.
    pub fn eofcreate_return<SPEC: Spec>(
        &mut self,
        interpreter_result: &mut InterpreterResult,
        address: Address,
        journal_checkpoint: JournalCheckpoint,
    ) {
        // Note we still execute RETURN opcode and return the bytes.
        // In EOF those opcodes should abort execution.
        //
        // In RETURN gas is still protecting us from ddos and in oog,
        // behaviour will be same as if it failed on return.
        //
        // Bytes of RETURN will drained in `insert_eofcreate_outcome`.
        if interpreter_result.result != InstructionResult::ReturnContract {
            self.journaled_state.checkpoint_revert(journal_checkpoint);
            return;
        }

        if interpreter_result.output.len() > self.cfg().max_code_size() {
            self.journaled_state.checkpoint_revert(journal_checkpoint);
            interpreter_result.result = InstructionResult::CreateContractSizeLimit;
            return;
        }

        // deduct gas for code deployment.
        let gas_for_code = interpreter_result.output.len() as u64 * gas::CODEDEPOSIT;
        if !interpreter_result.gas.record_cost(gas_for_code) {
            self.journaled_state.checkpoint_revert(journal_checkpoint);
            interpreter_result.result = InstructionResult::OutOfGas;
            return;
        }

        // commit changes reduces depth by -1.
        self.journaled_state.checkpoint_commit();

        // decode bytecode has a performance hit, but it has reasonable restrains.
        let bytecode =
            Eof::decode(interpreter_result.output.clone()).expect("Eof is already verified");

        // eof bytecode is going to be hashed.
        self.journaled_state
            .set_code(address, Bytecode::Eof(Arc::new(bytecode)));
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
        if SPEC::enabled(LONDON) && interpreter_result.output.first() == Some(&0xEF) {
            self.journaled_state.checkpoint_revert(journal_checkpoint);
            interpreter_result.result = InstructionResult::CreateContractStartingWithEF;
            return;
        }

        // EIP-170: Contract code size limit
        // By default limit is 0x6000 (~25kb)
        if SPEC::enabled(SPURIOUS_DRAGON)
            && interpreter_result.output.len() > self.cfg().max_code_size()
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
            AnalysisKind::Raw => Bytecode::new_legacy(interpreter_result.output.clone()),
            AnalysisKind::Analyse => {
                to_analysed(Bytecode::new_legacy(interpreter_result.output.clone()))
            }
        };

        // set code
        self.journaled_state.set_code(address, bytecode);

        interpreter_result.result = InstructionResult::Return;
    }
}
