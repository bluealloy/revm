//! Block Access List (BAL) data structures for efficient state access in blockchain execution.
//!
//! This module provides types for managing Block Access Lists, which optimize state access
//! by pre-computing and organizing data that will be accessed during block execution.
//!
//! ## Key Types
//!
//! - [`BlockAccessIndex`]: block access index
//! - [`BalAccountInfo`]: account fields a BAL changed, each `None` when not written
//! - [`BalAccountLookup`]: complete, partial, or uncovered account information at a read position
//! - **`Bal`**: Main BAL structure containing a map of accounts
//! - **`BalWrites<T>`**: Array of (index, value) pairs representing sequential writes to a state item
//! - **`AccountBal`**: Complete BAL structure for an account (balance, nonce, code, and storage)
//! - **`AccountInfoBal`**: Account info BAL data (nonce, balance, code)
//! - **`StorageBal`**: Storage-level BAL data for an account

pub mod account;
pub mod alloy;
pub mod writes;

pub use account::{AccountBal, AccountInfoBal, StorageBal};
pub use alloy_eip7928::{BalAccountInfo, BlockAccessIndex};
pub use writes::BalWrites;

use crate::{Account, AccountId, AccountInfo};
use alloy_eip7928::BlockAccessList as AlloyBal;
use primitives::{Address, AddressIndexMap, StorageKey, StorageValue};

/// BAL structure.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bal {
    /// Accounts bal.
    pub accounts: AddressIndexMap<AccountBal>,
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
            accounts: AddressIndexMap::default(),
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
        sorted_accounts.sort_unstable_by_key(|(address, _)| *address);

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
    pub fn update_account(
        &mut self,
        bal_index: BlockAccessIndex,
        address: Address,
        account: &Account,
    ) {
        let bal_account = self.accounts.entry(address).or_default();
        bal_account.update(bal_index, account);
    }

    /// Populate account from BAL. Return true if account info got changed.
    pub fn populate_account_info(
        &self,
        account_id: AccountId,
        bal_index: BlockAccessIndex,
        account: &mut AccountInfo,
    ) -> Result<bool, BalError> {
        let Some((_, bal_account)) = self.accounts.get_index(account_id.get()) else {
            return Err(BalError::InvalidAccountId { account_id });
        };
        account.account_id = Some(account_id);

        Ok(bal_account.populate_account_info(bal_index, account))
    }

    /// Populate storage slot from BAL.
    ///
    /// If slot is not found in BAL, it will return an error.
    #[inline]
    pub fn populate_storage_slot_by_account_id(
        &self,
        account_id: AccountId,
        bal_index: BlockAccessIndex,
        key: StorageKey,
        value: &mut StorageValue,
    ) -> Result<(), BalError> {
        let Some((address, bal_account)) = self.accounts.get_index(account_id.get()) else {
            return Err(BalError::InvalidAccountId { account_id });
        };

        if let Some(bal_value) = bal_account.storage.get(address, key, bal_index)? {
            *value = bal_value;
        };

        Ok(())
    }

    /// Populate storage slot from BAL by account address.
    #[inline]
    pub fn populate_storage_slot(
        &self,
        account_address: Address,
        bal_index: BlockAccessIndex,
        key: StorageKey,
        value: &mut StorageValue,
    ) -> Result<(), BalError> {
        let Some(bal_account) = self.accounts.get(&account_address) else {
            return Err(BalError::AccountNotFound {
                address: account_address,
            });
        };

        if let Some(bal_value) = bal_account.storage.get(&account_address, key, bal_index)? {
            *value = bal_value;
        };
        Ok(())
    }

    /// Get storage from BAL.
    pub fn account_storage(
        &self,
        account_id: AccountId,
        key: StorageKey,
        bal_index: BlockAccessIndex,
    ) -> Result<StorageValue, BalError> {
        let Some((address, bal_account)) = self.accounts.get_index(account_id.get()) else {
            return Err(BalError::InvalidAccountId { account_id });
        };

        let Some(storage_value) = bal_account.storage.get(address, key, bal_index)? else {
            return Err(BalError::SlotNotFound {
                address: *address,
                slot: key,
            });
        };

        Ok(storage_value)
    }

    /// Consume `Bal` and create a canonical EIP-7928 [`AlloyBal`].
    ///
    /// The returned access list is ordered deterministically: accounts are
    /// sorted lexicographically by address, and each account's nested reads and
    /// changes are sorted by [`AccountBal::into_alloy_account`].
    ///
    /// This matches the EIP-7928 ordering requirements:
    /// <https://eips.ethereum.org/EIPS/eip-7928#ordering-uniqueness-and-determinism>.
    pub fn into_alloy_bal(self) -> AlloyBal {
        let mut alloy_bal = AlloyBal::from_iter(
            self.accounts
                .into_iter()
                .map(|(address, account)| account.into_alloy_account(address)),
        );
        alloy_bal.sort_unstable_by_key(|a| a.address);
        alloy_bal
    }
}

