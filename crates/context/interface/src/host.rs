mod dummy;

pub use crate::journaled_state::StateLoad;
pub use dummy::DummyHost;

use crate::{journaled_state::AccountLoad, BlockGetter, CfgGetter, TransactionGetter};
use auto_impl::auto_impl;
use primitives::{Address, Bytes, Log, B256, U256};

/// EVM context host.
#[auto_impl(&mut, Box)]
pub trait Host: TransactionGetter + BlockGetter + CfgGetter {
    /// Load an account code.
    fn load_account_delegated(&mut self, address: Address) -> Option<StateLoad<AccountLoad>>;

    /// Gets the block hash of the given block `number`.
    fn block_hash(&mut self, number: u64) -> Option<B256>;

    /// Gets balance of `address` and if the account is cold.
    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>>;

    /// Gets code of `address` and if the account is cold.
    fn code(&mut self, address: Address) -> Option<StateLoad<Bytes>>;

    /// Gets code hash of `address` and if the account is cold.
    fn code_hash(&mut self, address: Address) -> Option<StateLoad<B256>>;

    /// Gets storage value of `address` at `index` and if the account is cold.
    fn sload(&mut self, address: Address, index: U256) -> Option<StateLoad<U256>>;

    /// Sets storage value of account address at index.
    ///
    /// Returns [`StateLoad`] with [`SStoreResult`] that contains original/new/old storage value.
    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<StateLoad<SStoreResult>>;

    /// Gets the transient storage value of `address` at `index`.
    fn tload(&mut self, address: Address, index: U256) -> U256;

    /// Sets the transient storage value of `address` at `index`.
    fn tstore(&mut self, address: Address, index: U256, value: U256);

    /// Emits a log owned by `address` with given `LogData`.
    fn log(&mut self, log: Log);

    /// Marks `address` to be deleted, with funds transferred to `target`.
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Option<StateLoad<SelfDestructResult>>;
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

// TODO TEST
// #[cfg(test)]
// mod tests {
//     use database_interface::EmptyDB;
//     use context_interface::EthereumWiring;

//     use super::*;

//     fn assert_host<H: Host + ?Sized>() {}

//     #[test]
//     fn object_safety() {
//         assert_host::<DummyHost<EthereumWiring<EmptyDB, ()>>>();
//         assert_host::<dyn Host<EvmWiringT = EthereumWiring<EmptyDB, ()>>>();
//     }
// }
