/// After account get loaded from database it can be in a lot of different states
/// while we execute multiple transaction and even blocks over account that is in memory.
/// This structure models all possible states that account can be in.
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub enum AccountStatus {
    #[default]
    LoadedNotExisting,
    Loaded,
    LoadedEmptyEIP161,
    Changed,
    New,
    NewChanged,
    Destroyed,
    DestroyedNew,
    DestroyedNewChanged,
    DestroyedAgain,
}

impl AccountStatus {
    /// Account is not midified and just loaded from database.
    pub fn not_modified(&self) -> bool {
        match self {
            AccountStatus::LoadedNotExisting
            | AccountStatus::Loaded
            | AccountStatus::LoadedEmptyEIP161 => true,
            _ => false,
        }
    }

    /// Account was destroyed by calling SELFDESTRUCT.
    /// This means that full account and storage are inside memory.
    pub fn was_destroyed(&self) -> bool {
        match self {
            AccountStatus::Destroyed
            | AccountStatus::DestroyedNew
            | AccountStatus::DestroyedNewChanged
            | AccountStatus::DestroyedAgain => true,
            _ => false,
        }
    }

    /// Account is modified but not destroyed.
    /// This means that some of storage values can be found in both
    /// memory and database.
    pub fn modified_but_not_destroyed(&self) -> bool {
        match self {
            AccountStatus::Changed | AccountStatus::New | AccountStatus::NewChanged => true,
            _ => false,
        }
    }
}
