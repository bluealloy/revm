//! Database implementations.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

#[cfg(feature = "alloydb")]
mod alloydb;

pub use database_interface::*;

pub mod in_memory_db;
pub mod states;

#[cfg(feature = "alloydb")]
pub use alloydb::{AlloyDB, BlockId, DBTransportError};

pub use in_memory_db::*;
pub use states::{
    AccountRevert, AccountStatus, BundleAccount, BundleState, CacheState, DBBox,
    OriginalValuesKnown, PlainAccount, RevertToSlot, State, StateBuilder, StateDBBox,
    StorageWithOriginalValues, TransitionAccount, TransitionState,
};
