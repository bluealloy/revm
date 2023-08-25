use super::{
    changes::StateChangeset, reverts::AccountInfoRevert, AccountRevert, AccountStatus,
    BundleAccount, RevertToSlot, StateReverts, TransitionState,
};
use rayon::slice::ParallelSliceMut;
use revm_interpreter::primitives::{
    hash_map::{self, Entry},
    AccountInfo, Bytecode, HashMap, StorageSlot, B160, B256, U256,
};

/// Bundle state contain only values that got changed
///
/// For every account it contains both original and present state.
/// This is needed to decide if there were any changes to the account.
///
/// Reverts and created when TransitionState is applied to BundleState.
/// And can be used to revert BundleState to the state before transition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BundleState {
    /// Account state.
    pub state: HashMap<B160, BundleAccount>,
    /// All created contracts in this block.
    pub contracts: HashMap<B256, Bytecode>,
    /// Changes to revert.
    ///
    /// Note: Inside vector is *not* sorted by address.
    /// But it is unique by address.
    pub reverts: Vec<Vec<(B160, AccountRevert)>>,
}

impl Default for BundleState {
    fn default() -> Self {
        Self {
            state: HashMap::new(),
            reverts: Vec::new(),
            contracts: HashMap::new(),
        }
    }
}

impl BundleState {
    /// Return reference to the state.
    pub fn state(&self) -> &HashMap<B160, BundleAccount> {
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

    /// Create it with new and old values of both Storage and AccountInfo.
    pub fn new(
        state: impl IntoIterator<
            Item = (
                B160,
                Option<AccountInfo>,
                Option<AccountInfo>,
                HashMap<U256, (U256, U256)>,
            ),
        >,
        reverts: impl IntoIterator<
            Item = impl IntoIterator<
                Item = (
                    B160,
                    Option<Option<AccountInfo>>,
                    impl IntoIterator<Item = (U256, U256)>,
                ),
            >,
        >,
        contracts: impl IntoIterator<Item = (B256, Bytecode)>,
    ) -> Self {
        // Create state from iterator.
        let state = state
            .into_iter()
            .map(|(address, original, present, storage)| {
                (
                    address,
                    BundleAccount::new(
                        original,
                        present,
                        storage
                            .into_iter()
                            .map(|(k, (o_val, p_val))| (k, StorageSlot::new_changed(o_val, p_val)))
                            .collect(),
                        AccountStatus::Changed,
                    ),
                )
            })
            .collect();

        // Create reverts from iterator.
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
                        (
                            address,
                            AccountRevert {
                                account,
                                storage: storage
                                    .into_iter()
                                    .map(|(k, v)| (k, RevertToSlot::Some(v)))
                                    .collect(),
                                previous_status: AccountStatus::Changed,
                                wipe_storage: false,
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self {
            state,
            contracts: contracts.into_iter().collect(),
            reverts,
        }
    }

    /// Get account from state
    pub fn account(&self, address: &B160) -> Option<&BundleAccount> {
        self.state.get(address)
    }

    /// Get bytecode from state
    pub fn bytecode(&self, hash: &B256) -> Option<Bytecode> {
        self.contracts.get(hash).cloned()
    }

    /// Consume `TransitionState` by applying the changes and creating the reverts
    pub fn apply_block_substate_and_create_reverts(&mut self, transitions: TransitionState) {
        // pessimistically pre-allocate assuming _all_ accounts changed.
        let mut reverts = Vec::with_capacity(transitions.transitions.len());
        for (address, transition) in transitions.transitions.into_iter() {
            // add new contract if it was created/changed.
            if let Some((hash, new_bytecode)) = transition.has_new_contract() {
                self.contracts.insert(hash, new_bytecode.clone());
            }
            // update state and create revert.
            let revert = match self.state.entry(address) {
                hash_map::Entry::Occupied(mut entry) => {
                    // update and create revert if it is present
                    entry.get_mut().update_and_create_revert(transition)
                }
                hash_map::Entry::Vacant(entry) => {
                    // make revert from transition account
                    let present_bundle = transition.present_bundle_account();
                    let revert = transition.create_revert();
                    if revert.is_some() {
                        entry.insert(present_bundle);
                    }
                    revert
                }
            };

            // append revert if present.
            if let Some(revert) = revert {
                reverts.push((address, revert));
            }
        }
        self.reverts.push(reverts);
    }

    /// Return and clear all reverts from BundleState, sort them before returning.
    pub fn take_reverts(&mut self) -> StateReverts {
        let mut state_reverts = StateReverts::with_capacity(self.reverts.len());
        for reverts in self.reverts.drain(..) {
            // pessimistically pre-allocate assuming _all_ accounts changed.
            let mut accounts = Vec::with_capacity(reverts.len());
            let mut storage = Vec::with_capacity(reverts.len());
            for (address, revert_account) in reverts.into_iter() {
                match revert_account.account {
                    AccountInfoRevert::RevertTo(acc) => accounts.push((address, Some(acc))),
                    AccountInfoRevert::DeleteIt => accounts.push((address, None)),
                    AccountInfoRevert::DoNothing => (),
                }
                if revert_account.wipe_storage || !revert_account.storage.is_empty() {
                    let mut account_storage =
                        revert_account.storage.into_iter().collect::<Vec<_>>();
                    account_storage.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
                    storage.push((address, revert_account.wipe_storage, account_storage));
                }
            }
            accounts.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
            state_reverts.accounts.push(accounts);
            state_reverts.storage.push(storage);
        }

        state_reverts
    }

    /// Consume the bundle state and return sorted plain state.
    ///
    /// `omit_changed_check` does not check If account is same as
    /// original state, this assumption can't be made in cases when
    /// we split the bundle state and commit part of it.
    pub fn into_plain_state_sorted(self, omit_changed_check: bool) -> StateChangeset {
        // pessimistically pre-allocate assuming _all_ accounts changed.
        let state_len = self.state.len();
        let mut accounts = Vec::with_capacity(state_len);
        let mut storage = Vec::with_capacity(state_len);

        for (address, account) in self.state {
            // append account info if it is changed.
            let was_destroyed = account.was_destroyed();
            if omit_changed_check || account.is_info_changed() {
                let info = account.info.map(AccountInfo::without_code);
                accounts.push((address, info));
            }

            // append storage changes

            // NOTE: Assumption is that revert is going to remove whole plain storage from
            // database so we can check if plain state was wiped or not.
            let mut account_storage_changed = Vec::with_capacity(account.storage.len());

            for (key, slot) in account.storage {
                // If storage was destroyed that means that storage was wiped.
                // In that case we need to check if present storage value is different then ZERO.
                let destroyed_and_not_zero = was_destroyed && slot.present_value != U256::ZERO;

                // If account is not destroyed check if original values was changed,
                // so we can update it.
                let not_destroyed_and_changed = !was_destroyed && slot.is_changed();

                if omit_changed_check || destroyed_and_not_zero || not_destroyed_and_changed {
                    account_storage_changed.push((key, slot.present_value));
                }
            }

            if !account_storage_changed.is_empty() {
                account_storage_changed.sort_by(|a, b| a.0.cmp(&b.0));
                // append storage changes to account.
                storage.push((
                    address,
                    (account.status.was_destroyed(), account_storage_changed),
                ));
            }
        }

        accounts.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
        storage.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut contracts = self.contracts.into_iter().collect::<Vec<_>>();
        contracts.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));

        StateChangeset {
            accounts,
            storage,
            contracts,
        }
    }

    /// Consume the bundle state and split it into reverts and plain state.
    pub fn into_sorted_plain_state_and_reverts(
        mut self,
        omit_changed_check: bool,
    ) -> (StateChangeset, StateReverts) {
        let reverts = self.take_reverts();
        let plain_state = self.into_plain_state_sorted(omit_changed_check);
        (plain_state, reverts)
    }

    /// Extend the state with state that is build on top of it.
    pub fn extend(&mut self, other: Self) {
        // Extend the state.
        for (address, other) in other.state {
            match self.state.entry(address) {
                hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().extend(other);
                }
                hash_map::Entry::Vacant(entry) => {
                    // just insert if empty
                    entry.insert(other);
                }
            }
        }
        // Contract can be just extended, when counter is introduced we will take into account that.
        self.contracts.extend(other.contracts);
        // Reverts can be just extended
        self.reverts.extend(other.reverts);
    }

