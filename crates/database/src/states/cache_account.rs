use super::{
    plain_account::PlainStorage, AccountStatus, BundleAccount, PlainAccount,
    StorageWithOriginalValues, TransitionAccount,
};
use primitives::{HashMap, StorageKey, StorageValue, U256};
use state::{Account, AccountExtension, AccountInfo, EvmStorage};
use std::borrow::Cow;

/// Cache account contains plain state that gets updated
/// at every transaction when evm output is applied to CacheState.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheAccount<EXT: AccountExtension = ()> {
    /// Account information and storage, if account exists.
    pub account: Option<PlainAccount<EXT>>,
    /// Account status flags.
    pub status: AccountStatus,
}

impl<EXT: AccountExtension> From<BundleAccount<EXT>> for CacheAccount<EXT> {
    fn from(account: BundleAccount<EXT>) -> Self {
        CacheAccount::from(&account)
    }
}

impl<EXT: AccountExtension> From<&BundleAccount<EXT>> for CacheAccount<EXT> {
    fn from(account: &BundleAccount<EXT>) -> Self {
        let storage = account
            .storage
            .iter()
            .map(|(k, v)| (*k, v.present_value))
            .collect();
        let plain_account = account
            .account_info()
            .map(|info| PlainAccount { info, storage });
        Self {
            account: plain_account,
            status: account.status,
        }
    }
}

impl<EXT: AccountExtension> CacheAccount<EXT> {
    /// Creates new account that is loaded from database.
    pub const fn new_loaded(info: AccountInfo<EXT>, storage: PlainStorage) -> Self {
        Self {
            account: Some(PlainAccount { info, storage }),
            status: AccountStatus::Loaded,
        }
    }

    /// Creates new account that is loaded empty from database.
    pub fn new_loaded_empty_eip161(storage: PlainStorage) -> Self {
        Self {
            account: Some(PlainAccount::new_empty_with_storage(storage)),
            status: AccountStatus::LoadedEmptyEIP161,
        }
    }

    /// Loaded not existing account.
    pub const fn new_loaded_not_existing() -> Self {
        Self {
            account: None,
            status: AccountStatus::LoadedNotExisting,
        }
    }

    /// Creates new account that is newly created.
    pub const fn new_newly_created(info: AccountInfo<EXT>, storage: PlainStorage) -> Self {
        Self {
            account: Some(PlainAccount { info, storage }),
            status: AccountStatus::InMemoryChange,
        }
    }

    /// Creates account that is destroyed.
    pub const fn new_destroyed() -> Self {
        Self {
            account: None,
            status: AccountStatus::Destroyed,
        }
    }

    /// Creates changed account.
    pub const fn new_changed(info: AccountInfo<EXT>, storage: PlainStorage) -> Self {
        Self {
            account: Some(PlainAccount { info, storage }),
            status: AccountStatus::Changed,
        }
    }

    /// Returns true if account is some.
    pub const fn is_some(&self) -> bool {
        matches!(
            self.status,
            AccountStatus::Changed
                | AccountStatus::InMemoryChange
                | AccountStatus::DestroyedChanged
                | AccountStatus::Loaded
                | AccountStatus::LoadedEmptyEIP161
        )
    }

    /// Returns storage slot if it exists.
    pub fn storage_slot(&self, slot: StorageKey) -> Option<StorageValue> {
        self.account
            .as_ref()
            .and_then(|a| a.storage.get(&slot).cloned())
    }

    /// Fetches account info if it exists.
    pub fn account_info(&self) -> Option<AccountInfo<EXT>> {
        self.account.as_ref().map(|a| a.info.clone())
    }

    /// Dissolves account into components.
    pub fn into_components(self) -> (Option<(AccountInfo<EXT>, PlainStorage)>, AccountStatus) {
        (self.account.map(|a| a.into_components()), self.status)
    }

