use super::{
    changes::{PlainStorageChangeset, StateChangeset},
    reverts::{AccountInfoRevert, Reverts},
    AccountRevert, AccountStatus, BundleAccount, PlainStateReverts, RevertToSlot, StorageSlot,
    TransitionState,
};
use core::{mem, ops::RangeInclusive};
use revm_interpreter::primitives::{
    hash_map::{self, Entry},
    AccountInfo, Address, Bytecode, HashMap, HashSet, B256, KECCAK_EMPTY, U256,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};

/// This builder is used to help to facilitate the initialization of `BundleState` struct
#[derive(Debug)]
pub struct BundleBuilder {
    states: HashSet<Address>,
    state_original: HashMap<Address, AccountInfo>,
    state_present: HashMap<Address, AccountInfo>,
    state_storage: HashMap<Address, HashMap<U256, (U256, U256)>>,

    reverts: BTreeSet<(u64, Address)>,
    revert_range: RangeInclusive<u64>,
    revert_account: HashMap<(u64, Address), Option<Option<AccountInfo>>>,
    revert_storage: HashMap<(u64, Address), Vec<(U256, U256)>>,

    contracts: HashMap<B256, Bytecode>,
}

/// Option for [`BundleState`] when converting it to the plain state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OriginalValuesKnown {
    /// Check changed with original values that [BundleState] has.
    ///
    /// If we don't expect parent blocks to be committed or unwinded from database, this option
    /// should be used.
    Yes,
    /// Don't check original values, see the implementation of [BundleState::into_plain_state] for
    /// more info.
    ///
    /// If the Bundle can be split or extended, we would not be sure about original values, in that
    /// case this option should be used.
    No,
}
impl OriginalValuesKnown {
    /// Original value is not known for sure.
    pub fn is_not_known(&self) -> bool {
        matches!(self, Self::No)
    }
}

impl Default for BundleBuilder {
    fn default() -> Self {
        BundleBuilder {
            states: HashSet::new(),
            state_original: HashMap::new(),
            state_present: HashMap::new(),
            state_storage: HashMap::new(),
            reverts: BTreeSet::new(),
            revert_range: 0..=0,
            revert_account: HashMap::new(),
            revert_storage: HashMap::new(),
            contracts: HashMap::new(),
        }
    }
}

impl BundleBuilder {
    /// Create builder instance
    ///
    /// `revert_range` indicates the size of BundleState `reverts` field
    pub fn new(revert_range: RangeInclusive<u64>) -> Self {
        BundleBuilder {
            revert_range,
            ..Default::default()
        }
    }

