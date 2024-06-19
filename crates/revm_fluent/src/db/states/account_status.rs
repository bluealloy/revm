/// After account get loaded from database it can be in a lot of different states
/// while we execute multiple transaction and even blocks over account that is in memory.
/// This structure models all possible states that account can be in.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AccountStatus {
    #[default]
    LoadedNotExisting,
    Loaded,
    LoadedEmptyEIP161,
    InMemoryChange,
    Changed,
    Destroyed,
    DestroyedChanged,
    DestroyedAgain,
}

impl AccountStatus {
    /// Account is not modified and just loaded from database.
    pub fn is_not_modified(&self) -> bool {
        matches!(
            self,
            AccountStatus::LoadedNotExisting
                | AccountStatus::Loaded
                | AccountStatus::LoadedEmptyEIP161
        )
    }

    /// Account was destroyed by calling SELFDESTRUCT.
    /// This means that full account and storage are inside memory.
    pub fn was_destroyed(&self) -> bool {
        matches!(
            self,
            AccountStatus::Destroyed
                | AccountStatus::DestroyedChanged
                | AccountStatus::DestroyedAgain
        )
    }

    /// This means storage is known, it can be newly created or storage got destroyed.
    pub fn is_storage_known(&self) -> bool {
        matches!(
            self,
            AccountStatus::LoadedNotExisting
                | AccountStatus::InMemoryChange
                | AccountStatus::Destroyed
                | AccountStatus::DestroyedChanged
                | AccountStatus::DestroyedAgain
        )
    }

    /// Account is modified but not destroyed.
    /// This means that some storage values can be found in both
    /// memory and database.
    pub fn is_modified_and_not_destroyed(&self) -> bool {
        matches!(self, AccountStatus::Changed | AccountStatus::InMemoryChange)
    }

    /// Returns the next account status on creation.
    pub fn on_created(&self) -> AccountStatus {
        match self {
            // if account was destroyed previously just copy new info to it.
            AccountStatus::DestroyedAgain
            | AccountStatus::Destroyed
            | AccountStatus::DestroyedChanged => AccountStatus::DestroyedChanged,
            // if account is loaded from db.
            AccountStatus::LoadedNotExisting
            // Loaded empty eip161 to creates is not possible as CREATE2 was added after EIP-161
            | AccountStatus::LoadedEmptyEIP161
            | AccountStatus::Loaded
            | AccountStatus::Changed
            | AccountStatus::InMemoryChange => {
                // If account is loaded and not empty this means that account has some balance.
                // This means that account cannot be created.
                // We are assuming that EVM did necessary checks before allowing account to be created.
                AccountStatus::InMemoryChange
            }
        }
    }

    /// Returns the next account status on touched empty account post state clear EIP (EIP-161).
    ///
    /// # Panics
    ///
    /// If current status is [AccountStatus::Loaded] or [AccountStatus::Changed].
    pub fn on_touched_empty_post_eip161(&self) -> AccountStatus {
        match self {
            // Account can be touched but not existing. The status should remain the same.
            AccountStatus::LoadedNotExisting => AccountStatus::LoadedNotExisting,
            // Account can be created empty and only then touched.
            AccountStatus::InMemoryChange
            | AccountStatus::Destroyed
            | AccountStatus::LoadedEmptyEIP161 => AccountStatus::Destroyed,
            // Transition to destroy the account.
            AccountStatus::DestroyedAgain | AccountStatus::DestroyedChanged => {
                AccountStatus::DestroyedAgain
            }
            // Account statuses considered unreachable.
            AccountStatus::Loaded | AccountStatus::Changed => {
                unreachable!("Wrong state transition, touch empty is not possible from {self:?}");
            }
        }
    }

    /// Returns the next account status on touched or created account pre state clear EIP (EIP-161).
    /// Returns `None` if the account status didn't change.
    ///
    /// # Panics
    ///
    /// If current status is [AccountStatus::Loaded] or [AccountStatus::Changed].
    pub fn on_touched_created_pre_eip161(&self, had_no_info: bool) -> Option<AccountStatus> {
        match self {
            AccountStatus::LoadedEmptyEIP161 => None,
            AccountStatus::DestroyedChanged => {
                if had_no_info {
                    None
                } else {
                    Some(AccountStatus::DestroyedChanged)
                }
            }
            AccountStatus::Destroyed | AccountStatus::DestroyedAgain => {
                Some(AccountStatus::DestroyedChanged)
            }
            AccountStatus::InMemoryChange | AccountStatus::LoadedNotExisting => {
                Some(AccountStatus::InMemoryChange)
            }
            AccountStatus::Loaded | AccountStatus::Changed => {
                unreachable!("Wrong state transition, touch crate is not possible from {self:?}")
            }
        }
    }