/// Account information available from a BAL at a read position.
///
/// Returned by [`AccountInfoBal::account_info_lookup`] and `BalState::get_bal_account_info`
/// without consulting the backing database.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BalAccountLookup {
    /// Every account field is known, including decoded bytecode.
    ///
    /// This describes field completeness, not account existence. An empty account is returned as
    /// an [`AccountInfo`]; the caller decides whether state-clearing rules make it absent.
    Complete(AccountInfo),
    /// Only some account fields are known. Missing fields must be read from the backing account.
    ///
    /// An empty [`BalAccountInfo`] means no account fields are known at this position; it does
    /// not mean the account itself is empty or absent. Code changes carry their hash only.
    Partial(BalAccountInfo),
    /// No BAL is attached, or the account is missing and database fallback is enabled.
    NotCovered,
}

/// Error returned when a BAL (Block Access List, [EIP-7928]) lookup
/// cannot find data the caller expected to be present.
///
/// A BAL is supposed to enumerate every account and storage slot a block
/// will touch, so when execution queries the BAL for an entry that is
/// missing, the BAL is either malformed or being consulted for state that
/// it does not cover. Each variant identifies which kind of lookup failed
/// and carries the key that was queried so callers can report it.
///
/// Produced by [`Bal`] read paths ([`Bal::populate_account_info`],
/// [`Bal::populate_storage_slot`], [`Bal::populate_storage_slot_by_account_id`],
/// [`Bal::account_storage`], [`StorageBal::get`], [`StorageBal::get_bal_writes`])
/// and surfaced through `BalState` / `BalDatabase` in `revm-database-interface`,
/// where it is wrapped into `EvmDatabaseError::Bal` before reaching the EVM.
///
/// [EIP-7928]: https://eips.ethereum.org/EIPS/eip-7928
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BalError {
    /// The address was not present in the BAL's accounts map.
    ///
    /// Returned by address-keyed lookups (e.g. `BalState::get_account_id`,
    /// `BalState::storage`, `Bal::populate_storage_slot`) when the BAL is
    /// attached but does not list this account. Means the BAL is
    /// incomplete for the access being attempted.
    AccountNotFound {
        /// Address that was not found.
        address: Address,
    },
    /// The supplied [`AccountId`] index is out of range for the BAL's
    /// accounts map.
    ///
    /// `AccountId`s are positional indices into the BAL — they are only
    /// valid for the same BAL they were obtained from. This variant
    /// indicates a stale or mismatched id was used (e.g. an id from a
    /// different BAL, or one created before the current BAL was built).
    InvalidAccountId {
        /// Account id that was supplied.
        account_id: AccountId,
    },
    /// The account exists in the BAL but the requested storage slot is not
    /// listed under it.
    ///
    /// Returned by storage lookups when the account is covered by the BAL
    /// yet this particular slot was not declared. As with
    /// [`BalError::AccountNotFound`], this indicates the BAL is incomplete
    /// for the access being attempted.
    SlotNotFound {
        /// Address of the account whose slot was missing.
        address: Address,
        /// Storage slot that was not found.
        slot: StorageKey,
    },
}