    /// Apply a transformation to the builder.
    pub fn apply<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }

    /// Apply a mutable transformation to the builder.
    pub fn apply_mut<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        f(self);
        self
    }

    /// Collect address info of BundleState state
    pub fn state_address(mut self, address: Address) -> Self {
        self.set_state_address(address);
        self
    }

    /// Collect account info of BundleState state
    pub fn state_original_account_info(mut self, address: Address, original: AccountInfo) -> Self {
        self.set_state_original_account_info(address, original);
        self
    }

    /// Collect account info of BundleState state
    pub fn state_present_account_info(mut self, address: Address, present: AccountInfo) -> Self {
        self.set_state_present_account_info(address, present);
        self
    }

    /// Collect storage info of BundleState state
    pub fn state_storage(mut self, address: Address, storage: HashMap<U256, (U256, U256)>) -> Self {
        self.set_state_storage(address, storage);
        self
    }

    /// Collect address info of BundleState reverts
    ///
    /// `block_number` must respect `revert_range`, or the input
    /// will be ignored during the final build process
    pub fn revert_address(mut self, block_number: u64, address: Address) -> Self {
        self.set_revert_address(block_number, address);
        self
    }

    /// Collect account info of BundleState reverts
    ///
    /// `block_number` must respect `revert_range`, or the input
    /// will be ignored during the final build process
    pub fn revert_account_info(
        mut self,
        block_number: u64,
        address: Address,
        account: Option<Option<AccountInfo>>,
    ) -> Self {
        self.set_revert_account_info(block_number, address, account);
        self
    }

    /// Collect storage info of BundleState reverts
    ///
    /// `block_number` must respect `revert_range`, or the input
    /// will be ignored during the final build process
    pub fn revert_storage(
        mut self,
        block_number: u64,
        address: Address,
        storage: Vec<(U256, U256)>,
    ) -> Self {
        self.set_revert_storage(block_number, address, storage);
        self
    }

    /// Collect contracts info
    pub fn contract(mut self, address: B256, bytecode: Bytecode) -> Self {
        self.set_contract(address, bytecode);
        self
    }

    /// Set address info of BundleState state.
    pub fn set_state_address(&mut self, address: Address) -> &mut Self {
        self.states.insert(address);
        self
    }

    /// Set original account info of BundleState state.
    pub fn set_state_original_account_info(
        &mut self,
        address: Address,
        original: AccountInfo,
    ) -> &mut Self {
        self.states.insert(address);
        self.state_original.insert(address, original);
        self
    }

    /// Set present account info of BundleState state.
    pub fn set_state_present_account_info(
        &mut self,
        address: Address,
        present: AccountInfo,
    ) -> &mut Self {
        self.states.insert(address);
        self.state_present.insert(address, present);
        self
    }

    /// Set storage info of BundleState state.
    pub fn set_state_storage(
        &mut self,
        address: Address,
        storage: HashMap<U256, (U256, U256)>,
    ) -> &mut Self {
        self.states.insert(address);
        self.state_storage.insert(address, storage);
        self
    }

    /// Set address info of BundleState reverts.
    pub fn set_revert_address(&mut self, block_number: u64, address: Address) -> &mut Self {
        self.reverts.insert((block_number, address));
        self
    }

    /// Set account info of BundleState reverts.
    pub fn set_revert_account_info(
        &mut self,
        block_number: u64,
        address: Address,
        account: Option<Option<AccountInfo>>,
    ) -> &mut Self {
        self.reverts.insert((block_number, address));
        self.revert_account.insert((block_number, address), account);
        self
    }

    /// Set storage info of BundleState reverts.
    pub fn set_revert_storage(
        &mut self,
        block_number: u64,
        address: Address,
        storage: Vec<(U256, U256)>,
    ) -> &mut Self {
        self.reverts.insert((block_number, address));
        self.revert_storage.insert((block_number, address), storage);
        self
    }

    /// Set contracts info.
    pub fn set_contract(&mut self, address: B256, bytecode: Bytecode) -> &mut Self {
        self.contracts.insert(address, bytecode);
        self
    }

    /// Create `BundleState` instance based on collected information
    pub fn build(mut self) -> BundleState {
        let mut state_size = 0;
        let state = self
            .states
            .into_iter()
            .map(|address| {
                let storage = self
                    .state_storage
                    .remove(&address)
                    .map(|s| {
                        s.into_iter()
                            .map(|(k, (o_val, p_val))| (k, StorageSlot::new_changed(o_val, p_val)))
                            .collect()
                    })
                    .unwrap_or_default();
                let bundle_account = BundleAccount::new(
                    self.state_original.remove(&address),
                    self.state_present.remove(&address),
                    storage,
                    AccountStatus::Changed,
                );
                state_size += bundle_account.size_hint();
                (address, bundle_account)
            })
            .collect();

        let mut reverts_size = 0;
        let mut reverts_map = BTreeMap::new();
        for block_number in self.revert_range {
            reverts_map.insert(block_number, Vec::new());
        }
        self.reverts
            .into_iter()
            .for_each(|(block_number, address)| {
                let account = match self
                    .revert_account
                    .remove(&(block_number, address))
                    .unwrap_or_default()
                {
                    Some(Some(account)) => AccountInfoRevert::RevertTo(account),
                    Some(None) => AccountInfoRevert::DeleteIt,
                    None => AccountInfoRevert::DoNothing,
                };
                let storage = self
                    .revert_storage
                    .remove(&(block_number, address))
                    .map(|s| {
                        s.into_iter()
                            .map(|(k, v)| (k, RevertToSlot::Some(v)))
                            .collect()
                    })
                    .unwrap_or_default();
                let account_revert = AccountRevert {
                    account,
                    storage,
                    previous_status: AccountStatus::Changed,
                    wipe_storage: false,
                };

                if reverts_map.contains_key(&block_number) {
                    reverts_size += account_revert.size_hint();
                    reverts_map
                        .entry(block_number)
                        .or_insert(Vec::new())
                        .push((address, account_revert));
                }
            });

        BundleState {
            state,
            contracts: self.contracts,
            reverts: Reverts::new(reverts_map.into_values().collect()),
            state_size,
            reverts_size,
        }
    }

    /// Getter for `states` field
    pub fn get_states(&self) -> &HashSet<Address> {
        &self.states
    }

    /// Mutable getter for `states` field
    pub fn get_states_mut(&mut self) -> &mut HashSet<Address> {
        &mut self.states
    }

    /// Mutable getter for `state_original` field
    pub fn get_state_original_mut(&mut self) -> &mut HashMap<Address, AccountInfo> {
        &mut self.state_original
    }

    /// Mutable getter for `state_present` field
    pub fn get_state_present_mut(&mut self) -> &mut HashMap<Address, AccountInfo> {
        &mut self.state_present
    }

    /// Mutable getter for `state_storage` field
    pub fn get_state_storage_mut(&mut self) -> &mut HashMap<Address, HashMap<U256, (U256, U256)>> {
        &mut self.state_storage
    }

    /// Mutable getter for `reverts` field
    pub fn get_reverts_mut(&mut self) -> &mut BTreeSet<(u64, Address)> {
        &mut self.reverts
    }

    /// Mutable getter for `revert_range` field
    pub fn get_revert_range_mut(&mut self) -> &mut RangeInclusive<u64> {
        &mut self.revert_range
    }

    /// Mutable getter for `revert_account` field
    pub fn get_revert_account_mut(
        &mut self,
    ) -> &mut HashMap<(u64, Address), Option<Option<AccountInfo>>> {
        &mut self.revert_account
    }

    /// Mutable getter for `revert_storage` field
    pub fn get_revert_storage_mut(&mut self) -> &mut HashMap<(u64, Address), Vec<(U256, U256)>> {
        &mut self.revert_storage
    }

    /// Mutable getter for `contracts` field
    pub fn get_contracts_mut(&mut self) -> &mut HashMap<B256, Bytecode> {
        &mut self.contracts
    }
}

