use crate::primitives::{Address, Bytes, Env, Log, B256, U256};
use core::ops::{Deref, DerefMut};

mod dummy;
pub use dummy::DummyHost;

/// EVM context host.
pub trait Host {
    /// Returns a reference to the environment.
    fn env(&self) -> &Env;

    /// Returns a mutable reference to the environment.
    fn env_mut(&mut self) -> &mut Env;

    /// Load an account code.
    fn load_account_delegated(&mut self, address: Address) -> Option<AccountLoad>;

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
    /// Returns [`StateLoad`] with [`SStoreResult`] that contains original/new/old storage value.
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

/// Result of the account load from Journal state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountLoad {
    /// Is account and delegate code are loaded
    pub load: Eip7702CodeLoad<()>,
    /// Is account empty, if true account is not created.
    pub is_empty: bool,
}

impl Deref for AccountLoad {
    type Target = Eip7702CodeLoad<()>;

    fn deref(&self) -> &Self::Target {
        &self.load
    }
}

impl DerefMut for AccountLoad {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.load
    }
}

/// State load information that contains the data and if the account or storage is cold loaded.
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
    /// Returns a new [`StateLoad`] with the given data and cold load status.
    pub fn new(data: T, is_cold: bool) -> Self {
        Self { data, is_cold }
    }

    /// Maps the data of the [`StateLoad`] to a new value.
    ///
    /// Useful for transforming the data of the [`StateLoad`] without changing the cold load status.
    pub fn map<B, F>(self, f: F) -> StateLoad<B>
    where
        F: FnOnce(T) -> B,
    {
        StateLoad::new(f(self.data), self.is_cold)
    }
}

/// EIP-7702 code load result that contains optional delegation is_cold information.
///
/// [`Self::is_delegate_account_cold`] will be [`Some`] if account has delegation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eip7702CodeLoad<T> {
    /// returned data
    pub state_load: StateLoad<T>,
    /// True if account has delegate code and delegated account is cold loaded.
    pub is_delegate_account_cold: Option<bool>,
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
    /// Returns a new [`Eip7702CodeLoad`] with the given data and without delegation.
    pub fn new_state_load(state_load: StateLoad<T>) -> Self {
        Self {
            state_load,
            is_delegate_account_cold: None,
        }
    }

    /// Returns a new [`Eip7702CodeLoad`] with the given data and without delegation.
    pub fn new_not_delegated(data: T, is_cold: bool) -> Self {
        Self {
            state_load: StateLoad::new(data, is_cold),
            is_delegate_account_cold: None,
        }
    }

    /// Deconstructs the [`Eip7702CodeLoad`] by extracting data and
    /// returning a new [`Eip7702CodeLoad`] with empty data.
    pub fn into_components(self) -> (T, Eip7702CodeLoad<()>) {
        let is_cold = self.is_cold;
        (
            self.state_load.data,
            Eip7702CodeLoad {
                state_load: StateLoad::new((), is_cold),
                is_delegate_account_cold: self.is_delegate_account_cold,
            },
        )
    }

    /// Sets the delegation cold load status.
    pub fn set_delegate_load(&mut self, is_delegate_account_cold: bool) {
        self.is_delegate_account_cold = Some(is_delegate_account_cold);
    }

    /// Returns a new [`Eip7702CodeLoad`] with the given data and delegation cold load status.
    pub fn new(state_load: StateLoad<T>, is_delegate_account_cold: bool) -> Self {
        Self {
            state_load,
            is_delegate_account_cold: Some(is_delegate_account_cold),
        }
    }
}

/// Result of a selfdestruct action.
///
/// Value returned are needed to calculate the gas spent.
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
