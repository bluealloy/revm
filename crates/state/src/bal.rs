//! Block Access List (BAL) data structures for efficient state access in blockchain execution.
//!
//! This module provides types for managing Block Access Lists, which optimize state access
//! by pre-computing and organizing data that will be accessed during block execution.
//!
//! ## Key Types
//!
//! - **`BalIndex`**: Block access index (0 for pre-execution, 1..n for transactions, n+1 for post-execution)
//! - **`Bal`**: Main BAL structure containing a map of accounts
//! - **`BalWrites<T>`**: Array of (index, value) pairs representing sequential writes to a state item
//! - **`AccountBal`**: Complete BAL structure for an account (balance, nonce, code, and storage)
//! - **`AccountInfoBal`**: Account info BAL data (nonce, balance, code)
//! - **`StorageBal`**: Storage-level BAL data for an account

pub mod account;
pub mod writes;

pub use account::{AccountBal, AccountInfoBal, StorageBal};
pub use writes::BalWrites;

use bytecode::Bytecode;
use primitives::{Address, StorageKey, StorageValue, B256, U256};
use std::collections::{btree_map::Entry, BTreeMap};

///Block access index (0 for pre-execution, 1..n for transactions, n+1 for post-execution)
pub type BalIndex = u64;

/// BAL structure.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bal {
    /// Accounts bal.
    pub accounts: BTreeMap<Address, AccountBal>,
}

impl Bal {
    /// Create a new BAL builder.
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
        }
    }

    /// Insert account into the builder.
    pub fn insert_account(
        &mut self,
        address: Address,
        nonce: BalWrites<U256>,
        balance: BalWrites<U256>,
        code: BalWrites<(B256, Bytecode)>,
        storage: impl Iterator<Item = (StorageKey, BalWrites<StorageValue>)>,
    ) {
        match self.accounts.entry(address) {
            Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .insert_account(nonce, balance, code, storage);
            }
            Entry::Vacant(entry) => {
                entry.insert(AccountBal {
                    account_info: AccountInfoBal {
                        nonce,
                        balance,
                        code,
                    },
                    storage: StorageBal {
                        storage: storage.collect(),
                    },
                });
            }
        }
    }
}
