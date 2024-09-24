use super::{Account, EvmStorageSlot};
use primitives::{Address, HashMap, U256};
use rustc_hash::{FxBuildHasher};

/// EVM State is a mapping from addresses to accounts.
pub type EvmState = HashMap<Address, Account, FxBuildHasher>;

/// Structure used for EIP-1153 transient storage.
pub type TransientStorage = HashMap<(Address, U256), U256, FxBuildHasher>;

/// An account's Storage is a mapping from 256-bit integer keys to [EvmStorageSlot]s.
pub type EvmStorage = HashMap<U256, EvmStorageSlot, FxBuildHasher>;