impl core::fmt::Display for BalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AccountNotFound { address } => {
                write!(f, "Account {address} not found in BAL")
            }
            Self::InvalidAccountId { account_id } => {
                write!(f, "Invalid BAL account id {}", account_id.get())
            }
            Self::SlotNotFound { address, slot } => {
                write!(f, "Slot {slot:#x} not found in BAL for account {address}")
            }
        }
    }
}

impl core::error::Error for BalError {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_eip7928::{
        AccountChanges as AlloyAccountChanges, BalanceChange as AlloyBalanceChange,
        CodeChange as AlloyCodeChange, NonceChange as AlloyNonceChange,
        SlotChanges as AlloySlotChanges, StorageChange as AlloyStorageChange,
    };
    use bytecode::Bytecode;
    use primitives::{Bytes, B256, U256};
    use std::collections::BTreeMap;

    fn code(byte: u8) -> (B256, Bytecode) {
        let bytecode = Bytecode::new_raw(vec![byte].into());
        (bytecode.hash_slow(), bytecode)
    }

    #[test]
    #[cfg(all(feature = "serde", feature = "account-ext"))]
    fn extension_history_messagepack_roundtrip() {
        let mut account = AccountInfoBal::default();
        let legacy = (&account.nonce, &account.balance, &account.code);
        let encoded = rmp_serde::to_vec(&account).unwrap();
        assert_eq!(encoded, rmp_serde::to_vec(&legacy).unwrap());
        assert_eq!(
            rmp_serde::from_slice::<AccountInfoBal>(&encoded).unwrap(),
            account
        );

        // Clearing an extension is a write, not an empty history.
        account
            .extension
            .force_update(idx(1), crate::AccountExtension::default());
        let encoded = rmp_serde::to_vec(&account).unwrap();
        assert_eq!(
            rmp_serde::from_slice::<AccountInfoBal>(&encoded).unwrap(),
            account
        );
    }

    const fn idx(index: u64) -> BlockAccessIndex {
        BlockAccessIndex::new(index)
    }

    #[test]
    #[cfg(feature = "account-ext")]
    fn account_extension_roundtrips_through_bal() {
        let original = AccountInfo::default();
        let present = original
            .clone()
            .with_extension(Bytes::from_static(b"extension"));
        let mut bal = AccountInfoBal::default();
        bal.update(idx(1), &original, &present);

        let mut replayed = original;
        assert!(bal.populate_account_info(idx(2), &mut replayed));
        assert_eq!(replayed.extension, present.extension);
    }

