use bytecode::Bytecode;
use context_interface::{
    context::{SStoreResult, SelfDestructResult, StateLoad},
    journaled_state::{AccountLoad, Journal, JournalCheckpoint, TransferError},
};
use database_interface::Database;
use primitives::{
    hash_map::Entry, Address, HashMap, HashSet, Log, B256, KECCAK_EMPTY, PRECOMPILE3, U256,
};
use specification::hardfork::{SpecId, SpecId::*};
use state::{Account, EvmState, EvmStorageSlot, TransientStorage};

use core::mem;
use std::{vec, vec::Vec};

use crate::JournalInit;

/// Trait for journal entries. it tracks changes across the state and allows to revert them.
pub trait JournalEntryTr {
    fn account_warmed(address: Address) -> Self;

    fn account_destroyed(
        address: Address,
        target: Address,
        was_destroyed: bool,
        had_balance: U256,
    ) -> Self;

    fn account_touched(address: Address) -> Self;

    fn balance_transfer(from: Address, to: Address, balance: U256) -> Self;

    fn nonce_changed(address: Address) -> Self;

    fn account_created(address: Address) -> Self;

    fn storage_changed(address: Address, key: U256, had_value: U256) -> Self;

    fn storage_warmed(address: Address, key: U256) -> Self;

    fn transient_storage_changed(address: Address, key: U256, had_value: U256) -> Self;

    fn code_changed(address: Address) -> Self;

    fn revert(
        self,
        state: &mut EvmState,
        transient_storage: &mut TransientStorage,
        is_spurious_dragon_enabled: bool,
    );
}

/// A journal of state changes internal to the EVM
///
/// On each additional call, the depth of the journaled state is increased (`depth`) and a new journal is added.
///
/// The journal contains every state change that happens within that call, making it possible to revert changes made in a specific call.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JournaledState<DB, ENTRY = JournalEntry>
where
    ENTRY: JournalEntryTr,
{
    /// Database
    pub database: DB,
    /// The current state
    pub state: EvmState,
    /// Transient storage that is discarded after every transaction.
    ///
    /// See [EIP-1153](https://eips.ethereum.org/EIPS/eip-1153).
    pub transient_storage: TransientStorage,
    /// Emitted logs
    pub logs: Vec<Log>,
    /// The current call stack depth
    pub depth: usize,
    /// The journal of state changes, one for each call
    pub journal: Vec<Vec<ENTRY>>,
    /// The spec ID for the EVM
    ///
    /// This spec is used for two things:
    ///
    /// - [EIP-161]: Prior to this EIP, Ethereum had separate definitions for empty and non-existing accounts.
    /// - [EIP-6780]: `SELFDESTRUCT` only in same transaction
    ///
    /// [EIP-161]: https://eips.ethereum.org/EIPS/eip-161
    /// [EIP-6780]: https://eips.ethereum.org/EIPS/eip-6780
    pub spec: SpecId,
    /// Warm loaded addresses are used to check if loaded address
    /// should be considered cold or warm loaded when the account
    /// is first accessed.
    ///
    /// Note that this not include newly loaded accounts, account and storage
    /// is considered warm if it is found in the `State`.
    pub warm_preloaded_addresses: HashSet<Address>,
    /// Precompile addresses
    pub precompiles: HashSet<Address>,
}

impl<DB: Database, ENTRY: JournalEntryTr> Journal for JournaledState<DB, ENTRY> {
    type Database = DB;
    // TODO : Make a struck here.
    type FinalOutput = (EvmState, Vec<Log>);

    fn new(database: DB) -> JournaledState<DB, ENTRY> {
        Self::new(SpecId::LATEST, database)
    }

    fn db_ref(&self) -> &Self::Database {
        &self.database
    }

    fn db(&mut self) -> &mut Self::Database {
        &mut self.database
    }

    fn sload(
        &mut self,
        address: Address,
        key: U256,
    ) -> Result<StateLoad<U256>, <Self::Database as Database>::Error> {
        self.sload(address, key)
    }

    fn sstore(
        &mut self,
        address: Address,
        key: U256,
        value: U256,
    ) -> Result<StateLoad<SStoreResult>, <Self::Database as Database>::Error> {
        self.sstore(address, key, value)
    }

    fn tload(&mut self, address: Address, key: U256) -> U256 {
        self.tload(address, key)
    }

