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
use indexmap::{map::Entry, IndexMap};
pub use writes::BalWrites;

use bytecode::Bytecode;
use primitives::{Address, StorageKey, StorageValue, B256, U256};

use crate::Account;

///Block access index (0 for pre-execution, 1..n for transactions, n+1 for post-execution)
pub type BalIndex = u64;

/// BAL structure.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bal {
    /// Accounts bal.
    pub accounts: IndexMap<Address, AccountBal>,
}

impl Bal {
    /// Create a new BAL builder.
    pub fn new() -> Self {
        Self {
            accounts: IndexMap::new(),
        }
    }

    /// Pretty print the entire BAL structure in a human-readable format.
    pub fn pretty_print(&self) {
        println!("=== Block Access List (BAL) ===");
        println!("Total accounts: {}", self.accounts.len());
        println!();

        if self.accounts.is_empty() {
            println!("(empty)");
            return;
        }

        for (idx, (address, account)) in self.accounts.iter().enumerate() {
            println!("Account #{} - Address: {:?}", idx, address);
            println!("  Account Info:");

            // Print nonce writes
            if account.account_info.nonce.is_empty() {
                println!("    Nonce: (read-only, no writes)");
            } else {
                println!("    Nonce writes:");
                for (bal_index, nonce) in &account.account_info.nonce.writes {
                    println!("      [{}] -> {}", bal_index, nonce);
                }
            }

            // Print balance writes
            if account.account_info.balance.is_empty() {
                println!("    Balance: (read-only, no writes)");
            } else {
                println!("    Balance writes:");
                for (bal_index, balance) in &account.account_info.balance.writes {
                    println!("      [{}] -> {}", bal_index, balance);
                }
            }

            // Print code writes
            if account.account_info.code.is_empty() {
                println!("    Code: (read-only, no writes)");
            } else {
                println!("    Code writes:");
                for (bal_index, (code_hash, bytecode)) in &account.account_info.code.writes {
                    println!(
                        "      [{}] -> hash: {:?}, size: {} bytes",
                        bal_index,
                        code_hash,
                        bytecode.len()
                    );
                }
            }

            // Print storage writes
            println!("  Storage:");
            if account.storage.storage.is_empty() {
                println!("    (no storage slots)");
            } else {
                println!("    Total slots: {}", account.storage.storage.len());
                for (storage_key, storage_writes) in &account.storage.storage {
                    println!("    Slot: {:#x}", storage_key);
                    if storage_writes.is_empty() {
                        println!("      (read-only, no writes)");
                    } else {
                        println!("      Writes:");
                        for (bal_index, value) in &storage_writes.writes {
                            println!("        [{}] -> {:?}", bal_index, value);
                        }
                    }
                }
            }

            println!();
        }
        println!("=== End of BAL ===");
    }

    #[inline]
    /// Extend BAL with account.
    pub fn extend_account(&mut self, address: Address, account: &mut Account) {
        match self.accounts.entry(address) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().extend_account(account);
            }
            Entry::Vacant(entry) => {
                entry.insert(account.take_account_bal());
            }
        }
    }

    /// Insert account into the builder.
    pub fn insert_account(
        &mut self,
        address: Address,
        nonce: BalWrites<u64>,
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

    /// Populate account from BAL.
    pub fn populate_account(
        &self,
        address: Address,
        bal_index: BalIndex,
        account: &mut Account,
    ) -> Result<(), BalError> {
        let Some((index, _, bal_account)) = self.accounts.get_full(&address) else {
            return Err(BalError::AccountNotFound);
        };

        bal_account.populate_account(bal_index, account);
        account.bal_account_index = Some(index);

        Ok(())
    }

    /// Get storage from BAL.
    pub fn account_storage(
        &self,
        account_index: usize,
        key: StorageKey,
        bal_index: BalIndex,
    ) -> Result<StorageValue, BalError> {
        let Some((_, bal_account)) = self.accounts.get_index(account_index) else {
            return Err(BalError::AccountNotFound);
        };

        let Some(storage_value) = bal_account.storage.get(key, bal_index)? else {
            return Err(BalError::SlotNotFound);
        };

        Ok(storage_value)
    }
}

/// BAL error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BalError {
    /// Account not found in BAL.
    AccountNotFound,
    /// Slot not found in BAL.
    SlotNotFound,
}