    /// Touch empty account, related to EIP-161 state clear.
    ///
    /// This account returns the Transition that is used to create the BundleState.
    pub fn touch_empty_eip161<'a>(
        &mut self,
    ) -> Option<TransitionAccount<Option<Cow<'a, EvmStorage>>, EXT>> {
        let previous_status = self.status;

        // Set account to None.
        let previous_info = self.account.take().map(|acc| acc.info);

        // Set account state to Destroyed as we need to clear the storage if it exist.
        self.status = self.status.on_touched_empty_post_eip161();

        if matches!(
            previous_status,
            AccountStatus::LoadedNotExisting
                | AccountStatus::Destroyed
                | AccountStatus::DestroyedAgain
        ) {
            None
        } else {
            Some(TransitionAccount {
                info: None,
                status: self.status,
                previous_info,
                previous_status,
                storage: None,
                storage_was_destroyed: true,
            })
        }
    }

    /// Consumes self and make account as destroyed.
    ///
    /// Sets account as None and set status to Destroyer or DestroyedAgain.
    pub fn selfdestruct<'a>(
        &mut self,
    ) -> Option<TransitionAccount<Option<Cow<'a, EvmStorage>>, EXT>> {
        // Account should be None after selfdestruct so we can take it.
        let previous_info = self.account.take().map(|a| a.info);
        let previous_status = self.status;

        self.status = self.status.on_selfdestructed();

        if previous_status == AccountStatus::LoadedNotExisting {
            None
        } else {
            Some(TransitionAccount {
                info: None,
                status: self.status,
                previous_info,
                previous_status,
                storage: None,
                storage_was_destroyed: true,
            })
        }
    }

    /// Newly created account.
    pub fn newly_created<'a>(
        &mut self,
        account: Cow<'a, Account<EXT>>,
    ) -> TransitionAccount<Option<Cow<'a, EvmStorage>>, EXT> {
        let previous_status = self.status;
        let previous_info = self.account.take().map(|a| a.info);

        let new_bundle_storage = account
            .storage
            .iter()
            .filter_map(|(k, s)| s.is_changed().then_some((*k, s.present_value)))
            .collect();

        self.status = self.status.on_created();
        let (info, storage) = match account {
            Cow::Borrowed(account) => (account.info.clone(), Cow::Borrowed(&account.storage)),
            Cow::Owned(account) => (account.info, Cow::Owned(account.storage)),
        };
        let transition_account = TransitionAccount {
            info: Some(info.clone()),
            status: self.status,
            previous_status,
            previous_info,
            storage: Some(storage),
            storage_was_destroyed: false,
        };
        self.account = Some(PlainAccount {
            info,
            storage: new_bundle_storage,
        });
        transition_account
    }

    /// Increment balance by `balance` amount. Assume that balance will not
    /// overflow or be zero.
    ///
    /// Note: Only if balance is zero we would return None as no transition would be made.
    pub fn increment_balance(
        &mut self,
        balance: u128,
    ) -> Option<TransitionAccount<StorageWithOriginalValues, EXT>> {
        if balance == 0 {
            return None;
        }
        let (_, transition) = self.account_info_change(|info| {
            info.balance = info.balance.saturating_add(U256::from(balance));
        });
        Some(transition)
    }

    fn account_info_change<T, F: FnOnce(&mut AccountInfo<EXT>) -> T>(
        &mut self,
        change: F,
    ) -> (T, TransitionAccount<StorageWithOriginalValues, EXT>) {
        let previous_status = self.status;
        let previous_info = self.account_info();
        let mut account = self.account.take().unwrap_or_default();
        let output = change(&mut account.info);
        self.account = Some(account);

        let had_no_nonce_and_code = previous_info
            .as_ref()
            .map(AccountInfo::has_no_code_and_nonce)
            .unwrap_or_default();
        self.status = self.status.on_changed(had_no_nonce_and_code);

        (
            output,
            TransitionAccount {
                info: self.account_info(),
                status: self.status,
                previous_info,
                previous_status,
                storage: HashMap::default(),
                storage_was_destroyed: false,
            },
        )
    }

    /// Drain balance from account and return drained amount and transition.
    ///
    /// Used for DAO hardfork transition.
    pub fn drain_balance(&mut self) -> (u128, TransitionAccount<StorageWithOriginalValues, EXT>) {
        self.account_info_change(|info| {
            let output = info.balance;
            info.balance = U256::ZERO;
            output.try_into().unwrap()
        })
    }

    /// Updates the account with new information and storage changes.
    ///
    /// Merges the provided storage values with the existing storage and updates the account status.
    pub fn change<'a>(
        &mut self,
        account: Cow<'a, Account<EXT>>,
    ) -> TransitionAccount<Option<Cow<'a, EvmStorage>>, EXT> {
        let previous_status = self.status;
        let (previous_info, mut this_storage) = if let Some(account) = self.account.take() {
            (Some(account.info), account.storage)
        } else {
            (None, Default::default())
        };

        let (info, storage) = match account {
            Cow::Borrowed(account) => (account.info.clone(), Cow::Borrowed(&account.storage)),
            Cow::Owned(account) => (account.info, Cow::Owned(account.storage)),
        };

        this_storage.extend(
            storage
                .iter()
                .filter_map(|(k, s)| s.is_changed().then_some((*k, s.present_value))),
        );
        let changed_account = PlainAccount {
            info,
            storage: this_storage,
        };

        let had_no_nonce_and_code = previous_info
            .as_ref()
            .map(AccountInfo::has_no_code_and_nonce)
            .unwrap_or_default();
        self.status = self.status.on_changed(had_no_nonce_and_code);
        self.account = Some(changed_account);

        TransitionAccount {
            info: self.account.as_ref().map(|a| a.info.clone()),
            status: self.status,
            previous_info,
            previous_status,
            storage: Some(storage),
            storage_was_destroyed: false,
        }
    }
}
