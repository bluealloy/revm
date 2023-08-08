/// After account get loaded from database it can be in a lot of different states
/// while we execute multiple transaction and even blocks over account that is in memory.
/// This structure models all possible states that account can be in.
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
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
    /// Account is not midified and just loaded from database.
    pub fn not_modified(&self) -> bool {
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
    pub fn storage_known(&self) -> bool {
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
    /// This means that some of storage values can be found in both
    /// memory and database.
    pub fn modified_but_not_destroyed(&self) -> bool {
        matches!(self, AccountStatus::Changed | AccountStatus::InMemoryChange)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_account_status() {
        // account not modified
        assert!(AccountStatus::Loaded.not_modified());
        assert!(AccountStatus::LoadedEmptyEIP161.not_modified());
        assert!(AccountStatus::LoadedNotExisting.not_modified());
        assert!(!AccountStatus::Changed.not_modified());
        assert!(!AccountStatus::InMemoryChange.not_modified());
        assert!(!AccountStatus::Destroyed.not_modified());
        assert!(!AccountStatus::DestroyedChanged.not_modified());
        assert!(!AccountStatus::DestroyedAgain.not_modified());

        // we know full storage
        assert!(!AccountStatus::LoadedEmptyEIP161.storage_known());
        assert!(AccountStatus::LoadedNotExisting.storage_known());
        assert!(AccountStatus::InMemoryChange.storage_known());
        assert!(AccountStatus::Destroyed.storage_known());
        assert!(AccountStatus::DestroyedChanged.storage_known());
        assert!(AccountStatus::DestroyedAgain.storage_known());
        assert!(!AccountStatus::Loaded.storage_known());
        assert!(!AccountStatus::Changed.storage_known());

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
        assert!(AccountStatus::Changed.modified_but_not_destroyed());
        assert!(AccountStatus::InMemoryChange.modified_but_not_destroyed());
        assert!(!AccountStatus::Loaded.modified_but_not_destroyed());
        assert!(!AccountStatus::LoadedEmptyEIP161.modified_but_not_destroyed());
        assert!(!AccountStatus::LoadedNotExisting.modified_but_not_destroyed());
        assert!(!AccountStatus::Destroyed.modified_but_not_destroyed());
        assert!(!AccountStatus::DestroyedChanged.modified_but_not_destroyed());
        assert!(!AccountStatus::DestroyedAgain.modified_but_not_destroyed());
    }
}
