//! BAL builder module

use crate::bal::writes::BalWrites;
use bytecode::Bytecode;
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

/// Account info bal structure.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountInfoBal {
    /// Nonce builder.
    pub nonce: BalWrites<U256>,
    /// Balance builder.
    pub balance: BalWrites<U256>,
    /// Code builder.
    pub code: BalWrites<(B256, Bytecode)>,
}

impl AccountInfoBal {
    /// Insert account into the builder.
    pub fn insert_account(
        &mut self,
        nonce: BalWrites<U256>,
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
        nonce: BalWrites<U256>,
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