    /// This will return detached lower part of reverts
    ///
    /// Note that plain state will stay the same and returned BundleState
    /// will contain only reverts and will be considered broken.
    ///
    /// If given number is greater then number of reverts then None is returned.
    /// Same if given transition number is zero.
    pub fn detach_lower_part_reverts(&mut self, num_of_detachments: usize) -> Option<Self> {
        if num_of_detachments == 0 || num_of_detachments > self.reverts.len() {
            return None;
        }

        // split is done as [0, num) and [num, len].
        let (detach, this) = self.reverts.split_at(num_of_detachments);

        let detached_reverts = detach.to_vec();
        self.reverts = this.to_vec();
        Some(Self {
            reverts: detached_reverts,
            ..Default::default()
        })
    }

    /// Reverse the state changes by N transitions back
    pub fn revert(&mut self, mut transition: usize) {
        if transition == 0 {
            return;
        }

        // revert the state.
        while let Some(reverts) = self.reverts.pop() {
            for (address, revert_account) in reverts.into_iter() {
                if let Entry::Occupied(mut entry) = self.state.entry(address) {
                    if entry.get_mut().revert(revert_account) {
                        entry.remove();
                    }
                } else {
                    unreachable!("Account {address:?} {revert_account:?} for revert should exist");
                }
            }
            transition -= 1;
            if transition == 0 {
                // break the loop.
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::StorageWithOriginalValues, TransitionAccount};
    use revm_interpreter::primitives::KECCAK_EMPTY;

    #[test]
    fn transition_states() {
        // dummy data
        let address = B160([0x01; 20]);
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
        bundle_state.apply_block_substate_and_create_reverts(TransitionState::single(
            address,
            transition.clone(),
        ));
    }

    fn account1() -> B160 {
        [0x60; 20].into()
    }

    fn account2() -> B160 {
        [0x61; 20].into()
    }

    fn slot() -> U256 {
        U256::from(5)
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
                    HashMap::from([(slot(), (U256::from(0), U256::from(10)))]),
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
                (account1(), Some(None), vec![(slot(), U256::from(0))]),
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
                HashMap::from([(slot(), (U256::from(0), U256::from(15)))]),
            )],
            vec![vec![(
                account1(),
                Some(Some(AccountInfo {
                    nonce: 1,
                    balance: U256::from(10),
                    code_hash: KECCAK_EMPTY,
                    code: None,
                })),
                vec![(slot(), U256::from(10))],
            )]],
            vec![],
        )
    }

    #[test]
    fn sanity_path() {
        let bundle1 = test_bundle1();
        let bundle2 = test_bundle2();

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
}
