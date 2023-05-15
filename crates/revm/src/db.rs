pub mod in_memory_db;

pub mod db_state;
#[cfg(feature = "ethersdb")]
pub mod ethersdb;
#[cfg(feature = "ethersdb")]
pub use ethersdb::EthersDB;

#[cfg(all(not(feature = "ethersdb"), feature = "web3db"))]
compile_error!(
    "`web3db` feature is deprecated, drop-in replacement can be found with feature `ethersdb`"
);

pub use crate::primitives::db::*;
pub use db_state::{BlockState, PlainAccount, StateWithChange};
pub use in_memory_db::*;
