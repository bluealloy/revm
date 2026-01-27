use super::{Account, EvmStorageSlot};
use primitives::{Address, AddressMap, HashMap, StorageKey, StorageKeyMap, StorageValue};

/// EVM State is a mapping from addresses to accounts.
pub type EvmState = AddressMap<Account>;

/// Structure used for EIP-1153 transient storage
pub type TransientStorage = HashMap<(Address, StorageKey), StorageValue>;

/// An account's Storage is a mapping from 256-bit integer keys to [EvmStorageSlot]s.
pub type EvmStorage = StorageKeyMap<EvmStorageSlot>;