    #[test]
    fn into_alloy_bal_canonicalizes_eip_7928_ordering() {
        let low_address = Address::with_last_byte(1);
        let high_address = Address::with_last_byte(2);

        let unordered_account = AccountBal {
            account_info: AccountInfoBal {
                nonce: BalWrites {
                    writes: vec![(idx(9), 90), (idx(4), 40)],
                },
                balance: BalWrites {
                    writes: vec![(idx(5), U256::from(50)), (idx(2), U256::from(20))],
                },
                code: BalWrites {
                    writes: vec![(idx(7), code(7)), (idx(3), code(3))],
                },
                #[cfg(feature = "account-ext")]
                extension: BalWrites::default(),
            },
            storage: StorageBal {
                storage: BTreeMap::from([
                    (
                        U256::from(4),
                        BalWrites {
                            writes: vec![(idx(8), U256::from(80)), (idx(6), U256::from(60))],
                        },
                    ),
                    (U256::from(1), BalWrites { writes: vec![] }),
                    (
                        U256::from(2),
                        BalWrites {
                            writes: vec![(idx(3), U256::from(30)), (idx(1), U256::from(10))],
                        },
                    ),
                    (U256::from(3), BalWrites { writes: vec![] }),
                ]),
            },
        };

        let alloy_bal = Bal::from_iter([
            (high_address, AccountBal::default()),
            (low_address, unordered_account),
        ])
        .into_alloy_bal();

        assert_eq!(
            alloy_bal
                .iter()
                .map(|account| account.address)
                .collect::<Vec<_>>(),
            vec![low_address, high_address]
        );

        let account = &alloy_bal[0];
        assert_eq!(account.storage_reads, vec![U256::from(1), U256::from(3)]);
        assert_eq!(
            account
                .storage_changes
                .iter()
                .map(|slot| slot.slot)
                .collect::<Vec<_>>(),
            vec![U256::from(2), U256::from(4)]
        );
        assert_eq!(
            account.storage_changes[0]
                .changes
                .iter()
                .map(|change| change.block_access_index)
                .collect::<Vec<_>>(),
            vec![idx(1), idx(3)]
        );
        assert_eq!(
            account.storage_changes[1]
                .changes
                .iter()
                .map(|change| change.block_access_index)
                .collect::<Vec<_>>(),
            vec![idx(6), idx(8)]
        );
        assert_eq!(
            account
                .balance_changes
                .iter()
                .map(|change| change.block_access_index)
                .collect::<Vec<_>>(),
            vec![idx(2), idx(5)]
        );
        assert_eq!(
            account
                .nonce_changes
                .iter()
                .map(|change| change.block_access_index)
                .collect::<Vec<_>>(),
            vec![idx(4), idx(9)]
        );
        assert_eq!(
            account
                .code_changes
                .iter()
                .map(|change| change.block_access_index)
                .collect::<Vec<_>>(),
            vec![idx(3), idx(7)]
        );
    }

    #[test]
    fn try_from_alloy_decodes_block_access_list() {
        let address = Address::with_last_byte(1);
        let code_bytes = Bytes::from_static(&[0x60, 0x00]);
        let alloy_bal = vec![AlloyAccountChanges {
            address,
            code_changes: vec![AlloyCodeChange::new(idx(1), code_bytes.clone())],
            ..Default::default()
        }];

        let bal = Bal::try_from_alloy(alloy_bal).unwrap();
        let account = bal.accounts.get(&address).unwrap();
        let (_, bytecode) = &account.account_info.code.writes[0].1;

        assert_eq!(bytecode.original_bytes(), code_bytes);
    }

    #[test]
    fn clone_from_alloy_matches_owned_conversion() {
        let address = Address::with_last_byte(1);
        let code_bytes = Bytes::from_static(&[0x60, 0x00]);
        let alloy_bal = vec![AlloyAccountChanges {
            address,
            storage_changes: vec![AlloySlotChanges::new(
                U256::from(1),
                vec![AlloyStorageChange::new(idx(1), U256::from(10))],
            )],
            storage_reads: vec![U256::from(2)],
            balance_changes: vec![AlloyBalanceChange::new(idx(2), U256::from(20))],
            nonce_changes: vec![AlloyNonceChange::new(idx(3), 30)],
            code_changes: vec![AlloyCodeChange::new(idx(4), code_bytes.clone())],
        }];

        let borrowed = Bal::clone_from_alloy(&alloy_bal).unwrap();
        let owned = Bal::try_from_alloy(alloy_bal.clone()).unwrap();

        assert_eq!(borrowed, owned);
        assert_eq!(alloy_bal[0].code_changes[0].new_code(), &code_bytes);
    }

    #[test]
    fn try_from_alloy_errors_on_invalid_code_change() {
        let alloy_bal = vec![AlloyAccountChanges {
            address: Address::with_last_byte(1),
            code_changes: vec![AlloyCodeChange::new(idx(1), vec![0xef, 0x01, 0xde].into())],
            ..Default::default()
        }];

        assert!(Bal::try_from_alloy(alloy_bal).is_err());
    }

    #[test]
    fn clone_from_alloy_errors_on_invalid_code_change() {
        let alloy_bal = vec![AlloyAccountChanges {
            address: Address::with_last_byte(1),
            code_changes: vec![AlloyCodeChange::new(idx(1), vec![0xef, 0x01, 0xde].into())],
            ..Default::default()
        }];

        assert!(Bal::clone_from_alloy(&alloy_bal).is_err());
    }

