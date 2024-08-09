use std::ops::{Deref, DerefMut};

use crate::primitives::{Address, Bytes, Env, Log, B256, U256};

mod dummy;
pub use dummy::DummyHost;

/// EVM context host.
pub trait Host {
    /// Returns a reference to the environment.
    fn env(&self) -> &Env;

    /// Returns a mutable reference to the environment.
    fn env_mut(&mut self) -> &mut Env;

    /// Load an account.
    ///
    /// Returns (is_cold, is_new_account)
    fn load_account(&mut self, address: Address) -> Option<AccountLoad>;

    /// Get the block hash of the given block `number`.
    fn block_hash(&mut self, number: u64) -> Option<B256>;

    /// Get balance of `address` and if the account is cold.
    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>>;

    /// Get code of `address` and if the account is cold.
    fn code(&mut self, address: Address) -> Option<Eip7702CodeLoad<Bytes>>;

    /// Get code hash of `address` and if the account is cold.
    fn code_hash(&mut self, address: Address) -> Option<Eip7702CodeLoad<B256>>;

    /// Get storage value of `address` at `index` and if the account is cold.
    fn sload(&mut self, address: Address, index: U256) -> Option<StateLoad<U256>>;

    /// Set storage value of account address at index.
    ///
    /// Returns (original, present, new, is_cold).
    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<StateLoad<SStoreResult>>;

    /// Get the transient storage value of `address` at `index`.
    fn tload(&mut self, address: Address, index: U256) -> U256;

    /// Set the transient storage value of `address` at `index`.
    fn tstore(&mut self, address: Address, index: U256, value: U256);

    /// Emit a log owned by `address` with given `LogData`.
    fn log(&mut self, log: Log);

    /// Mark `address` to be deleted, with funds transferred to `target`.
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

/// Result of the account load from Journal state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountLoad {
    /// Is account cold loaded
    pub is_cold: bool,
    /// Is account empty, if true account is not created.
    pub is_empty: bool,
}

/// Account access information.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateLoad<T> {
    /// returned data
    pub data: T,
    /// True if account is cold loaded.
    pub is_cold: bool,
}

impl<T> Deref for StateLoad<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for StateLoad<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> StateLoad<T> {
    pub fn new(data: T, is_cold: bool) -> Self {
        Self { data, is_cold }
    }

    pub fn map<B, F>(self, f: F) -> StateLoad<B>
    where
        F: FnOnce(T) -> B,
    {
        StateLoad::new(f(self.data), self.is_cold)
    }
}

/// EIP-7702 code load with potential delegate account cold load.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eip7702CodeLoad<T> {
    /// returned data
    pub state_load: StateLoad<T>,
    /// True if account has delegate code and delegated account is cold loaded.
    pub is_delegate_account_cold: bool,
}

impl<T> Deref for Eip7702CodeLoad<T> {
    type Target = StateLoad<T>;

    fn deref(&self) -> &Self::Target {
        &self.state_load
    }
}

impl<T> DerefMut for Eip7702CodeLoad<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state_load
    }
}

impl<T> Eip7702CodeLoad<T> {
    pub fn new_state_load(state_load: StateLoad<T>) -> Self {
        Self {
            state_load,
            is_delegate_account_cold: false,
        }
    }
    pub fn new_not_delegated(data: T, is_cold: bool) -> Self {
        Self {
            state_load: StateLoad::new(data, is_cold),
            is_delegate_account_cold: false,
        }
    }

    pub fn new(state_load: StateLoad<T>, is_delegate_account_cold: bool) -> Self {
        Self {
            state_load,
            is_delegate_account_cold,
        }
    }
}

/// Result of a selfdestruct instruction.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfDestructResult {
    pub had_value: bool,
    pub target_exists: bool,
    pub previously_destroyed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_host<H: Host + ?Sized>() {}

    #[test]
    fn object_safety() {
        assert_host::<DummyHost>();
        assert_host::<dyn Host>();
    }
}
