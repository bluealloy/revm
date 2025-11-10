//! BAL builder module

use crate::{
    bal::{writes::BalWrites, BalError, BalIndex},
    Account, AccountInfo, EvmStorage,
};
use alloy_eip7928::{
    AccountChanges as AlloyAccountChanges, BalanceChange as AlloyBalanceChange,
    CodeChange as AlloyCodeChange, NonceChange as AlloyNonceChange,
    SlotChanges as AlloySlotChanges, StorageChange as AlloyStorageChange,
};
use bytecode::{Bytecode, BytecodeDecodeError};
use core::ops::{Deref, DerefMut};
use primitives::{Address, StorageKey, StorageValue, B256, U256};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    vec::Vec,
};

/// Account BAL structure.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountBal {
    /// Account info bal.
    pub account_info: AccountInfoBal,
    /// Storage bal.
    pub storage: StorageBal,
}

impl Deref for AccountBal {
    type Target = AccountInfoBal;

    fn deref(&self) -> &Self::Target {
        &self.account_info
    }
}

impl DerefMut for AccountBal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account_info
    }
}

impl AccountBal {
    /// Populate account from BAL. Return true if account info got changed
    pub fn populate_account_info(&self, bal_index: BalIndex, account: &mut AccountInfo) -> bool {
        self.account_info.populate_account_info(bal_index, account)
    }

    /// Extend account from another account.
    #[inline]
    pub fn update(&mut self, bal_index: BalIndex, account: &Account) {
        if account.is_selfdestructed_locally() {
            let empty_info = AccountInfo::default();
            self.account_info
                .update(bal_index, &account.original_info, &empty_info);
            self.storage.update_reads(account.storage.keys().copied());
            return;
        }

        self.account_info
            .update(bal_index, &account.original_info, &account.info);

        self.storage.update(bal_index, &account.storage);
    }

    /// Create account from alloy account changes.
    #[inline]
    pub fn try_from_alloy(
        alloy_account: AlloyAccountChanges,
    ) -> Result<(Address, Self), BytecodeDecodeError> {
        Ok((
            alloy_account.address,
            AccountBal {
                account_info: AccountInfoBal {
                    nonce: BalWrites::from(alloy_account.nonce_changes),
                    balance: BalWrites::from(alloy_account.balance_changes),
                    code: BalWrites::try_from(alloy_account.code_changes)?,
                },
                storage: StorageBal::from_iter(
                    alloy_account
                        .storage_changes
                        .into_iter()
                        .chain(
                            alloy_account
                                .storage_reads
                                .into_iter()
                                .map(|key| AlloySlotChanges::new(key, Default::default())),
                        )
                        .map(|slot| (slot.slot.into(), BalWrites::from(slot.changes))),
                ),
            },
        ))
    }

    /// Consumes AccountBal and converts it into [`AlloyAccountChanges`].
    #[inline]
    pub fn into_alloy_account(self, address: Address) -> AlloyAccountChanges {
        let mut storage_reads = Vec::new();
        let mut storage_changes = Vec::new();
        for (key, value) in self.storage.storage {
            if value.writes.is_empty() {
                storage_reads.push(key.into());
            } else {
                storage_changes.push(AlloySlotChanges::new(
                    key.into(),
                    value
                        .writes
                        .into_iter()
                        .map(|(index, value)| AlloyStorageChange::new(index, value.into()))
                        .collect(),
                ));
            }
        }

        AlloyAccountChanges {
            address,
            storage_changes,
            storage_reads,
            balance_changes: self
                .account_info
                .balance
                .writes
                .into_iter()
                .map(|(index, value)| AlloyBalanceChange::new(index, value))
                .collect(),
            nonce_changes: self
                .account_info
                .nonce
                .writes
                .into_iter()
                .map(|(index, value)| AlloyNonceChange::new(index, value))
                .collect(),
            code_changes: self
                .account_info
                .code
                .writes
                .into_iter()
                .map(|(index, (_, value))| AlloyCodeChange::new(index, value.original_bytes()))
                .collect(),
        }
    }
}

/// Account info bal structure.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountInfoBal {
    /// Nonce builder.
    pub nonce: BalWrites<u64>,
    /// Balance builder.
    pub balance: BalWrites<U256>,
    /// Code builder.
    pub code: BalWrites<(B256, Bytecode)>,
}

