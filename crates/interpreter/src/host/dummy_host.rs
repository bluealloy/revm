use hashbrown::{hash_map::Entry, HashMap};
use ruint::aliases::U256;

use crate::{
    Bytecode, CallInputs, CreateInputs, Env, Gas, Host, Interpreter, Log, Return,
    SelfDestructResult, B160, B256, KECCAK_EMPTY,
};

pub struct DummyHost {
    pub env: Env,
    pub storage: HashMap<U256, U256>,
    pub log: Vec<Log>,
}

impl DummyHost {
    pub fn new(env: Env) -> Self {
        Self {
            env,
            storage: HashMap::new(),
            log: Vec::new(),
        }
    }
    pub fn clear(&mut self) {
        self.storage.clear();
        self.log.clear();
    }
}

impl Host for DummyHost {
    fn step(&mut self, _interp: &mut Interpreter, _is_static: bool) -> Return {
        Return::Continue
    }

    fn step_end(&mut self, _interp: &mut Interpreter, _is_static: bool, _ret: Return) -> Return {
        Return::Continue
    }

    fn env(&mut self) -> &mut Env {
        &mut self.env
    }

    fn load_account(&mut self, _address: B160) -> Option<(bool, bool)> {
        Some((true, true))
    }

    fn block_hash(&mut self, _number: ruint::aliases::U256) -> Option<B256> {
        Some(B256::zero())
    }

    fn balance(&mut self, _address: B160) -> Option<(ruint::aliases::U256, bool)> {
        Some((U256::ZERO, false))
    }

    fn code(&mut self, _address: B160) -> Option<(Bytecode, bool)> {
        Some((Bytecode::default(), false))
    }

    fn code_hash(&mut self, __address: B160) -> Option<(B256, bool)> {
        Some((KECCAK_EMPTY, false))
    }

    fn sload(
        &mut self,
        __address: B160,
        index: ruint::aliases::U256,
    ) -> Option<(ruint::aliases::U256, bool)> {
        match self.storage.entry(index) {
            Entry::Occupied(entry) => Some((*entry.get(), false)),
            Entry::Vacant(entry) => {
                entry.insert(U256::ZERO);
                Some((U256::ZERO, true))
            }
        }
    }

    fn sstore(
        &mut self,
        _address: B160,
        index: ruint::aliases::U256,
        value: ruint::aliases::U256,
    ) -> Option<(
        ruint::aliases::U256,
        ruint::aliases::U256,
        ruint::aliases::U256,
        bool,
    )> {
        let (present, is_cold) = match self.storage.entry(index) {
            Entry::Occupied(mut entry) => (entry.insert(value), false),
            Entry::Vacant(entry) => {
                entry.insert(value);
                (U256::ZERO, true)
            }
        };

        Some((U256::ZERO, present, value, is_cold))
    }

    fn log(&mut self, address: B160, topics: Vec<B256>, data: bytes::Bytes) {
        self.log.push(Log {
            address,
            topics,
            data,
        })
    }

    fn selfdestruct(&mut self, _address: B160, _target: B160) -> Option<SelfDestructResult> {
        panic!("Create is not supported for this host")
    }

    fn create(&mut self, _inputs: &mut CreateInputs) -> (Return, Option<B160>, Gas, bytes::Bytes) {
        panic!("Create is not supported for this host")
    }

    fn call(&mut self, _input: &mut CallInputs) -> (Return, Gas, bytes::Bytes) {
        panic!("Call is not supported for this host")
    }
}
