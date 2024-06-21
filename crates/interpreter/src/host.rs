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
    fn load_account(&mut self, address: Address) -> Option<LoadAccountResult>;

    /// Get the block hash of the given block `number`.
    fn block_hash(&mut self, number: u64) -> Option<B256>;

    /// Get balance of `address` and if the account is cold.
    fn balance(&mut self, address: Address) -> Option<(U256, bool)>;

    /// Get code of `address` and if the account is cold.
    fn code(&mut self, address: Address) -> Option<(Bytes, bool)>;

    /// Get code hash of `address` and if the account is cold.
    fn code_hash(&mut self, address: Address) -> Option<(B256, bool)>;

    /// Get storage value of `address` at `index` and if the account is cold.
    fn sload(&mut self, address: Address, index: U256) -> Option<(U256, bool)>;

    /// Set storage value of account address at index.
    ///
    /// Returns (original, present, new, is_cold).
    fn sstore(&mut self, address: Address, index: U256, value: U256) -> Option<SStoreResult>;

    /// Get the transient storage value of `address` at `index`.
    fn tload(&mut self, address: Address, index: U256) -> U256;

    /// Set the transient storage value of `address` at `index`.
    fn tstore(&mut self, address: Address, index: U256, value: U256);

    /// Emit a log owned by `address` with given `LogData`.
    fn log(&mut self, log: Log);

    /// Mark `address` to be deleted, with funds transferred to `target`.
    fn selfdestruct(&mut self, address: Address, target: Address) -> Option<SelfDestructResult>;
}

/// Represents the result of an `sstore` operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SStoreResult {
    /// Value of the storage when it is first read
    pub original_value: U256,
    /// Current value of the storage
    pub present_value: U256,
    /// New value that is set
    pub new_value: U256,
    /// Is storage slot loaded from database
    pub is_cold: bool,
}

/// Result of the account load from Journal state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoadAccountResult {
    /// Is account cold loaded
    pub is_cold: bool,
    /// Is account empty, if true account is not created.
    pub is_empty: bool,
}

/// Result of a selfdestruct instruction.
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfDestructResult {
    pub had_value: bool,
    pub target_exists: bool,
    pub is_cold: bool,
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