    fn tstore(&mut self, address: Address, key: U256, value: U256) {
        self.tstore(address, key, value)
    }

    fn log(&mut self, log: Log) {
        self.log(log)
    }

    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Result<StateLoad<SelfDestructResult>, DB::Error> {
        self.selfdestruct(address, target)
    }

    fn warm_account(&mut self, address: Address) {
        self.warm_preloaded_addresses.insert(address);
    }

    fn warm_precompiles(&mut self, address: HashSet<Address>) {
        self.precompiles = address;
        self.warm_preloaded_addresses
            .extend(self.precompiles.iter());
    }

    #[inline]
    fn precompile_addresses(&self) -> &HashSet<Address> {
        &self.precompiles
    }

    /// Returns call depth.
    #[inline]
    fn depth(&self) -> usize {
        self.depth
    }

    fn warm_account_and_storage(
        &mut self,
        address: Address,
        storage_keys: impl IntoIterator<Item = U256>,
    ) -> Result<(), <Self::Database as Database>::Error> {
        self.initial_account_load(address, storage_keys)?;
        Ok(())
    }

    fn set_spec_id(&mut self, spec_id: SpecId) {
        self.spec = spec_id;
    }

    fn transfer(
        &mut self,
        from: &Address,
        to: &Address,
        balance: U256,
    ) -> Result<Option<TransferError>, DB::Error> {
        self.transfer(from, to, balance)
    }

    fn touch_account(&mut self, address: Address) {
        self.touch(&address);
    }

    fn inc_account_nonce(&mut self, address: Address) -> Result<Option<u64>, DB::Error> {
        Ok(self.inc_nonce(address))
    }

    fn load_account(&mut self, address: Address) -> Result<StateLoad<&mut Account>, DB::Error> {
        self.load_account(address)
    }