impl AccountInfoBal {
    /// Populate account info from BAL. Return true if account info got changed
    pub fn populate_account_info(&self, bal_index: BalIndex, account: &mut AccountInfo) -> bool {
        let mut changed = false;
        if let Some(nonce) = self.nonce.get(bal_index) {
            account.nonce = nonce;
            changed = true;
        }
        if let Some(balance) = self.balance.get(bal_index) {
            account.balance = balance;
            changed = true;
        }
        if let Some(code) = self.code.get(bal_index) {
            account.code_hash = code.0;
            account.code = Some(code.1);
            changed = true;
        }
        changed
    }

    /// Extend account info from another account info.
    #[inline]
    pub fn update(&mut self, index: BalIndex, original: &AccountInfo, present: &AccountInfo) {
        self.nonce.update(index, &original.nonce, present.nonce);
        self.balance
            .update(index, &original.balance, present.balance);
        self.code.update_with_key(
            index,
            &original.code_hash,
            (present.code_hash, present.code.clone().unwrap_or_default()),
            |i| &i.0,
        );
    }

    /// Extend account info from another account info.
    #[inline]
    pub fn extend(&mut self, bal_account: AccountInfoBal) {
        self.nonce.extend(bal_account.nonce);
        self.balance.extend(bal_account.balance);
        self.code.extend(bal_account.code);
    }

    /// Update account balance in BAL.
    #[inline]
    pub fn balance_update(&mut self, bal_index: BalIndex, original_balance: &U256, balance: U256) {
        self.balance.update(bal_index, original_balance, balance);
    }

    /// Update account nonce in BAL.
    #[inline]
    pub fn nonce_update(&mut self, bal_index: BalIndex, original_nonce: &u64, nonce: u64) {
        self.nonce.update(bal_index, original_nonce, nonce);
    }

    /// Update account code in BAL.
    #[inline]
    pub fn code_update(
        &mut self,
        bal_index: BalIndex,
        original_code_hash: &B256,
        code_hash: B256,
        code: Bytecode,
    ) {
        self.code
            .update_with_key(bal_index, original_code_hash, (code_hash, code), |i| &i.0);
    }
}

/// Storage BAL
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageBal {
    /// Storage with writes and reads.
    pub storage: BTreeMap<StorageKey, BalWrites<StorageValue>>,
}

impl StorageBal {
    /// Get storage from the builder.
    pub fn get(
        &self,
        key: StorageKey,
        bal_index: BalIndex,
    ) -> Result<Option<StorageValue>, BalError> {
        let Some(value) = self.storage.get(&key) else {
            return Err(BalError::SlotNotFound);
        };

        Ok(value.get(bal_index))
    }

    /// Extend storage from another storage.
    #[inline]
    pub fn extend(&mut self, storage: StorageBal) {
        for (key, value) in storage.storage {
            match self.storage.entry(key) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().extend(value);
                }
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        }
    }

    /// Update storage from [`EvmStorage`].
    #[inline]
    pub fn update(&mut self, bal_index: BalIndex, storage: &EvmStorage) {
        for (key, value) in storage {
            self.storage.entry(*key).or_default().update(
                bal_index,
                &value.original_value,
                value.present_value,
            );
        }
    }

    /// Update reads from [`EvmStorage`].
    ///
    /// It will expend inner map with new reads.
    #[inline]
    pub fn update_reads(&mut self, storage: impl Iterator<Item = StorageKey>) {
        for key in storage {
            self.storage.entry(key).or_default();
        }
    }

    /// Insert storage into the builder.
    pub fn extend_iter(
        &mut self,
        storage: impl Iterator<Item = (StorageKey, BalWrites<StorageValue>)>,
    ) {
        for (key, value) in storage {
            self.storage.insert(key, value);
        }
    }

    /// Convert the storage into a vector of reads and writes
    pub fn into_vecs(self) -> (Vec<StorageKey>, Vec<(StorageKey, BalWrites<StorageValue>)>) {
        let mut reads = Vec::new();
        let mut writes = Vec::new();

        for (key, value) in self.storage {
            if value.writes.is_empty() {
                reads.push(key);
            } else {
                writes.push((key, value));
            }
        }

        (reads, writes)
    }
}

impl FromIterator<(StorageKey, BalWrites<StorageValue>)> for StorageBal {
    fn from_iter<I: IntoIterator<Item = (StorageKey, BalWrites<StorageValue>)>>(iter: I) -> Self {
        Self {
            storage: iter.into_iter().collect(),
        }
    }
}