/// Bundle retention policy for applying substate to the bundle.
#[derive(Debug)]
pub enum BundleRetention {
    /// Only plain state is updated.
    PlainState,
    /// Both, plain state and reverts, are retained
    Reverts,
}

impl BundleRetention {
    /// Returns `true` if reverts should be retained.
    pub fn includes_reverts(&self) -> bool {
        matches!(self, Self::Reverts)
    }
}

/// Bundle state contain only values that got changed
///
/// For every account it contains both original and present state.
/// This is needed to decide if there were any changes to the account.
///
/// Reverts and created when TransitionState is applied to BundleState.
/// And can be used to revert BundleState to the state before transition.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BundleState {
    /// Account state.
    pub state: HashMap<Address, BundleAccount>,
    /// All created contracts in this block.
    pub contracts: HashMap<B256, Bytecode>,
    /// Changes to revert.
    ///
    /// Note: Inside vector is *not* sorted by address.
    /// But it is unique by address.
    pub reverts: Reverts,
    /// The size of the plain state in the bundle state.
    pub state_size: usize,
    /// The size of reverts in the bundle state.
    pub reverts_size: usize,
}

impl BundleState {
    /// Return builder instance for further manipulation
    pub fn builder(revert_range: RangeInclusive<u64>) -> BundleBuilder {
        BundleBuilder::new(revert_range)
    }

