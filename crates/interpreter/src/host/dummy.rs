use crate::{Host, SStoreResult, SelfDestructResult};
use derive_where::derive_where;
use primitives::{hash_map::Entry, Address, Bytes, HashMap, Log, B256, KECCAK_EMPTY, U256};
use std::vec::Vec;
use wiring::{
    default::{Env, EnvWiring},
    EvmWiring,
};

use super::{AccountLoad, Eip7702CodeLoad, StateLoad};

/// A dummy [Host] implementation.
#[derive_where(Clone, Debug, Default; EvmWiringT::Block, EvmWiringT::Transaction)]
pub struct DummyHost<EvmWiringT>
where
    EvmWiringT: EvmWiring,
{
    pub env: Env<EvmWiringT::Block, EvmWiringT::Transaction>,
    pub storage: HashMap<U256, U256>,
    pub transient_storage: HashMap<U256, U256>,
    pub log: Vec<Log>,
}

impl<EvmWiringT> DummyHost<EvmWiringT>
where
    EvmWiringT: EvmWiring,
{
    /// Create a new dummy host with the given [`Env`].
    #[inline]
    pub fn new(env: EnvWiring<EvmWiringT>) -> Self {
        Self {
            env,
            storage: HashMap::default(),
            transient_storage: HashMap::default(),
            log: Vec::new(),
        }
    }

    /// Clears the storage and logs of the dummy host.
    #[inline]
    pub fn clear(&mut self) {
        self.storage.clear();
        self.log.clear();
    }
}

impl<EvmWiringT> Host for DummyHost<EvmWiringT>
where
    EvmWiringT: EvmWiring,
{
    type EvmWiringT = EvmWiringT;

    #[inline]
    fn env(&self) -> &EnvWiring<Self::EvmWiringT> {
        &self.env
    }

    #[inline]
    fn env_mut(&mut self) -> &mut EnvWiring<Self::EvmWiringT> {
        &mut self.env
    }

    #[inline]
    fn load_account_delegated(&mut self, _address: Address) -> Option<AccountLoad> {
        Some(AccountLoad::default())
    }

    #[inline]
    fn block_hash(&mut self, _number: u64) -> Option<B256> {
        Some(B256::ZERO)
    }

    #[inline]
    fn balance(&mut self, _address: Address) -> Option<StateLoad<U256>> {
        Some(Default::default())
    }

    #[inline]
    fn code(&mut self, _address: Address) -> Option<Eip7702CodeLoad<Bytes>> {
        Some(Default::default())
    }

    #[inline]
    fn code_hash(&mut self, _address: Address) -> Option<Eip7702CodeLoad<B256>> {
        Some(Eip7702CodeLoad::new_not_delegated(KECCAK_EMPTY, false))
    }

    #[inline]
    fn sload(&mut self, _address: Address, index: U256) -> Option<StateLoad<U256>> {
        match self.storage.entry(index) {
            Entry::Occupied(entry) => Some(StateLoad::new(*entry.get(), false)),
            Entry::Vacant(entry) => {
                entry.insert(U256::ZERO);
                Some(StateLoad::new(U256::ZERO, true))
            }
        }
    }

    #[inline]
    fn sstore(
        &mut self,
        _address: Address,
        index: U256,
        value: U256,
    ) -> Option<StateLoad<SStoreResult>> {
        let present = self.storage.insert(index, value);
        Some(StateLoad {
            data: SStoreResult {
                original_value: U256::ZERO,
                present_value: present.unwrap_or(U256::ZERO),
                new_value: value,
            },
            is_cold: present.is_none(),
        })
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
    fn log(&mut self, log: Log) {
        self.log.push(log)
    }

    #[inline]
    fn selfdestruct(
        &mut self,
        _address: Address,
        _target: Address,
    ) -> Option<StateLoad<SelfDestructResult>> {
        Some(StateLoad::default())
    }
}
