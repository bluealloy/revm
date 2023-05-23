
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