    /// Create it with new and old values of both Storage and AccountInfo.
    pub fn new(
        state: impl IntoIterator<
            Item = (
                Address,
                Option<AccountInfo>,
                Option<AccountInfo>,
                HashMap<U256, (U256, U256)>,
            ),
        >,
        reverts: impl IntoIterator<
            Item = impl IntoIterator<
                Item = (
                    Address,
                    Option<Option<AccountInfo>>,
                    impl IntoIterator<Item = (U256, U256)>,
                ),
            >,
        >,
        contracts: impl IntoIterator<Item = (B256, Bytecode)>,
    ) -> Self {
        // Create state from iterator.
        let mut state_size = 0;
        let state = state
            .into_iter()
            .map(|(address, original, present, storage)| {
                let account = BundleAccount::new(
                    original,
                    present,
                    storage
                        .into_iter()
                        .map(|(k, (o_val, p_val))| (k, StorageSlot::new_changed(o_val, p_val)))
                        .collect(),
                    AccountStatus::Changed,
                );
                state_size += account.size_hint();
                (address, account)
            })
            .collect();

        // Create reverts from iterator.
        let mut reverts_size = 0;
        let reverts = reverts
            .into_iter()
            .map(|block_reverts| {
                block_reverts
                    .into_iter()
                    .map(|(address, account, storage)| {
                        let account = match account {
                            Some(Some(account)) => AccountInfoRevert::RevertTo(account),
                            Some(None) => AccountInfoRevert::DeleteIt,
                            None => AccountInfoRevert::DoNothing,
                        };
                        let revert = AccountRevert {
                            account,
                            storage: storage
                                .into_iter()
                                .map(|(k, v)| (k, RevertToSlot::Some(v)))
                                .collect(),
                            previous_status: AccountStatus::Changed,
                            wipe_storage: false,
                        };
                        reverts_size += revert.size_hint();
                        (address, revert)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self {
            state,
            contracts: contracts.into_iter().collect(),
            reverts: Reverts::new(reverts),
            state_size,
            reverts_size,
        }
    }

    /// Returns the approximate size of changes in the bundle state.
    /// The estimation is not precise, because the information about the number of
    /// destroyed entries that need to be removed is not accessible to the bundle state.
    pub fn size_hint(&self) -> usize {
        self.state_size + self.reverts_size + self.contracts.len()
    }

    /// Return reference to the state.
    pub fn state(&self) -> &HashMap<Address, BundleAccount> {
        &self.state
    }

    /// Is bundle state empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return number of changed accounts.
    pub fn len(&self) -> usize {
        self.state.len()
    }

    /// Get account from state
    pub fn account(&self, address: &Address) -> Option<&BundleAccount> {
        self.state.get(address)
    }

    /// Get bytecode from state
    pub fn bytecode(&self, hash: &B256) -> Option<Bytecode> {
        self.contracts.get(hash).cloned()
    }

    /// Consume [`TransitionState`] by applying the changes and creating the
    /// reverts.
    ///
    /// If [BundleRetention::includes_reverts] is `true`, then the reverts will
    /// be retained.
    pub fn apply_transitions_and_create_reverts(
        &mut self,
        transitions: TransitionState,
        retention: BundleRetention,
    ) {
        let include_reverts = retention.includes_reverts();
        // pessimistically pre-allocate assuming _all_ accounts changed.
        let reverts_capacity = if include_reverts {
            transitions.transitions.len()
        } else {
            0
        };
        let mut reverts = Vec::with_capacity(reverts_capacity);

        for (address, transition) in transitions.transitions.into_iter() {
            // add new contract if it was created/changed.
            if let Some((hash, new_bytecode)) = transition.has_new_contract() {
                self.contracts.insert(hash, new_bytecode.clone());
            }
            // update state and create revert.
            let revert = match self.state.entry(address) {
                hash_map::Entry::Occupied(mut entry) => {
                    let entry = entry.get_mut();
                    self.state_size -= entry.size_hint();
                    // update and create revert if it is present
                    let revert = entry.update_and_create_revert(transition);
                    // update the state size
                    self.state_size += entry.size_hint();
                    revert
                }
                hash_map::Entry::Vacant(entry) => {
                    // make revert from transition account
                    let present_bundle = transition.present_bundle_account();
                    let revert = transition.create_revert();
                    if revert.is_some() {
                        self.state_size += present_bundle.size_hint();
                        entry.insert(present_bundle);
                    }
                    revert
                }
            };

            // append revert if present.
            if let Some(revert) = revert.filter(|_| include_reverts) {
                self.reverts_size += revert.size_hint();
                reverts.push((address, revert));
            }
        }

        self.reverts.push(reverts);
    }

    /// Generate a [`StateChangeset`] from the bundle state without consuming
    /// it.
    pub fn to_plain_state(&self, is_value_known: OriginalValuesKnown) -> StateChangeset {
        // pessimistically pre-allocate assuming _all_ accounts changed.
        let state_len = self.state.len();
        let mut accounts = Vec::with_capacity(state_len);
        let mut storage = Vec::with_capacity(state_len);

        for (address, account) in self.state.iter() {
            // append account info if it is changed.
            let was_destroyed = account.was_destroyed();
            if is_value_known.is_not_known() || account.is_info_changed() {
                let info = account.info.as_ref().map(AccountInfo::copy_without_code);
                accounts.push((*address, info));
            }

            // append storage changes

            // NOTE: Assumption is that revert is going to remove whole plain storage from
            // database so we can check if plain state was wiped or not.
            let mut account_storage_changed = Vec::with_capacity(account.storage.len());

            for (key, slot) in account.storage.iter().map(|(k, v)| (*k, *v)) {
                // If storage was destroyed that means that storage was wiped.
                // In that case we need to check if present storage value is different then ZERO.
                let destroyed_and_not_zero = was_destroyed && !slot.present_value.is_zero();

                // If account is not destroyed check if original values was changed,
                // so we can update it.
                let not_destroyed_and_changed = !was_destroyed && slot.is_changed();

                if is_value_known.is_not_known()
                    || destroyed_and_not_zero
                    || not_destroyed_and_changed
                {
                    account_storage_changed.push((key, slot.present_value));
                }
            }

            if !account_storage_changed.is_empty() || was_destroyed {
                // append storage changes to account.
                storage.push(PlainStorageChangeset {
                    address: *address,
                    wipe_storage: was_destroyed,
                    storage: account_storage_changed,
                });
            }
        }

        let contracts = self
            .contracts
            .iter()
            // remove empty bytecodes
            .filter(|(b, _)| **b != KECCAK_EMPTY)
            .map(|(b, code)| (*b, code.clone()))
            .collect::<Vec<_>>();
        StateChangeset {
            accounts,
            storage,
            contracts,
        }
    }

    /// Convert the bundle state into a [`StateChangeset`].
    #[deprecated = "Use `to_plain_state` instead"]
    pub fn into_plain_state(self, is_value_known: OriginalValuesKnown) -> StateChangeset {
        self.to_plain_state(is_value_known)
    }

    /// Generate a [`StateChangeset`] and [`PlainStateReverts`] from the bundle
    /// state.
    pub fn to_plain_state_and_reverts(
        &self,
        is_value_known: OriginalValuesKnown,
    ) -> (StateChangeset, PlainStateReverts) {
        (
            self.to_plain_state(is_value_known),
            self.reverts.to_plain_state_reverts(),
        )
    }

    /// Consume the bundle state and split it into a [`StateChangeset`] and a
    /// [`PlainStateReverts`].
    #[deprecated = "Use `to_plain_state_and_reverts` instead"]
    pub fn into_plain_state_and_reverts(
        self,
        is_value_known: OriginalValuesKnown,
    ) -> (StateChangeset, PlainStateReverts) {
        self.to_plain_state_and_reverts(is_value_known)
    }

    /// Extend the bundle with other state
    ///
    /// Update the `other` state only if `other` is not flagged as destroyed.
    pub fn extend_state(&mut self, other_state: HashMap<Address, BundleAccount>) {
        for (address, other_account) in other_state {
            match self.state.entry(address) {
                hash_map::Entry::Occupied(mut entry) => {
                    let this = entry.get_mut();
                    self.state_size -= this.size_hint();

                    // if other was destroyed. replace `this` storage with
                    // the `other one.
                    if other_account.was_destroyed() {
                        this.storage = other_account.storage;
                    } else {
                        // otherwise extend this storage with other
                        for (key, storage_slot) in other_account.storage {
                            // update present value or insert storage slot.
                            this.storage
                                .entry(key)
                                .or_insert(storage_slot)
                                .present_value = storage_slot.present_value;
                        }
                    }
                    this.info = other_account.info;
                    this.status.transition(other_account.status);

                    // Update the state size
                    self.state_size += this.size_hint();
                }
                hash_map::Entry::Vacant(entry) => {
                    // just insert if empty
                    self.state_size += other_account.size_hint();
                    entry.insert(other_account);
                }
            }
        }
    }
    /// Extend the state with state that is build on top of it.
    ///
    /// If storage was wiped in `other` state, copy `this` plain state
    /// and put it inside `other` revert (if there is no duplicates of course).
    ///
    /// If `this` and `other` accounts were both destroyed invalidate second
    /// wipe flag (from `other`). As wiping from database should be done only once
    /// and we already transferred all potentially missing storages to the `other` revert.
    pub fn extend(&mut self, mut other: Self) {
        // iterate over reverts and if its storage is wiped try to add previous bundle
        // state as there is potential missing slots.
        for (address, revert) in other.reverts.iter_mut().flatten() {
            if revert.wipe_storage {
                // If there is wipe storage in `other` revert
                // we need to move storage from present state.
                if let Some(this_account) = self.state.get_mut(address) {
                    // As this account was destroyed inside `other` bundle.
                    // we are fine to wipe/drain this storage and put it inside revert.
                    for (key, value) in this_account.storage.drain() {
                        revert
                            .storage
                            .entry(key)
                            .or_insert(RevertToSlot::Some(value.present_value));
                    }

                    // nullify `other` wipe as primary database wipe is done in `this`.
                    if this_account.was_destroyed() {
                        revert.wipe_storage = false;
                    }
                }
            }

            // Increment reverts size for each of the updated reverts.
            self.reverts_size += revert.size_hint();
        }
        // Extension of state
        self.extend_state(other.state);
        // Contract can be just extended, when counter is introduced we will take into account that.
        self.contracts.extend(other.contracts);
        // Reverts can be just extended
        self.reverts.extend(other.reverts);
    }

    /// Take first N raw reverts from the [BundleState].
    pub fn take_n_reverts(&mut self, reverts_to_take: usize) -> Reverts {
        // split is done as [0, num) and [num, len].
        if reverts_to_take > self.reverts.len() {
            return self.take_all_reverts();
        }
        let (detach, this) = self.reverts.split_at(reverts_to_take);
        let detached_reverts = Reverts::new(detach.to_vec());
        self.reverts_size = this
            .iter()
            .flatten()
            .fold(0, |acc, (_, revert)| acc + revert.size_hint());
        self.reverts = Reverts::new(this.to_vec());
        detached_reverts
    }

    /// Return and clear all reverts from [BundleState]
    pub fn take_all_reverts(&mut self) -> Reverts {
        self.reverts_size = 0;
        core::mem::take(&mut self.reverts)
    }

    /// Reverts the state changes of the latest transition
    ///
    /// Note: This is the same as `BundleState::revert(1)`
    ///
    /// Returns true if the state was reverted.
    pub fn revert_latest(&mut self) -> bool {
        // revert the latest recorded state
        if let Some(reverts) = self.reverts.pop() {
            for (address, revert_account) in reverts.into_iter() {
                self.reverts_size -= revert_account.size_hint();
                match self.state.entry(address) {
                    Entry::Occupied(mut entry) => {
                        let account = entry.get_mut();
                        self.state_size -= account.size_hint();
                        if account.revert(revert_account) {
                            entry.remove();
                        } else {
                            self.state_size += account.size_hint();
                        }
                    }
                    Entry::Vacant(entry) => {
                        // create empty account that we will revert on.
                        // Only place where this account is not existing is if revert is DeleteIt.
                        let mut account = BundleAccount::new(
                            None,
                            None,
                            HashMap::new(),
                            AccountStatus::LoadedNotExisting,
                        );
                        if !account.revert(revert_account) {
                            self.state_size += account.size_hint();
                            entry.insert(account);
                        }
                    }
                }
            }
            return true;
        }

        false
    }

    /// Reverts the state changes by N transitions back.
    ///
    /// See also [Self::revert_latest]
    pub fn revert(&mut self, mut num_transitions: usize) {
        if num_transitions == 0 {
            return;
        }

        while self.revert_latest() {
            num_transitions -= 1;
            if num_transitions == 0 {
                // break the loop.
                break;
            }
        }
    }

    /// Prepends present the state with the given BundleState.
    /// It adds changes from the given state but does not override any existing changes.
    ///
    /// Reverts are not updated.
    pub fn prepend_state(&mut self, mut other: BundleState) {
        // take this bundle
        let this_bundle = mem::take(self);
        // extend other bundle state with this
        other.extend_state(this_bundle.state);
        // extend other contracts
        other.contracts.extend(this_bundle.contracts);
        // swap bundles
        mem::swap(self, &mut other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::StorageWithOriginalValues, TransitionAccount};

    #[test]
    fn transition_states() {
        // dummy data
        let address = Address::new([0x01; 20]);
        let acc1 = AccountInfo {
            balance: U256::from(10),
            nonce: 1,
            code_hash: KECCAK_EMPTY,
            code: None,
        };

        let mut bundle_state = BundleState::default();

        // have transition from loaded to all other states

        let transition = TransitionAccount {
            info: Some(acc1),
            status: AccountStatus::InMemoryChange,
            previous_info: None,
            previous_status: AccountStatus::LoadedNotExisting,
            storage: StorageWithOriginalValues::default(),
            storage_was_destroyed: false,
        };

        // apply first transition
        bundle_state.apply_transitions_and_create_reverts(
            TransitionState::single(address, transition.clone()),
            BundleRetention::Reverts,
        );
    }

    const fn account1() -> Address {
        Address::new([0x60; 20])
    }

    const fn account2() -> Address {
        Address::new([0x61; 20])
    }

    fn slot1() -> U256 {
        U256::from(5)
    }

    fn slot2() -> U256 {
        U256::from(7)
    }

    /// Test bundle one
    fn test_bundle1() -> BundleState {
        // block changes
        BundleState::new(
            vec![
                (
                    account1(),
                    None,
                    Some(AccountInfo {
                        nonce: 1,
                        balance: U256::from(10),
                        code_hash: KECCAK_EMPTY,
                        code: None,
                    }),
                    HashMap::from([
                        (slot1(), (U256::from(0), U256::from(10))),
                        (slot2(), (U256::from(0), U256::from(15))),
                    ]),
                ),
                (
                    account2(),
                    None,
                    Some(AccountInfo {
                        nonce: 1,
                        balance: U256::from(10),
                        code_hash: KECCAK_EMPTY,
                        code: None,
                    }),
                    HashMap::from([]),
                ),
            ],
            vec![vec![
                (
                    account1(),
                    Some(None),
                    vec![(slot1(), U256::from(0)), (slot2(), U256::from(0))],
                ),
                (account2(), Some(None), vec![]),
            ]],
            vec![],
        )
    }

    /// Test bundle two
    fn test_bundle2() -> BundleState {
        // block changes
        BundleState::new(
            vec![(
                account1(),
                None,
                Some(AccountInfo {
                    nonce: 3,
                    balance: U256::from(20),
                    code_hash: KECCAK_EMPTY,
                    code: None,
                }),
                HashMap::from([(slot1(), (U256::from(0), U256::from(15)))]),
            )],
            vec![vec![(
                account1(),
                Some(Some(AccountInfo {
                    nonce: 1,
                    balance: U256::from(10),
                    code_hash: KECCAK_EMPTY,
                    code: None,
                })),
                vec![(slot1(), U256::from(10))],
            )]],
            vec![],
        )
    }

    /// Test bundle three
    fn test_bundle3() -> BundleState {
        BundleState::builder(0..=0)
            .state_present_account_info(
                account1(),
                AccountInfo {
                    nonce: 1,
                    balance: U256::from(10),
                    code_hash: KECCAK_EMPTY,
                    code: None,
                },
            )
            .state_storage(
                account1(),
                HashMap::from([(slot1(), (U256::from(0), U256::from(10)))]),
            )
            .state_address(account2())
            .state_present_account_info(
                account2(),
                AccountInfo {
                    nonce: 1,
                    balance: U256::from(10),
                    code_hash: KECCAK_EMPTY,
                    code: None,
                },
            )
            .revert_address(0, account1())
            .revert_account_info(0, account1(), Some(None))
            .revert_storage(0, account1(), vec![(slot1(), U256::from(0))])
            .revert_account_info(0, account2(), Some(None))
            .build()
    }

    /// Test bundle four
    fn test_bundle4() -> BundleState {
        BundleState::builder(0..=0)
            .state_present_account_info(
                account1(),
                AccountInfo {
                    nonce: 3,
                    balance: U256::from(20),
                    code_hash: KECCAK_EMPTY,
                    code: None,
                },
            )
            .state_storage(
                account1(),
                HashMap::from([(slot1(), (U256::from(0), U256::from(15)))]),
            )
            .revert_address(0, account1())
            .revert_account_info(
                0,
                account1(),
                Some(Some(AccountInfo {
                    nonce: 1,
                    balance: U256::from(10),
                    code_hash: KECCAK_EMPTY,
                    code: None,
                })),
            )
            .revert_storage(0, account1(), vec![(slot1(), U256::from(10))])
            .build()
    }

    fn sanity_path(bundle1: BundleState, bundle2: BundleState) {
        let mut extended = bundle1.clone();
        extended.extend(bundle2.clone());

        let mut reverted = extended.clone();
        // revert zero does nothing.
        reverted.revert(0);
        assert_eq!(reverted, extended);

        // revert by one gives us bundle one.
        reverted.revert(1);
        assert_eq!(reverted, bundle1);

        // reverted by additional one gives us empty bundle.
        reverted.revert(1);
        assert_eq!(reverted, BundleState::default());

        let mut reverted = extended.clone();

        // reverted by bigger number gives us empty bundle
        reverted.revert(10);
        assert_eq!(reverted, BundleState::default());
    }

    #[test]
    fn extend_on_destroyed_values() {
        let base_bundle1 = test_bundle1();
        let base_bundle2 = test_bundle2();

        // test1
        // bundle1 has Destroyed
        // bundle2 has Changed
        // end should be DestroyedChanged.
        let mut b1 = base_bundle1.clone();
        let mut b2 = base_bundle2.clone();
        b1.state.get_mut(&account1()).unwrap().status = AccountStatus::Destroyed;
        b2.state.get_mut(&account1()).unwrap().status = AccountStatus::Changed;
        b1.extend(b2);
        assert_eq!(
            b1.state.get_mut(&account1()).unwrap().status,
            AccountStatus::DestroyedChanged
        );

        // test2
        // bundle1 has Changed
        // bundle2 has Destroyed
        // end should be Destroyed
        let mut b1 = base_bundle1.clone();
        let mut b2 = base_bundle2.clone();
        b1.state.get_mut(&account1()).unwrap().status = AccountStatus::Changed;
        b2.state.get_mut(&account1()).unwrap().status = AccountStatus::Destroyed;
        b2.reverts[0][0].1.wipe_storage = true;
        b1.extend(b2);
        assert_eq!(
            b1.state.get_mut(&account1()).unwrap().status,
            AccountStatus::Destroyed
        );

        // test2 extension
        // revert of b2 should contains plain state of b1.
        let mut revert1 = base_bundle2.reverts[0][0].clone();
        revert1.1.wipe_storage = true;
        revert1
            .1
            .storage
            .insert(slot2(), RevertToSlot::Some(U256::from(15)));

        assert_eq!(
            b1.reverts.as_ref(),
            vec![base_bundle1.reverts[0].clone(), vec![revert1]],
        );

        // test3
        // bundle1 has InMemoryChange
        // bundle2 has Change
        // end should be InMemoryChange.

        let mut b1 = base_bundle1.clone();
        let mut b2 = base_bundle2.clone();
        b1.state.get_mut(&account1()).unwrap().status = AccountStatus::InMemoryChange;
        b2.state.get_mut(&account1()).unwrap().status = AccountStatus::Changed;
        b1.extend(b2);
        assert_eq!(
            b1.state.get_mut(&account1()).unwrap().status,
            AccountStatus::InMemoryChange
        );
    }

    #[test]
    fn test_sanity_path() {
        sanity_path(test_bundle1(), test_bundle2());
        sanity_path(test_bundle3(), test_bundle4());
    }

    #[test]
    fn test_multi_reverts_with_delete() {
        let mut state = BundleBuilder::new(0..=3)
            .revert_address(0, account1())
            .revert_account_info(2, account1(), Some(Some(AccountInfo::default())))
            .revert_account_info(3, account1(), Some(None))
            .build();

        state.revert_latest();
        // state for account one was deleted
        assert_eq!(state.state.get(&account1()), None);

        state.revert_latest();
        // state is set to
        assert_eq!(
            state.state.get(&account1()),
            Some(&BundleAccount::new(
                None,
                Some(AccountInfo::default()),
                HashMap::new(),
                AccountStatus::Changed
            ))
        );
    }

    #[test]
    fn test_revert_capacity() {
        let state = BundleState::builder(0..=3)
            .revert_address(0, account1())
            .revert_address(2, account2())
            .revert_account_info(0, account1(), Some(None))
            .revert_account_info(2, account2(), None)
            .revert_storage(0, account1(), vec![(slot1(), U256::from(10))])
            .build();

        assert_eq!(state.reverts.len(), 4);
        assert_eq!(state.reverts[1], vec![]);
        assert_eq!(state.reverts[3], vec![]);
        assert_eq!(state.reverts[0].len(), 1);
        assert_eq!(state.reverts[2].len(), 1);

        let (addr1, revert1) = &state.reverts[0][0];
        assert_eq!(addr1, &account1());
        assert_eq!(revert1.account, AccountInfoRevert::DeleteIt);

        let (addr2, revert2) = &state.reverts[2][0];
        assert_eq!(addr2, &account2());
        assert_eq!(revert2.account, AccountInfoRevert::DoNothing);
    }

    #[test]
    fn take_reverts() {
        let bundle1 = test_bundle1();
        let bundle2 = test_bundle2();

        let mut extended = bundle1.clone();
        extended.extend(bundle2.clone());

        // check that we have two reverts
        assert_eq!(extended.reverts.len(), 2);

        // take all by big N
        let mut extended2 = extended.clone();
        assert_eq!(extended2.take_n_reverts(100), extended.reverts);

        // take all reverts
        let mut extended2 = extended.clone();
        assert_eq!(extended2.take_all_reverts(), extended.reverts);

        // take zero revert
        let taken_reverts = extended.take_n_reverts(0);
        assert_eq!(taken_reverts, Reverts::default());
        assert_eq!(extended.reverts.len(), 2);

        // take one revert
        let taken_reverts = extended.take_n_reverts(1);
        assert_eq!(taken_reverts, bundle1.reverts);

        // take last revert
        let taken_reverts = extended.take_n_reverts(1);
        assert_eq!(taken_reverts, bundle2.reverts);
    }

    #[test]
    fn prepend_state() {
        let address1 = account1();
        let address2 = account2();

        let account1 = AccountInfo {
            nonce: 1,
            ..Default::default()
        };
        let account1_changed = AccountInfo {
            nonce: 1,
            ..Default::default()
        };
        let account2 = AccountInfo {
            nonce: 1,
            ..Default::default()
        };

        let present_state = BundleState::builder(2..=2)
            .state_present_account_info(address1, account1_changed.clone())
            .build();
        assert_eq!(present_state.reverts.len(), 1);
        let previous_state = BundleState::builder(1..=1)
            .state_present_account_info(address1, account1)
            .state_present_account_info(address2, account2.clone())
            .build();
        assert_eq!(previous_state.reverts.len(), 1);

        let mut test = present_state;

        test.prepend_state(previous_state);

        assert_eq!(test.state.len(), 2);
        // reverts num should stay the same.
        assert_eq!(test.reverts.len(), 1);
        // account1 is not overwritten.
        assert_eq!(
            test.state.get(&address1).unwrap().info,
            Some(account1_changed)
        );
        // account2 got inserted
        assert_eq!(test.state.get(&address2).unwrap().info, Some(account2));
    }

    #[test]
    fn test_getters() {
        let mut builder = BundleBuilder::new(0..=3);

        // Test get_states and get_states_mut
        assert!(builder.get_states().is_empty());
        builder.get_states_mut().insert(account1());
        assert!(builder.get_states().contains(&account1()));

        // Test get_state_original_mut
        assert!(builder.get_state_original_mut().is_empty());
        builder
            .get_state_original_mut()
            .insert(account1(), AccountInfo::default());
        assert!(builder.get_state_original_mut().contains_key(&account1()));

        // Test get_state_present_mut
        assert!(builder.get_state_present_mut().is_empty());
        builder
            .get_state_present_mut()
            .insert(account1(), AccountInfo::default());
        assert!(builder.get_state_present_mut().contains_key(&account1()));

        // Test get_state_storage_mut
        assert!(builder.get_state_storage_mut().is_empty());
        builder
            .get_state_storage_mut()
            .insert(account1(), HashMap::new());
        assert!(builder.get_state_storage_mut().contains_key(&account1()));

        // Test get_reverts_mut
        assert!(builder.get_reverts_mut().is_empty());
        builder.get_reverts_mut().insert((0, account1()));
        assert!(builder.get_reverts_mut().contains(&(0, account1())));

        // Test get_revert_range_mut
        assert_eq!(builder.get_revert_range_mut().clone(), 0..=3);

        // Test get_revert_account_mut
        assert!(builder.get_revert_account_mut().is_empty());
        builder
            .get_revert_account_mut()
            .insert((0, account1()), Some(None));
        assert!(builder
            .get_revert_account_mut()
            .contains_key(&(0, account1())));

        // Test get_revert_storage_mut
        assert!(builder.get_revert_storage_mut().is_empty());
        builder
            .get_revert_storage_mut()
            .insert((0, account1()), vec![(slot1(), U256::from(0))]);
        assert!(builder
            .get_revert_storage_mut()
            .contains_key(&(0, account1())));

        // Test get_contracts_mut
        assert!(builder.get_contracts_mut().is_empty());
        builder
            .get_contracts_mut()
            .insert(B256::default(), Bytecode::default());
        assert!(builder.get_contracts_mut().contains_key(&B256::default()));
    }
}
