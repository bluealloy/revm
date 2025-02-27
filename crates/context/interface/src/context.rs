pub use crate::journaled_state::StateLoad;
use crate::{journaled_state::AccountLoad, Block, Cfg, Database, Journal, Transaction};
use auto_impl::auto_impl;
use primitives::{Address, Bytes, Log, B256, BLOCK_HASH_HISTORY, U256};

#[auto_impl(&mut, Box)]
pub trait ContextTr {
    type Block: Block;
    type Tx: Transaction;
    type Cfg: Cfg;
    type Db: Database;
    type Journal: Journal<Database = Self::Db>;
    type Chain;

    fn tx(&self) -> &Self::Tx;
    fn block(&self) -> &Self::Block;
    fn cfg(&self) -> &Self::Cfg;
    fn journal(&mut self) -> &mut Self::Journal;
    fn journal_ref(&self) -> &Self::Journal;
    fn db(&mut self) -> &mut Self::Db;
    fn db_ref(&self) -> &Self::Db;
    fn chain(&mut self) -> &mut Self::Chain;
    fn error(&mut self) -> &mut Result<(), <Self::Db as Database>::Error>;
    fn tx_journal(&mut self) -> (&mut Self::Tx, &mut Self::Journal);
    // default implementationHost calls interface

    /// Gets the block hash of the given block `number`.
    fn block_hash(&mut self, requested_number: u64) -> Option<B256> {
        let block_number = self.block().number();

        let Some(diff) = block_number.checked_sub(requested_number) else {
            return Some(B256::ZERO);
        };

        // blockhash should push zero if number is same as current block number.
        if diff == 0 {
            return Some(B256::ZERO);
        }

        if diff <= BLOCK_HASH_HISTORY {
            return self
                .journal()
                .db()
                .block_hash(requested_number)
                .map_err(|e| {
                    *self.error() = Err(e);
                })
                .ok();
        }

        Some(B256::ZERO)
    }

    fn load_account_delegated(&mut self, address: Address) -> Option<StateLoad<AccountLoad>> {
        self.journal()
            .load_account_delegated(address)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Gets balance of `address` and if the account is cold.
    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>> {
        self.journal()
            .load_account(address)
            .map(|acc| acc.map(|a| a.info.balance))
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Gets code of `address` and if the account is cold.
    fn code(&mut self, address: Address) -> Option<StateLoad<Bytes>> {
        self.journal()
            .code(address)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Gets code hash of `address` and if the account is cold.
    fn code_hash(&mut self, address: Address) -> Option<StateLoad<B256>> {
        self.journal()
            .code_hash(address)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Gets storage value of `address` at `index` and if the account is cold.
    fn sload(&mut self, address: Address, index: U256) -> Option<StateLoad<U256>> {
        self.journal()
            .sload(address, index)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Sets storage value of account address at index.
    ///
    /// Returns [`StateLoad`] with [`SStoreResult`] that contains original/new/old storage value.
    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<StateLoad<SStoreResult>> {
        self.journal()
            .sstore(address, index, value)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Gets the transient storage value of `address` at `index`.
    fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.journal().tload(address, index)
    }

    /// Sets the transient storage value of `address` at `index`.
    fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.journal().tstore(address, index, value)
    }

    /// Emits a log owned by `address` with given `LogData`.
    fn log(&mut self, log: Log) {
        self.journal().log(log);
    }

    /// Marks `address` to be deleted, with funds transferred to `target`.
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Option<StateLoad<SelfDestructResult>> {
        self.journal()
            .selfdestruct(address, target)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }
}

/// Represents the result of an `sstore` operation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SStoreResult {
    /// Value of the storage when it is first read
    pub original_value: U256,
    /// Current value of the storage
    pub present_value: U256,
    /// New value that is set
    pub new_value: U256,
}

impl SStoreResult {
    /// Returns `true` if the new value is equal to the present value.
    #[inline]
    pub fn is_new_eq_present(&self) -> bool {
        self.new_value == self.present_value
    }

    /// Returns `true` if the original value is equal to the present value.
    #[inline]
    pub fn is_original_eq_present(&self) -> bool {
        self.original_value == self.present_value
    }

    /// Returns `true` if the original value is equal to the new value.
    #[inline]
    pub fn is_original_eq_new(&self) -> bool {
        self.original_value == self.new_value
    }

    /// Returns `true` if the original value is zero.
    #[inline]
    pub fn is_original_zero(&self) -> bool {
        self.original_value.is_zero()
    }

    /// Returns `true` if the present value is zero.
    #[inline]
    pub fn is_present_zero(&self) -> bool {
        self.present_value.is_zero()
    }

    /// Returns `true` if the new value is zero.
    #[inline]
    pub fn is_new_zero(&self) -> bool {
        self.new_value.is_zero()
    }
}

/// Result of a selfdestruct action
///
/// Value returned are needed to calculate the gas spent.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfDestructResult {
    pub had_value: bool,
    pub target_exists: bool,
    pub previously_destroyed: bool,
}
