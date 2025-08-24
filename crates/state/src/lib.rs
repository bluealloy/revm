//! Account and storage state.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

mod account;
mod account_info;
mod storage;
mod types;

pub use account::{Account, AccountStatus};
pub use account_info::AccountInfo;
pub use bytecode::{self, Bytecode};
pub use primitives;
pub use storage::EvmStorageSlot;
pub use types::{AccountPages, EvmState, EvmStorage, TransientStorage};
