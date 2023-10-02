use crate::primitives::{hash_map::Entry, Bytecode, Bytes, HashMap, U256};
use crate::{
    primitives::{Address, Env, Log, B256, KECCAK_EMPTY},
    CallInputs, CreateInputs, Gas, Host, InstructionResult, Interpreter, SelfDestructResult,
};
use alloc::vec::Vec;

pub struct DummyHost {
    pub env: Env,
    pub storage: HashMap<U256, U256>,
    pub transient_storage: HashMap<U256, U256>,
    pub log: Vec<Log>,
}

impl DummyHost {
    /// Create a new dummy host with the given [`Env`].
    #[inline]
    pub fn new(env: Env) -> Self {
        Self {
            env,
            storage: HashMap::new(),
            transient_storage: Default::default(),
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

impl Host for DummyHost {
    #[inline]
    fn step(&mut self, _interp: &mut Interpreter) -> InstructionResult {
        InstructionResult::Continue
    }

    #[inline]
    fn step_end(
        &mut self,
        _interp: &mut Interpreter,
        _ret: InstructionResult,
    ) -> InstructionResult {
        InstructionResult::Continue
    }

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
        match self.storage.entry(index) {
            Entry::Occupied(entry) => Some((*entry.get(), false)),
            Entry::Vacant(entry) => {
                entry.insert(U256::ZERO);
                Some((U256::ZERO, true))
            }
        }
    }

    #[inline]
    fn sstore(
        &mut self,
        _address: Address,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        let (present, is_cold) = match self.storage.entry(index) {
            Entry::Occupied(mut entry) => (entry.insert(value), false),
            Entry::Vacant(entry) => {
                entry.insert(value);
                (U256::ZERO, true)
            }
        };

        Some((U256::ZERO, present, value, is_cold))
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
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        panic!("Create is not supported for this host")
    }

    #[inline]
    fn call(&mut self, _input: &mut CallInputs) -> (InstructionResult, Gas, Bytes) {
        panic!("Call is not supported for this host")
    }
}