    fn load_account_code(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&mut Account>, DB::Error> {
        self.load_code(address)
    }

    fn load_account_delegated(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<AccountLoad>, DB::Error> {
        self.load_account_delegated(address)
    }

    fn checkpoint(&mut self) -> JournalCheckpoint {
        self.checkpoint()
    }

    fn checkpoint_commit(&mut self) {
        self.checkpoint_commit()
    }

    fn checkpoint_revert(&mut self, checkpoint: JournalCheckpoint) {
        self.checkpoint_revert(checkpoint)
    }

    fn set_code_with_hash(&mut self, address: Address, code: Bytecode, hash: B256) {
        self.set_code_with_hash(address, code, hash);
    }

    fn clear(&mut self) {
        // Clears the JournaledState. Preserving only the spec.
        self.state.clear();
        self.transient_storage.clear();
        self.logs.clear();
        self.journal = vec![vec![]];
        self.depth = 0;
        self.warm_preloaded_addresses.clear();
    }

    fn create_account_checkpoint(
        &mut self,
        caller: Address,
        address: Address,
        balance: U256,
        spec_id: SpecId,
    ) -> Result<JournalCheckpoint, TransferError> {
        // Ignore error.
        self.create_account_checkpoint(caller, address, balance, spec_id)
    }

    fn finalize(&mut self) -> Self::FinalOutput {
        let Self {
            state,
            transient_storage,
            logs,
            depth,
            journal,
            // kept, see [Self::new]
            spec: _,
            database: _,
            warm_preloaded_addresses: _,
            precompiles: _,
        } = self;

        *transient_storage = TransientStorage::default();
        *journal = vec![vec![]];
        *depth = 0;
        let state = mem::take(state);
        let logs = mem::take(logs);

        (state, logs)
    }
}

impl<DB: Database, ENTRY: JournalEntryTr> JournaledState<DB, ENTRY> {
    /// Creates new JournaledState.
    ///
    /// `warm_preloaded_addresses` is used to determine if address is considered warm loaded.
    /// In ordinary case this is precompile or beneficiary.
    ///
    /// # Note
    /// This function will journal state after Spurious Dragon fork.
    /// And will not take into account if account is not existing or empty.
    pub fn new(spec: SpecId, database: DB) -> JournaledState<DB, ENTRY> {
        Self {
            database,
            state: HashMap::default(),
            transient_storage: TransientStorage::default(),
            logs: Vec::new(),
            journal: vec![vec![]],
            depth: 0,
            spec,
            warm_preloaded_addresses: HashSet::default(),
            precompiles: HashSet::default(),
        }
    }

    /// Return reference to state.
    #[inline]
    pub fn state(&mut self) -> &mut EvmState {
        &mut self.state
    }

    /// Sets SpecId.
    #[inline]
    pub fn set_spec_id(&mut self, spec: SpecId) {
        self.spec = spec;
    }

    /// Mark account as touched as only touched accounts will be added to state.
    /// This is especially important for state clear where touched empty accounts needs to
    /// be removed from state.
    #[inline]
    pub fn touch(&mut self, address: &Address) {
        if let Some(account) = self.state.get_mut(address) {
            Self::touch_account(self.journal.last_mut().unwrap(), address, account);
        }
    }

    /// Mark account as touched.
    #[inline]
    fn touch_account(journal: &mut Vec<ENTRY>, address: &Address, account: &mut Account) {
        if !account.is_touched() {
            journal.push(ENTRY::account_touched(*address));
            account.mark_touch();
        }
    }

    /// Returns the _loaded_ [Account] for the given address.
    ///
    /// This assumes that the account has already been loaded.
    ///
    /// # Panics
    ///
    /// Panics if the account has not been loaded and is missing from the state set.
    #[inline]
    pub fn account(&self, address: Address) -> &Account {
        self.state
            .get(&address)
            .expect("Account expected to be loaded") // Always assume that acc is already loaded
    }

    /// Set code and its hash to the account.
    ///
    /// Note: Assume account is warm and that hash is calculated from code.
    #[inline]
    pub fn set_code_with_hash(&mut self, address: Address, code: Bytecode, hash: B256) {
        let account = self.state.get_mut(&address).unwrap();
        Self::touch_account(self.journal.last_mut().unwrap(), &address, account);

        self.journal
            .last_mut()
            .unwrap()
            .push(ENTRY::code_changed(address));

        account.info.code_hash = hash;
        account.info.code = Some(code);
    }

    /// Use it only if you know that acc is warm.
    ///
    /// Assume account is warm.
    #[inline]
    pub fn set_code(&mut self, address: Address, code: Bytecode) {
        let hash = code.hash_slow();
        self.set_code_with_hash(address, code, hash)
    }

    #[inline]
    pub fn inc_nonce(&mut self, address: Address) -> Option<u64> {
        let account = self.state.get_mut(&address).unwrap();
        // Check if nonce is going to overflow.
        if account.info.nonce == u64::MAX {
            return None;
        }
        Self::touch_account(self.journal.last_mut().unwrap(), &address, account);
        self.journal
            .last_mut()
            .unwrap()
            .push(ENTRY::nonce_changed(address));

        account.info.nonce += 1;

        Some(account.info.nonce)
    }

    /// Transfers balance from two accounts. Returns error if sender balance is not enough.
    #[inline]
    pub fn transfer(
        &mut self,
        from: &Address,
        to: &Address,
        balance: U256,
    ) -> Result<Option<TransferError>, DB::Error> {
        if balance.is_zero() {
            self.load_account(*to)?;
            let _ = self.load_account(*to)?;
            let to_account = self.state.get_mut(to).unwrap();
            Self::touch_account(self.journal.last_mut().unwrap(), to, to_account);
            return Ok(None);
        }
        // load accounts
        self.load_account(*from)?;
        self.load_account(*to)?;

        // sub balance from
        let from_account = &mut self.state.get_mut(from).unwrap();
        Self::touch_account(self.journal.last_mut().unwrap(), from, from_account);
        let from_balance = &mut from_account.info.balance;

        let Some(from_balance_decr) = from_balance.checked_sub(balance) else {
            return Ok(Some(TransferError::OutOfFunds));
        };
        *from_balance = from_balance_decr;

        // add balance to
        let to_account = &mut self.state.get_mut(to).unwrap();
        Self::touch_account(self.journal.last_mut().unwrap(), to, to_account);
        let to_balance = &mut to_account.info.balance;
        let Some(to_balance_incr) = to_balance.checked_add(balance) else {
            return Ok(Some(TransferError::OverflowPayment));
        };
        *to_balance = to_balance_incr;
        // Overflow of U256 balance is not possible to happen on mainnet. We don't bother to return funds from from_acc.

        self.journal
            .last_mut()
            .unwrap()
            .push(ENTRY::balance_transfer(*from, *to, balance));

        Ok(None)
    }

    /// Creates account or returns false if collision is detected.
    ///
    /// There are few steps done:
    /// 1. Make created account warm loaded (AccessList) and this should
    ///     be done before subroutine checkpoint is created.
    /// 2. Check if there is collision of newly created account with existing one.
    /// 3. Mark created account as created.
    /// 4. Add fund to created account
    /// 5. Increment nonce of created account if SpuriousDragon is active
    /// 6. Decrease balance of caller account.
    ///
    /// # Panics
    ///
    /// Panics if the caller is not loaded inside the EVM state.
    /// This should have been done inside `create_inner`.
    #[inline]
    pub fn create_account_checkpoint(
        &mut self,
        caller: Address,
        target_address: Address,
        balance: U256,
        spec_id: SpecId,
    ) -> Result<JournalCheckpoint, TransferError> {
        // Enter subroutine
        let checkpoint = self.checkpoint();

        // Fetch balance of caller.
        let caller_balance = self.state.get(&caller).unwrap().info.balance;
        // Check if caller has enough balance to send to the created contract.
        if caller_balance < balance {
            self.checkpoint_revert(checkpoint);
            return Err(TransferError::OutOfFunds);
        }

        // Newly created account is present, as we just loaded it.
        let target_acc = self.state.get_mut(&target_address).unwrap();
        let last_journal = self.journal.last_mut().unwrap();

        // New account can be created if:
        // Bytecode is not empty.
        // Nonce is not zero
        // Account is not precompile.
        if target_acc.info.code_hash != KECCAK_EMPTY || target_acc.info.nonce != 0 {
            self.checkpoint_revert(checkpoint);
            return Err(TransferError::CreateCollision);
        }

        // set account status to create.
        target_acc.mark_created();

        // this entry will revert set nonce.
        last_journal.push(ENTRY::account_created(target_address));
        target_acc.info.code = None;
        // EIP-161: State trie clearing (invariant-preserving alternative)
        if spec_id.is_enabled_in(SPURIOUS_DRAGON) {
            // nonce is going to be reset to zero in AccountCreated journal entry.
            target_acc.info.nonce = 1;
        }

        // touch account. This is important as for pre SpuriousDragon account could be
        // saved even empty.
        Self::touch_account(last_journal, &target_address, target_acc);

        // Add balance to created account, as we already have target here.
        let Some(new_balance) = target_acc.info.balance.checked_add(balance) else {
            self.checkpoint_revert(checkpoint);
            return Err(TransferError::OverflowPayment);
        };
        target_acc.info.balance = new_balance;

        // safe to decrement for the caller as balance check is already done.
        self.state.get_mut(&caller).unwrap().info.balance -= balance;

        // add journal entry of transferred balance
        last_journal.push(ENTRY::balance_transfer(caller, target_address, balance));

        Ok(checkpoint)
    }

    /// Reverts all changes that happened in given journal entries.
    #[inline]
    fn journal_revert(
        state: &mut EvmState,
        transient_storage: &mut TransientStorage,
        journal_entries: Vec<ENTRY>,
        is_spurious_dragon_enabled: bool,
    ) {
        for entry in journal_entries.into_iter().rev() {
            entry.revert(state, transient_storage, is_spurious_dragon_enabled);
        }
    }

    /// Makes a checkpoint that in case of Revert can bring back state to this point.
    #[inline]
    pub fn checkpoint(&mut self) -> JournalCheckpoint {
        let checkpoint = JournalCheckpoint {
            log_i: self.logs.len(),
            journal_i: self.journal.len(),
        };
        self.depth += 1;
        self.journal.push(Default::default());
        checkpoint
    }

    /// Commits the checkpoint.
    #[inline]
    pub fn checkpoint_commit(&mut self) {
        self.depth -= 1;
    }

    /// Reverts all changes to state until given checkpoint.
    #[inline]
    pub fn checkpoint_revert(&mut self, checkpoint: JournalCheckpoint) {
        let is_spurious_dragon_enabled = self.spec.is_enabled_in(SPURIOUS_DRAGON);
        let state = &mut self.state;
        let transient_storage = &mut self.transient_storage;
        self.depth -= 1;
        // iterate over last N journals sets and revert our global state
        let len = self.journal.len();
        self.journal
            .iter_mut()
            .rev()
            .take(len - checkpoint.journal_i)
            .for_each(|cs| {
                Self::journal_revert(
                    state,
                    transient_storage,
                    mem::take(cs),
                    is_spurious_dragon_enabled,
                )
            });

        self.logs.truncate(checkpoint.log_i);
        self.journal.truncate(checkpoint.journal_i);
    }

    /// Performs selfdestruct action.
    /// Transfers balance from address to target. Check if target exist/is_cold
    ///
    /// Note: Balance will be lost if address and target are the same BUT when
    /// current spec enables Cancun, this happens only when the account associated to address
    /// is created in the same tx
    ///
    /// # References:
    ///  * <https://github.com/ethereum/go-ethereum/blob/141cd425310b503c5678e674a8c3872cf46b7086/core/vm/instructions.go#L832-L833>
    ///  * <https://github.com/ethereum/go-ethereum/blob/141cd425310b503c5678e674a8c3872cf46b7086/core/state/statedb.go#L449>
    ///  * <https://eips.ethereum.org/EIPS/eip-6780>
    #[inline]
    pub fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Result<StateLoad<SelfDestructResult>, DB::Error> {
        let spec = self.spec;
        let account_load = self.load_account(target)?;
        let is_cold = account_load.is_cold;
        let is_empty = account_load.state_clear_aware_is_empty(spec);

        if address != target {
            // Both accounts are loaded before this point, `address` as we execute its contract.
            // and `target` at the beginning of the function.
            let acc_balance = self.state.get(&address).unwrap().info.balance;

            let target_account = self.state.get_mut(&target).unwrap();
            Self::touch_account(self.journal.last_mut().unwrap(), &target, target_account);
            target_account.info.balance += acc_balance;
        }

        let acc = self.state.get_mut(&address).unwrap();
        let balance = acc.info.balance;
        let previously_destroyed = acc.is_selfdestructed();
        let is_cancun_enabled = self.spec.is_enabled_in(CANCUN);

        // EIP-6780 (Cancun hard-fork): selfdestruct only if contract is created in the same tx
        let journal_entry = if acc.is_created() || !is_cancun_enabled {
            acc.mark_selfdestruct();
            acc.info.balance = U256::ZERO;
            Some(ENTRY::account_destroyed(
                address,
                target,
                previously_destroyed,
                balance,
            ))
        } else if address != target {
            acc.info.balance = U256::ZERO;
            Some(ENTRY::balance_transfer(address, target, balance))
        } else {
            // State is not changed:
            // * if we are after Cancun upgrade and
            // * Selfdestruct account that is created in the same transaction and
            // * Specify the target is same as selfdestructed account. The balance stays unchanged.
            None
        };

        if let Some(entry) = journal_entry {
            self.journal.last_mut().unwrap().push(entry);
        };

        Ok(StateLoad {
            data: SelfDestructResult {
                had_value: !balance.is_zero(),
                target_exists: !is_empty,
                previously_destroyed,
            },
            is_cold,
        })
    }

    /// Initial load of account. This load will not be tracked inside journal
    #[inline]
    pub fn initial_account_load(
        &mut self,
        address: Address,
        storage_keys: impl IntoIterator<Item = U256>,
    ) -> Result<&mut Account, DB::Error> {
        // load or get account.
        let account = match self.state.entry(address) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(vac) => vac.insert(
                self.database
                    .basic(address)?
                    .map(|i| i.into())
                    .unwrap_or(Account::new_not_existing()),
            ),
        };
        // preload storages.
        for storage_key in storage_keys.into_iter() {
            if let Entry::Vacant(entry) = account.storage.entry(storage_key) {
                let storage = self.database.storage(address, storage_key)?;
                entry.insert(EvmStorageSlot::new(storage));
            }
        }
        Ok(account)
    }

    /// Loads account into memory. return if it is cold or warm accessed
    #[inline]
    pub fn load_account(&mut self, address: Address) -> Result<StateLoad<&mut Account>, DB::Error> {
        self.load_account_optional(address, false)
    }

    #[inline]
    pub fn load_account_delegated(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<AccountLoad>, DB::Error> {
        let spec = self.spec;
        let account = self.load_code(address)?;
        let is_empty = account.state_clear_aware_is_empty(spec);

        let mut account_load = StateLoad::new(
            AccountLoad {
                is_delegate_account_cold: None,
                is_empty,
            },
            account.is_cold,
        );

        // load delegate code if account is EIP-7702
        if let Some(Bytecode::Eip7702(code)) = &account.info.code {
            let address = code.address();
            let delegate_account = self.load_account(address)?;
            account_load.data.is_delegate_account_cold = Some(delegate_account.is_cold);
        }

        Ok(account_load)
    }

    pub fn load_code(&mut self, address: Address) -> Result<StateLoad<&mut Account>, DB::Error> {
        self.load_account_optional(address, true)
    }

    /// Loads code
    #[inline]
    pub fn load_account_optional(
        &mut self,
        address: Address,
        load_code: bool,
    ) -> Result<StateLoad<&mut Account>, DB::Error> {
        let load = match self.state.entry(address) {
            Entry::Occupied(entry) => {
                let account = entry.into_mut();
                let is_cold = account.mark_warm();
                StateLoad {
                    data: account,
                    is_cold,
                }
            }
            Entry::Vacant(vac) => {
                let account = if let Some(account) = self.database.basic(address)? {
                    account.into()
                } else {
                    Account::new_not_existing()
                };

                // precompiles are warm loaded so we need to take that into account
                let is_cold = !self.warm_preloaded_addresses.contains(&address);

                StateLoad {
                    data: vac.insert(account),
                    is_cold,
                }
            }
        };
        // journal loading of cold account.
        if load.is_cold {
            self.journal
                .last_mut()
                .unwrap()
                .push(ENTRY::account_warmed(address));
        }
        if load_code {
            let info = &mut load.data.info;
            if info.code.is_none() {
                let code = if info.code_hash == KECCAK_EMPTY {
                    Bytecode::default()
                } else {
                    self.database.code_by_hash(info.code_hash)?
                };
                info.code = Some(code);
            }
        }

        Ok(load)
    }

    /// Loads storage slot.
    ///
    /// # Panics
    ///
    /// Panics if the account is not present in the state.
    #[inline]
    pub fn sload(&mut self, address: Address, key: U256) -> Result<StateLoad<U256>, DB::Error> {
        // assume acc is warm
        let account = self.state.get_mut(&address).unwrap();
        // only if account is created in this tx we can assume that storage is empty.
        let is_newly_created = account.is_created();
        let (value, is_cold) = match account.storage.entry(key) {
            Entry::Occupied(occ) => {
                let slot = occ.into_mut();
                let is_cold = slot.mark_warm();
                (slot.present_value, is_cold)
            }
            Entry::Vacant(vac) => {
                // if storage was cleared, we don't need to ping db.
                let value = if is_newly_created {
                    U256::ZERO
                } else {
                    self.database.storage(address, key)?
                };

                vac.insert(EvmStorageSlot::new(value));

                (value, true)
            }
        };

        if is_cold {
            // add it to journal as cold loaded.
            self.journal
                .last_mut()
                .unwrap()
                .push(ENTRY::storage_warmed(address, key));
        }

        Ok(StateLoad::new(value, is_cold))
    }

    /// Stores storage slot.
    ///
    /// And returns (original,present,new) slot value.
    ///
    /// **Note**: Account should already be present in our state.
    #[inline]
    pub fn sstore(
        &mut self,
        address: Address,
        key: U256,
        new: U256,
    ) -> Result<StateLoad<SStoreResult>, DB::Error> {
        // assume that acc exists and load the slot.
        let present = self.sload(address, key)?;
        let acc = self.state.get_mut(&address).unwrap();

        // if there is no original value in dirty return present value, that is our original.
        let slot = acc.storage.get_mut(&key).unwrap();

        // new value is same as present, we don't need to do anything
        if present.data == new {
            return Ok(StateLoad::new(
                SStoreResult {
                    original_value: slot.original_value(),
                    present_value: present.data,
                    new_value: new,
                },
                present.is_cold,
            ));
        }

        self.journal
            .last_mut()
            .unwrap()
            .push(ENTRY::storage_changed(address, key, present.data));
        // insert value into present state.
        slot.present_value = new;
        Ok(StateLoad::new(
            SStoreResult {
                original_value: slot.original_value(),
                present_value: present.data,
                new_value: new,
            },
            present.is_cold,
        ))
    }

    /// Read transient storage tied to the account.
    ///
    /// EIP-1153: Transient storage opcodes
    #[inline]
    pub fn tload(&mut self, address: Address, key: U256) -> U256 {
        self.transient_storage
            .get(&(address, key))
            .copied()
            .unwrap_or_default()
    }

    /// Store transient storage tied to the account.
    ///
    /// If values is different add entry to the journal
    /// so that old state can be reverted if that action is needed.
    ///
    /// EIP-1153: Transient storage opcodes
    #[inline]
    pub fn tstore(&mut self, address: Address, key: U256, new: U256) {
        let had_value = if new.is_zero() {
            // if new values is zero, remove entry from transient storage.
            // if previous values was some insert it inside journal.
            // If it is none nothing should be inserted.
            self.transient_storage.remove(&(address, key))
        } else {
            // insert values
            let previous_value = self
                .transient_storage
                .insert((address, key), new)
                .unwrap_or_default();

            // check if previous value is same
            if previous_value != new {
                // if it is different, insert previous values inside journal.
                Some(previous_value)
            } else {
                None
            }
        };

        if let Some(had_value) = had_value {
            // insert in journal only if value was changed.
            self.journal
                .last_mut()
                .unwrap()
                .push(ENTRY::transient_storage_changed(address, key, had_value));
        }
    }

    /// Pushes log into subroutine.
    #[inline]
    pub fn log(&mut self, log: Log) {
        self.logs.push(log);
    }
}

/// Journal entries that are used to track changes to the state and are used to revert it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JournalEntry {
    /// Used to mark account that is warm inside EVM in regard to EIP-2929 AccessList.
    /// Action: We will add Account to state.
    /// Revert: we will remove account from state.
    AccountWarmed { address: Address },
    /// Mark account to be destroyed and journal balance to be reverted
    /// Action: Mark account and transfer the balance
    /// Revert: Unmark the account and transfer balance back
    AccountDestroyed {
        address: Address,
        target: Address,
        was_destroyed: bool, // if account had already been destroyed before this journal entry
        had_balance: U256,
    },
    /// Loading account does not mean that account will need to be added to MerkleTree (touched).
    /// Only when account is called (to execute contract or transfer balance) only then account is made touched.
    /// Action: Mark account touched
    /// Revert: Unmark account touched
    AccountTouched { address: Address },
    /// Transfer balance between two accounts
    /// Action: Transfer balance
    /// Revert: Transfer balance back
    BalanceTransfer {
        from: Address,
        to: Address,
        balance: U256,
    },
    /// Increment nonce
    /// Action: Increment nonce by one
    /// Revert: Decrement nonce by one
    NonceChange {
        address: Address, //geth has nonce value,
    },
    /// Create account:
    /// Actions: Mark account as created
    /// Revert: Unmark account as created and reset nonce to zero.
    AccountCreated { address: Address },
    /// Entry used to track storage changes
    /// Action: Storage change
    /// Revert: Revert to previous value
    StorageChanged {
        address: Address,
        key: U256,
        had_value: U256,
    },
    /// Entry used to track storage warming introduced by EIP-2929.
    /// Action: Storage warmed
    /// Revert: Revert to cold state
    StorageWarmed { address: Address, key: U256 },
    /// It is used to track an EIP-1153 transient storage change.
    /// Action: Transient storage changed.
    /// Revert: Revert to previous value.
    TransientStorageChange {
        address: Address,
        key: U256,
        had_value: U256,
    },
    /// Code changed
    /// Action: Account code changed
    /// Revert: Revert to previous bytecode.
    CodeChange { address: Address },
}
impl JournalEntryTr for JournalEntry {
    fn account_warmed(address: Address) -> Self {
        JournalEntry::AccountWarmed { address }
    }