    #[test]
    fn account_info_lookup_uses_exclusive_index_and_includes_post_execution() {
        let (code_hash, bytecode) = code(1);
        #[cfg(feature = "account-ext")]
        let extension = crate::AccountExtension::copy_from_slice(b"extension");
        let account = AccountInfoBal {
            nonce: BalWrites::new(vec![(idx(2), 5)]),
            balance: BalWrites::new(vec![
                (idx(0), U256::from(1)),
                (idx(1), U256::from(7)),
                // Post-execution for a two-transaction block.
                (idx(3), U256::from(42)),
            ]),
            code: BalWrites::new(vec![(idx(3), (code_hash, bytecode.clone()))]),
            #[cfg(feature = "account-ext")]
            extension: BalWrites::new(vec![(idx(3), extension.clone())]),
        };

        for (index, balance, nonce) in [
            (0, None, None),
            (1, Some(U256::from(1)), None),
            (2, Some(U256::from(7)), None),
            (3, Some(U256::from(7)), Some(5)),
        ] {
            assert_eq!(
                account.account_info_lookup(idx(index)),
                BalAccountLookup::Partial(BalAccountInfo {
                    balance,
                    nonce,
                    code_hash: None
                })
            );
        }

        let BalAccountLookup::Complete(info) = account.account_info_lookup(idx(4)) else {
            panic!("all fields must be known after post-execution");
        };
        assert_eq!(info.balance, U256::from(42));
        assert_eq!(info.nonce, 5);
        assert_eq!(info.code_hash, code_hash);
        assert_eq!(info.code, Some(bytecode));
        #[cfg(feature = "account-ext")]
        assert_eq!(info.extension, extension);
    }

    #[test]
    fn account_info_lookup_distinguishes_missing_fields_from_empty_values() {
        let index = idx(1);
        let bytecode = Bytecode::default();
        for fields in 0..8 {
            let mut account = AccountInfoBal::default();
            let mut expected = BalAccountInfo::default();
            if fields & 1 != 0 {
                account.balance = BalWrites::new(vec![(index, U256::ZERO)]);
                expected.balance = Some(U256::ZERO);
            }
            if fields & 2 != 0 {
                account.nonce = BalWrites::new(vec![(index, 0)]);
                expected.nonce = Some(0);
            }
            if fields & 4 != 0 {
                account.code =
                    BalWrites::new(vec![(index, (bytecode.hash_slow(), bytecode.clone()))]);
                expected.code_hash = Some(bytecode.hash_slow());
            }
            #[cfg(feature = "account-ext")]
            {
                account.extension = BalWrites::new(vec![(index, crate::AccountExtension::new())]);
            }

            let lookup = account.account_info_lookup(idx(2));
            if expected.is_complete() {
                let BalAccountLookup::Complete(info) = lookup else {
                    panic!("all recorded fields must produce a complete account");
                };
                assert!(info.is_empty());
                assert_eq!(info.code, Some(bytecode.clone()));
            } else {
                assert_eq!(lookup, BalAccountLookup::Partial(expected));
            }
        }
    }

    #[test]
    #[cfg(feature = "account-ext")]
    fn account_info_lookup_requires_extension_write() {
        let (code_hash, bytecode) = code(1);
        let account = AccountInfoBal {
            nonce: BalWrites::new(vec![(idx(1), 1)]),
            balance: BalWrites::new(vec![(idx(1), U256::from(1))]),
            code: BalWrites::new(vec![(idx(1), (code_hash, bytecode))]),
            extension: BalWrites::default(),
        };

        // The backing account may carry an extension the BAL never wrote.
        assert_eq!(
            account.account_info_lookup(idx(2)),
            BalAccountLookup::Partial(BalAccountInfo {
                balance: Some(U256::from(1)),
                nonce: Some(1),
                code_hash: Some(code_hash),
            })
        );
    }
}
