//! BAL builder module

use crate::{
    bal::{writes::BalWrites, BalError, BalIndex},
    Account,
};
use bytecode::Bytecode;
use core::ops::{Deref, DerefMut};
use primitives::{StorageKey, StorageValue, B256, U256};
use std::collections::BTreeMap;

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
    /// Populate account from BAL.
    pub fn populate_account(&self, bal_index: BalIndex, account: &mut Account) {
        self.account_info.populate_account_info(bal_index, account);
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
    /// Populate account info from BAL.
    pub fn populate_account_info(&self, bal_index: BalIndex, account: &mut Account) {
        if let Some(nonce) = self.nonce.get(bal_index) {
            account.info.nonce = nonce;
        }
        if let Some(balance) = self.balance.get(bal_index) {
            account.info.balance = balance;
        }
        if let Some(code) = self.code.get(bal_index) {
            account.info.code_hash = code.0;
            account.info.code = Some(code.1);
        }
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

impl AccountInfoBal {
    /// Insert account into the builder.
    pub fn insert_account(
        &mut self,
        nonce: BalWrites<u64>,
        balance: BalWrites<U256>,
        code: BalWrites<(B256, Bytecode)>,
    ) {
        self.nonce.extend(nonce);
        self.balance.extend(balance);
        self.code.extend(code);
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

    /// Insert storage into the builder.
    pub fn insert_storage(
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

impl AccountBal {
    /// Insert account into the builder.
    pub fn insert_account(
        &mut self,
        nonce: BalWrites<u64>,
        balance: BalWrites<U256>,
        code: BalWrites<(B256, Bytecode)>,
        storage: impl Iterator<Item = (StorageKey, BalWrites<StorageValue>)>,
    ) {
        self.account_info.insert_account(nonce, balance, code);
        self.storage.insert_storage(storage);
    }

    /// TODO get struct from somewhere.
    pub fn into_vec(self) -> () {
        ()
    }
}