    fn account_destroyed(
        address: Address,
        target: Address,
        was_destroyed: bool, // if account had already been destroyed before this journal entry
        had_balance: U256,
    ) -> Self {
        JournalEntry::AccountDestroyed {
            address,
            target,
            was_destroyed,
            had_balance,
        }
    }

    fn account_touched(address: Address) -> Self {
        JournalEntry::AccountTouched { address }
    }

    fn balance_transfer(from: Address, to: Address, balance: U256) -> Self {
        JournalEntry::BalanceTransfer { from, to, balance }
    }

    fn account_created(address: Address) -> Self {
        JournalEntry::AccountCreated { address }
    }

    fn storage_changed(address: Address, key: U256, had_value: U256) -> Self {
        JournalEntry::StorageChanged {
            address,
            key,
            had_value,
        }
    }

    fn nonce_changed(address: Address) -> Self {
        JournalEntry::NonceChange { address }
    }

    fn storage_warmed(address: Address, key: U256) -> Self {
        JournalEntry::StorageWarmed { address, key }
    }

    fn transient_storage_changed(address: Address, key: U256, had_value: U256) -> Self {
        JournalEntry::TransientStorageChange {
            address,
            key,
            had_value,
        }
    }

    fn code_changed(address: Address) -> Self {
        JournalEntry::CodeChange { address }
    }

