//! Alloy BAL types conversions.

// Re-export Alloy BAL types.
pub use alloy_eip7928::{
    BalanceChange as AlloyBalanceChange, BlockAccessList as AlloyBal,
    CodeChange as AlloyCodeChange, NonceChange as AlloyNonceChange,
    StorageChange as AlloyStorageChange,
};

use crate::bal::{AccountBal, Bal, BalWrites};
use bytecode::{Bytecode, BytecodeDecodeError};
use primitives::{AddressIndexMap, B256, U256};
use std::vec::Vec;

impl TryFrom<AlloyBal> for Bal {
    type Error = BytecodeDecodeError;

    fn try_from(alloy_bal: AlloyBal) -> Result<Self, Self::Error> {
        let accounts = AddressIndexMap::from_iter(
            alloy_bal
                .into_iter()
                .map(AccountBal::try_from_alloy)
                .collect::<Result<Vec<_>, _>>()?,
        );

        Ok(Self { accounts })
    }
}

impl From<Vec<AlloyBalanceChange>> for BalWrites<U256> {
    fn from(value: Vec<AlloyBalanceChange>) -> Self {
        Self {
            writes: value
                .into_iter()
                .map(|change| (change.block_access_index, change.post_balance))
                .collect(),
        }
    }
}

impl From<Vec<AlloyNonceChange>> for BalWrites<u64> {
    fn from(value: Vec<AlloyNonceChange>) -> Self {
        Self {
            writes: value
                .into_iter()
                .map(|change| (change.block_access_index, change.new_nonce))
                .collect(),
        }
    }
}

impl From<Vec<AlloyStorageChange>> for BalWrites<U256> {
    fn from(value: Vec<AlloyStorageChange>) -> Self {
        Self {
            writes: value
                .into_iter()
                .map(|change| (change.block_access_index, change.new_value))
                .collect(),
        }
    }
}

impl TryFrom<Vec<AlloyCodeChange>> for BalWrites<(B256, Bytecode)> {
    type Error = BytecodeDecodeError;

    fn try_from(value: Vec<AlloyCodeChange>) -> Result<Self, Self::Error> {
        Ok(Self {
            writes: value
                .into_iter()
                .map(|change| {
                    // convert bytes to bytecode.
                    Bytecode::new_raw_checked(change.new_code).map(|bytecode| {
                        let hash = bytecode.hash_slow();
                        (change.block_access_index, (hash, bytecode))
                    })
                })
                .collect::<Result<Vec<_>, Self::Error>>()?,
        })
    }
}
