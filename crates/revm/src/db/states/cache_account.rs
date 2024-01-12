use super::{
    plain_account::PlainStorage, AccountStatus, BundleAccount, PlainAccount,
    StorageWithOriginalValues, TransitionAccount,
};
use revm_interpreter::primitives::{AccountInfo, U256};
use revm_precompile::HashMap;

/// Cache account contains plain state that gets updated
/// at every transaction when evm output is applied to CacheState.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheAccount {
    pub account: Option<PlainAccount>,
    pub status: AccountStatus,
}

impl From<BundleAccount> for CacheAccount {
    fn from(account: BundleAccount) -> Self {
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

impl CacheAccount {
    /// Create new account that is loaded from database.
    pub fn new_loaded(info: AccountInfo, storage: PlainStorage) -> Self {
        Self {
            account: Some(PlainAccount { info, storage }),
            status: AccountStatus::Loaded,
        }
    }

    /// Create new account that is loaded empty from database.
    pub fn new_loaded_empty_eip161(storage: PlainStorage) -> Self {
        Self {
            account: Some(PlainAccount::new_empty_with_storage(storage)),
            status: AccountStatus::LoadedEmptyEIP161,
        }
    }

    /// Loaded not existing account.
    pub fn new_loaded_not_existing() -> Self {
        Self {
            account: None,
            status: AccountStatus::LoadedNotExisting,
        }
    }

    /// Create new account that is newly created
    pub fn new_newly_created(info: AccountInfo, storage: PlainStorage) -> Self {
        Self {
            account: Some(PlainAccount { info, storage }),
            status: AccountStatus::InMemoryChange,
        }
    }

    /// Create account that is destroyed.
    pub fn new_destroyed() -> Self {
        Self {
            account: None,
            status: AccountStatus::Destroyed,
        }
    }

    /// Create changed account
    pub fn new_changed(info: AccountInfo, storage: PlainStorage) -> Self {
        Self {
            account: Some(PlainAccount { info, storage }),
            status: AccountStatus::Changed,
        }
    }

    /// Return true if account is some
    pub fn is_some(&self) -> bool {
        matches!(
            self.status,
            AccountStatus::Changed
                | AccountStatus::InMemoryChange
                | AccountStatus::DestroyedChanged
                | AccountStatus::Loaded
                | AccountStatus::LoadedEmptyEIP161
        )
    }

    /// Return storage slot if it exist.
    pub fn storage_slot(&self, slot: U256) -> Option<U256> {
        self.account
            .as_ref()
            .and_then(|a| a.storage.get(&slot).cloned())
    }

    /// Fetch account info if it exist.
    pub fn account_info(&self) -> Option<AccountInfo> {
        self.account.clone().map(|a| a.info)
    }

    /// Dissolve account into components.
    pub fn into_components(self) -> (Option<(AccountInfo, PlainStorage)>, AccountStatus) {
        (self.account.map(|a| a.into_components()), self.status)
    }

    /// Account got touched and before EIP161 state clear this account is considered created.
    pub fn touch_create_pre_eip161(
        &mut self,
        storage: StorageWithOriginalValues,
    ) -> Option<TransitionAccount> {
        let previous_status = self.status;

        let had_no_info = self
            .account
            .as_ref()
            .map(|a| a.info.is_empty())
            .unwrap_or_default();
        self.status = self.status.on_touched_created_pre_eip161(had_no_info)?;

        let plain_storage = storage.iter().map(|(k, v)| (*k, v.present_value)).collect();
        let previous_info = self.account.take().map(|a| a.info);

        self.account = Some(PlainAccount::new_empty_with_storage(plain_storage));

        Some(TransitionAccount {
            info: Some(AccountInfo::default()),
            status: self.status,
            previous_info,
            previous_status,
            storage,
            storage_was_destroyed: false,
        })
    }

    /// Touch empty account, related to EIP-161 state clear.
    ///
    /// This account returns the Transition that is used to create the BundleState.
    pub fn touch_empty_eip161(&mut self) -> Option<TransitionAccount> {
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
                storage: HashMap::default(),
                storage_was_destroyed: true,
            })
        }
    }

    /// Consume self and make account as destroyed.
    ///
    /// Set account as None and set status to Destroyer or DestroyedAgain.
    pub fn selfdestruct(&mut self) -> Option<TransitionAccount> {
        // account should be None after selfdestruct so we can take it.
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
                storage: HashMap::new(),
                storage_was_destroyed: true,
            })
        }
    }

    /// Newly created account.
    pub fn newly_created(
        &mut self,
        new_info: AccountInfo,
        new_storage: StorageWithOriginalValues,
    ) -> TransitionAccount {
        let previous_status = self.status;
        let previous_info = self.account.take().map(|a| a.info);

        let new_bundle_storage = new_storage
            .iter()
            .map(|(k, s)| (*k, s.present_value))
            .collect();

        self.status = self.status.on_created();
        let transition_account = TransitionAccount {
            info: Some(new_info.clone()),
            status: self.status,
            previous_status,
            previous_info,
            storage: new_storage,
            storage_was_destroyed: false,
        };
        self.account = Some(PlainAccount {
            info: new_info,
            storage: new_bundle_storage,
        });
        transition_account
    }

    /// Increment balance by `balance` amount. Assume that balance will not
    /// overflow or be zero.
    ///
    /// Note: only if balance is zero we would return None as no transition would be made.
    pub fn increment_balance(&mut self, balance: u128) -> Option<TransitionAccount> {
        if balance == 0 {
            return None;
        }
        let (_, transition) = self.account_info_change(|info| {
            info.balance = info.balance.saturating_add(U256::from(balance));
        });
        Some(transition)
    }

    fn account_info_change<T, F: FnOnce(&mut AccountInfo) -> T>(
        &mut self,
        change: F,
    ) -> (T, TransitionAccount) {
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
                storage: HashMap::new(),
                storage_was_destroyed: false,
            },
        )
    }

    /// Drain balance from account and return drained amount and transition.
    ///
    /// Used for DAO hardfork transition.
    pub fn drain_balance(&mut self) -> (u128, TransitionAccount) {
        self.account_info_change(|info| {
            let output = info.balance;
            info.balance = U256::ZERO;
            output.try_into().unwrap()
        })
    }

    pub fn change(
        &mut self,
        new: AccountInfo,
        storage: StorageWithOriginalValues,
    ) -> TransitionAccount {
        let previous_status = self.status;
        let previous_info = self.account.clone().map(|a| a.info);
        let mut this_storage = self
            .account
            .take()
            .map(|acc| acc.storage)
            .unwrap_or_default();

        this_storage.extend(storage.iter().map(|(k, s)| (*k, s.present_value)));
        let changed_account = PlainAccount {
            info: new,
            storage: this_storage,
        };

        let had_no_nonce_and_code = previous_info
            .as_ref()
            .map(AccountInfo::has_no_code_and_nonce)
            .unwrap_or_default();
        self.status = self.status.on_changed(had_no_nonce_and_code);
        self.account = Some(changed_account);

        TransitionAccount {
            info: self.account.clone().map(|a| a.info),
            status: self.status,
            previous_info,
            previous_status,
            storage,
            storage_was_destroyed: false,
        }
    }
}
