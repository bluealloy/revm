/// After account get loaded from database it can be in a lot of different states
/// while we execute multiple transaction and even blocks over account that is memory.
/// This structure models all possible states that account can be in.
#[derive(Clone, Default, Debug)]
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
    pub fn not_modified(&self) -> bool {
        match self {
            AccountStatus::LoadedNotExisting
            | AccountStatus::Loaded
            | AccountStatus::LoadedEmptyEIP161 => true,
            _ => false,
        }
    }

    pub fn was_destroyed(&self) -> bool {
        match self {
            AccountStatus::Destroyed
            | AccountStatus::DestroyedNew
            | AccountStatus::DestroyedNewChanged
            | AccountStatus::DestroyedAgain => true,
            _ => false,
        }
    }

    pub fn modified_but_not_destroyed(&self) -> bool {
        match self {
            AccountStatus::Changed | AccountStatus::New | AccountStatus::NewChanged => true,
            _ => false,
        }
    }
}