    /// Returns the next account status on change.
    pub fn on_changed(&self, had_no_nonce_and_code: bool) -> AccountStatus {
        match self {
            // If the account was loaded as not existing, promote it to changed.
            // This account was likely created by a balance transfer.
            AccountStatus::LoadedNotExisting => AccountStatus::InMemoryChange,
            // Change on empty account, should transfer storage if there is any.
            // There is possibility that there are storage entries inside db.
            // That storage is used in merkle tree calculation before state clear EIP.
            AccountStatus::LoadedEmptyEIP161 => AccountStatus::InMemoryChange,
            // The account was loaded as existing.
            AccountStatus::Loaded => {
                if had_no_nonce_and_code {
                    // account is fully in memory
                    AccountStatus::InMemoryChange
                } else {
                    // can be contract and some of storage slots can be present inside db.
                    AccountStatus::Changed
                }
            }

            // On change, the "changed" type account statuses are preserved.
            // Any checks for empty accounts are done outside of this fn.
            AccountStatus::Changed => AccountStatus::Changed,
            AccountStatus::InMemoryChange => AccountStatus::InMemoryChange,
            AccountStatus::DestroyedChanged => AccountStatus::DestroyedChanged,

            // If account is destroyed and then changed this means this is
            // balance transfer.
            AccountStatus::Destroyed | AccountStatus::DestroyedAgain => {
                AccountStatus::DestroyedChanged
            }
        }
    }

    /// Returns the next account status on selfdestruct.
    pub fn on_selfdestructed(&self) -> AccountStatus {
        match self {
            // Non existing account can't be destroyed.
            AccountStatus::LoadedNotExisting => AccountStatus::LoadedNotExisting,
            // If account is created and selfdestructed in the same block, mark it as destroyed again.
            // Note: there is no big difference between Destroyed and DestroyedAgain in this case,
            // but was added for clarity.
            AccountStatus::DestroyedChanged
            | AccountStatus::DestroyedAgain
            | AccountStatus::Destroyed => AccountStatus::DestroyedAgain,

            // Transition to destroyed status.
            _ => AccountStatus::Destroyed,
        }
    }

    /// Transition to other state while preserving invariance of this state.
    ///
    /// It this account was Destroyed and other account is not:
    /// we should mark extended account as destroyed too.
    /// and as other account had some changes, extended account
    /// should be marked as DestroyedChanged.
    ///
    /// If both account are not destroyed and if this account is in memory:
    /// this means that extended account is in memory too.
    ///
    /// Otherwise, if both are destroyed or other is destroyed:
    /// set other status to extended account.
    pub fn transition(&mut self, other: Self) {
        *self = match (self.was_destroyed(), other.was_destroyed()) {
            (true, false) => Self::DestroyedChanged,
            (false, false) if *self == Self::InMemoryChange => Self::InMemoryChange,
            _ => other,
        };
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_account_status() {
        // account not modified
        assert!(AccountStatus::Loaded.is_not_modified());
        assert!(AccountStatus::LoadedEmptyEIP161.is_not_modified());
        assert!(AccountStatus::LoadedNotExisting.is_not_modified());
        assert!(!AccountStatus::Changed.is_not_modified());
        assert!(!AccountStatus::InMemoryChange.is_not_modified());
        assert!(!AccountStatus::Destroyed.is_not_modified());
        assert!(!AccountStatus::DestroyedChanged.is_not_modified());
        assert!(!AccountStatus::DestroyedAgain.is_not_modified());

        // we know full storage
        assert!(!AccountStatus::LoadedEmptyEIP161.is_storage_known());
        assert!(AccountStatus::LoadedNotExisting.is_storage_known());
        assert!(AccountStatus::InMemoryChange.is_storage_known());
        assert!(AccountStatus::Destroyed.is_storage_known());
        assert!(AccountStatus::DestroyedChanged.is_storage_known());
        assert!(AccountStatus::DestroyedAgain.is_storage_known());
        assert!(!AccountStatus::Loaded.is_storage_known());
        assert!(!AccountStatus::Changed.is_storage_known());

        // account was destroyed
        assert!(!AccountStatus::LoadedEmptyEIP161.was_destroyed());
        assert!(!AccountStatus::LoadedNotExisting.was_destroyed());
        assert!(!AccountStatus::InMemoryChange.was_destroyed());
        assert!(AccountStatus::Destroyed.was_destroyed());
        assert!(AccountStatus::DestroyedChanged.was_destroyed());
        assert!(AccountStatus::DestroyedAgain.was_destroyed());
        assert!(!AccountStatus::Loaded.was_destroyed());
        assert!(!AccountStatus::Changed.was_destroyed());

        // account modified but not destroyed
        assert!(AccountStatus::Changed.is_modified_and_not_destroyed());
        assert!(AccountStatus::InMemoryChange.is_modified_and_not_destroyed());
        assert!(!AccountStatus::Loaded.is_modified_and_not_destroyed());
        assert!(!AccountStatus::LoadedEmptyEIP161.is_modified_and_not_destroyed());
        assert!(!AccountStatus::LoadedNotExisting.is_modified_and_not_destroyed());
        assert!(!AccountStatus::Destroyed.is_modified_and_not_destroyed());
        assert!(!AccountStatus::DestroyedChanged.is_modified_and_not_destroyed());
        assert!(!AccountStatus::DestroyedAgain.is_modified_and_not_destroyed());
    }
}
