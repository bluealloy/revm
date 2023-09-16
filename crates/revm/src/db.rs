pub mod emptydb;
#[cfg(feature = "ethersdb")]
pub mod ethersdb;
pub mod in_memory_db;
pub mod states;

pub use crate::primitives::db::*;
pub use emptydb::{EmptyDB, EmptyDBTyped};
#[cfg(feature = "ethersdb")]
pub use ethersdb::EthersDB;
pub use in_memory_db::*;
pub use states::{
    AccountRevert, AccountStatus, BundleAccount, BundleState, CacheState, DBBox,
    OriginalValuesKnown, PlainAccount, RevertToSlot, State, StateBuilder, StateDBBox,
    StorageWithOriginalValues, TransitionAccount, TransitionState,
};

#[cfg(all(not(feature = "ethersdb"), feature = "web3db"))]
compile_error!(
    "`web3db` feature is deprecated, drop-in replacement can be found with feature `ethersdb`"
);