    fn revert(
        self,
        state: &mut EvmState,
        transient_storage: &mut TransientStorage,
        is_spurious_dragon_enabled: bool,
    ) {
        match self {
            JournalEntry::AccountWarmed { address } => {
                state.get_mut(&address).unwrap().mark_cold();
            }
            JournalEntry::AccountTouched { address } => {
                if is_spurious_dragon_enabled && address == PRECOMPILE3 {
                    return;
                }
                // remove touched status
                state.get_mut(&address).unwrap().unmark_touch();
            }
            JournalEntry::AccountDestroyed {
                address,
                target,
                was_destroyed,
                had_balance,
            } => {
                let account = state.get_mut(&address).unwrap();
                // set previous state of selfdestructed flag, as there could be multiple
                // selfdestructs in one transaction.
                if was_destroyed {
                    // flag is still selfdestructed
                    account.mark_selfdestruct();
                } else {
                    // flag that is not selfdestructed
                    account.unmark_selfdestruct();
                }
                account.info.balance += had_balance;

                if address != target {
                    let target = state.get_mut(&target).unwrap();
                    target.info.balance -= had_balance;
                }
            }
            JournalEntry::BalanceTransfer { from, to, balance } => {
                // we don't need to check overflow and underflow when adding and subtracting the balance.
                let from = state.get_mut(&from).unwrap();
                from.info.balance += balance;
                let to = state.get_mut(&to).unwrap();
                to.info.balance -= balance;
            }
            JournalEntry::NonceChange { address } => {
                state.get_mut(&address).unwrap().info.nonce -= 1;
            }
            JournalEntry::AccountCreated { address } => {
                let account = &mut state.get_mut(&address).unwrap();
                account.unmark_created();
                account
                    .storage
                    .values_mut()
                    .for_each(|slot| slot.mark_cold());
                account.info.nonce = 0;
            }
            JournalEntry::StorageWarmed { address, key } => {
                state
                    .get_mut(&address)
                    .unwrap()
                    .storage
                    .get_mut(&key)
                    .unwrap()
                    .mark_cold();
            }
            JournalEntry::StorageChanged {
                address,
                key,
                had_value,
            } => {
                state
                    .get_mut(&address)
                    .unwrap()
                    .storage
                    .get_mut(&key)
                    .unwrap()
                    .present_value = had_value;
            }
            JournalEntry::TransientStorageChange {
                address,
                key,
                had_value,
            } => {
                let tkey = (address, key);
                if had_value.is_zero() {
                    // if previous value is zero, remove it
                    transient_storage.remove(&tkey);
                } else {
                    // if not zero, reinsert old value to transient storage.
                    transient_storage.insert(tkey, had_value);
                }
            }
            JournalEntry::CodeChange { address } => {
                let acc = state.get_mut(&address).unwrap();
                acc.info.code_hash = KECCAK_EMPTY;
                acc.info.code = None;
            }
        }
    }
}

impl<DB> JournaledState<DB> {
    /// Initialize a new JournaledState from JournalInit with a database
    pub fn from_init(init: &JournalInit, database: DB) -> Self {
        Self {
            database,
            state: init.state.clone(),
            transient_storage: init.transient_storage.clone(),
            logs: init.logs.clone(),
            depth: init.depth,
            journal: init.journal.clone(),
            spec: init.spec,
            warm_preloaded_addresses: init.warm_preloaded_addresses.clone(),
            precompiles: init.precompiles.clone(),
        }
    }
}
