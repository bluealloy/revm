use crate::primitives::{Bytecode, Bytes, HashMap, U256};
use crate::{
    primitives::{Address, Env, Log, B256, KECCAK_EMPTY},
    CallInputs, CreateInputs, Gas, Host, InstructionResult, SelfDestructResult, SharedMemory,
};
use alloc::vec::Vec;
use fluentbase_sdk::{Bytes32, LowLevelAPI, LowLevelSDK};

/// A dummy [Host] implementation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FluentHost {
    pub env: Env,
    pub storage: HashMap<U256, U256>,
    pub transient_storage: HashMap<U256, U256>,
    pub log: Vec<Log>,
}

impl FluentHost {
    /// Create a new dummy host with the given [`Env`].
    #[inline]
    pub fn new(env: Env) -> Self {
        Self {
            env,
            ..Default::default()
        }
    }

    /// Clears the storage and logs of the dummy host.
    #[inline]
    pub fn clear(&mut self) {
        self.storage.clear();
        self.log.clear();
    }
}

impl Host for FluentHost {
    #[inline]
    fn env(&mut self) -> &mut Env {
        &mut self.env
    }

    #[inline]
    fn load_account(&mut self, _address: Address) -> Option<(bool, bool)> {
        Some((true, true))
    }

    #[inline]
    fn block_hash(&mut self, _number: U256) -> Option<B256> {
        Some(B256::ZERO)
    }

    #[inline]
    fn balance(&mut self, _address: Address) -> Option<(U256, bool)> {
        Some((U256::ZERO, false))
    }

    #[inline]
    fn code(&mut self, _address: Address) -> Option<(Bytecode, bool)> {
        Some((Bytecode::default(), false))
    }

    #[inline]
    fn code_hash(&mut self, __address: Address) -> Option<(B256, bool)> {
        Some((KECCAK_EMPTY, false))
    }

    #[inline]
    fn sload(&mut self, __address: Address, index: U256) -> Option<(U256, bool)> {
        let mut key: Bytes32 = Default::default();
        key.copy_from_slice(index.as_le_slice());
        let mut result: [u8; 32] = [0; 32];
        LowLevelSDK::zktrie_load(&key, &mut result);
        let result = U256::from_le_slice(&result);
        Some((result, true))
    }

    #[inline]
    fn sstore(
        &mut self,
        _address: Address,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        let mut key: Bytes32 = Default::default();
        key.copy_from_slice(index.as_le_slice());
        LowLevelSDK::zktrie_store(&key, &key);
        Some((U256::ZERO, U256::ZERO, value, true))
    }

    #[inline]
    fn tload(&mut self, _address: Address, index: U256) -> U256 {
        self.transient_storage
            .get(&index)
            .copied()
            .unwrap_or_default()
    }

    #[inline]
    fn tstore(&mut self, _address: Address, index: U256, value: U256) {
        self.transient_storage.insert(index, value);
    }

    #[inline]
    fn log(&mut self, address: Address, topics: Vec<B256>, data: Bytes) {
        self.log.push(Log {
            address,
            topics,
            data,
        })
    }

    #[inline]
    fn selfdestruct(&mut self, _address: Address, _target: Address) -> Option<SelfDestructResult> {
        panic!("Selfdestruct is not supported for this host")
    }

    #[inline]
    fn create(
        &mut self,
        _inputs: &mut CreateInputs,
        _shared_memory: &mut SharedMemory,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        panic!("Create is not supported for this host")
    }

    #[inline]
    fn call(
        &mut self,
        _input: &mut CallInputs,
        _shared_memory: &mut SharedMemory,
    ) -> (InstructionResult, Gas, Bytes) {
        panic!("Call is not supported for this host")
    }
}
