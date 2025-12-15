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
pub mod alloy;
pub mod writes;

pub use account::{AccountBal, AccountInfoBal, StorageBal};
pub use writes::BalWrites;

use crate::{Account, AccountInfo};
use alloy_eip7928::BlockAccessList as AlloyBal;
use primitives::{Address, IndexMap, StorageKey, StorageValue};

/// Block access index (0 for pre-execution, 1..n for transactions, n+1 for post-execution)
pub type BalIndex = u64;

/// BAL structure.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bal {
    /// Accounts bal.
    pub accounts: IndexMap<Address, AccountBal>,
}

impl FromIterator<(Address, AccountBal)> for Bal {
    fn from_iter<I: IntoIterator<Item = (Address, AccountBal)>>(iter: I) -> Self {
        Self {
            accounts: iter.into_iter().collect(),
        }
    }
}

impl Bal {
    /// Create a new BAL builder.
    pub fn new() -> Self {
        Self {
            accounts: IndexMap::default(),
        }
    }

    /// Pretty print the entire BAL structure in a human-readable format.
    #[cfg(feature = "std")]
    pub fn pretty_print(&self) {
        println!("=== Block Access List (BAL) ===");
        println!("Total accounts: {}", self.accounts.len());
        println!();

        if self.accounts.is_empty() {
            println!("(empty)");
            return;
        }

        // Sort accounts by address before printing
        let mut sorted_accounts: Vec<_> = self.accounts.iter().collect();
        sorted_accounts.sort_by_key(|(address, _)| *address);

        for (idx, (address, account)) in sorted_accounts.into_iter().enumerate() {
            println!("Account #{idx} - Address: {address:?}");
            println!("  Account Info:");

            // Print nonce writes
            if account.account_info.nonce.is_empty() {
                println!("    Nonce: (read-only, no writes)");
            } else {
                println!("    Nonce writes:");
                for (bal_index, nonce) in &account.account_info.nonce.writes {
                    println!("      [{bal_index}] -> {nonce}");
                }
            }

            // Print balance writes
            if account.account_info.balance.is_empty() {
                println!("    Balance: (read-only, no writes)");
            } else {
                println!("    Balance writes:");
                for (bal_index, balance) in &account.account_info.balance.writes {
                    println!("      [{bal_index}] -> {balance}");
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
                    println!("    Slot: {storage_key:#x}");
                    if storage_writes.is_empty() {
                        println!("      (read-only, no writes)");
                    } else {
                        println!("      Writes:");
                        for (bal_index, value) in &storage_writes.writes {
                            println!("        [{bal_index}] -> {value:?}");
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
    pub fn update_account(&mut self, bal_index: BalIndex, address: Address, account: &Account) {
        let bal_account = self.accounts.entry(address).or_default();
        bal_account.update(bal_index, account);
    }

    /// Populate account from BAL. Return true if account info got changed
    pub fn populate_account_info(
        &self,
        account_id: usize,
        bal_index: BalIndex,
        account: &mut AccountInfo,
    ) -> Result<bool, BalError> {
        let Some((_, bal_account)) = self.accounts.get_index(account_id) else {
            return Err(BalError::AccountNotFound);
        };
        account.storage_id = Some(account_id);

        Ok(bal_account.populate_account_info(bal_index, account))
    }

    /// Populate storage slot from BAL.
    ///
    /// If slot is not found in BAL, it will return an error.
    #[inline]
    pub fn populate_storage_slot_by_account_id(
        &self,
        account_index: usize,
        bal_index: BalIndex,
        key: StorageKey,
        value: &mut StorageValue,
    ) -> Result<(), BalError> {
        let Some((_, bal_account)) = self.accounts.get_index(account_index) else {
            return Err(BalError::AccountNotFound);
        };

        if let Some(bal_value) = bal_account.storage.get(key, bal_index)? {
            *value = bal_value;
        };

        Ok(())
    }

    /// Populate storage slot from BAL by account address.
    #[inline]
    pub fn populate_storage_slot(
        &self,
        account_address: Address,
        bal_index: BalIndex,
        key: StorageKey,
        value: &mut StorageValue,
    ) -> Result<(), BalError> {
        let Some(bal_account) = self.accounts.get(&account_address) else {
            return Err(BalError::AccountNotFound);
        };

        if let Some(bal_value) = bal_account.storage.get(key, bal_index)? {
            *value = bal_value;
        };
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

    /// Consume Bal and create [`AlloyBal`]
    pub fn into_alloy_bal(self) -> AlloyBal {
        let mut alloy_bal = AlloyBal::from_iter(
            self.accounts
                .into_iter()
                .map(|(address, account)| account.into_alloy_account(address)),
        );
        alloy_bal.sort_by_key(|a| a.address);
        alloy_bal
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

impl core::fmt::Display for BalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AccountNotFound => write!(f, "Account not found in BAL"),
            Self::SlotNotFound => write!(f, "Slot not found in BAL"),
        }
    }
}
