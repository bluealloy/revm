use crate::{Host, SStoreResult, SelfDestructResult};
use context_interface::{Block, Cfg, CfgEnv, Transaction};
use primitives::{hash_map::Entry, Address, Bytes, HashMap, Log, B256, KECCAK_EMPTY, U256};
use std::vec::Vec;

use super::{AccountLoad, Eip7702CodeLoad, StateLoad};

/// A dummy [Host] implementation.
#[derive(Clone, Debug, Default)]
pub struct DummyHost<BLOCK, TX, CFG>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
{
    pub tx: TX,
    pub block: BLOCK,
    pub cfg: CFG,
    pub storage: HashMap<U256, U256>,
    pub transient_storage: HashMap<U256, U256>,
    pub log: Vec<Log>,
}

impl<BLOCK, TX> DummyHost<BLOCK, TX, CfgEnv>
where
    BLOCK: Block,
    TX: Transaction,
{
    /// Create a new dummy host with the given [`Env`].
    #[inline]
    pub fn new(tx: TX, block: BLOCK) -> Self {
        Self {
            tx,
            block,
            cfg: CfgEnv::default(),
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

impl<TX: Transaction, BLOCK: Block, CFG: Cfg> Host for DummyHost<BLOCK, TX, CFG> {
    type TX = TX;
    type BLOCK = BLOCK;
    type CFG = CFG;

    #[inline]
    fn tx(&self) -> &Self::TX {
        &self.tx
    }

    #[inline]
    fn block(&self) -> &Self::BLOCK {
        &self.block
    }

    #[inline]
    fn cfg(&self) -> &Self::CFG {
        &self.cfg
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
