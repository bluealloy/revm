use crate::primitives::Bytecode;
use crate::{
    primitives::{Bytes, Env, B160, B256, U256},
    CallInputs, CreateInputs, Gas, InstructionResult, Interpreter, SelfDestructResult,
};
use alloc::vec::Vec;
pub use dummy::DummyHost;

mod dummy;

/// EVM context host.
pub trait Host {
    fn step(&mut self, interpreter: &mut Interpreter) -> InstructionResult;
    fn step_end(
        &mut self,
        interpreter: &mut Interpreter,
        ret: InstructionResult,
    ) -> InstructionResult;

    fn env(&mut self) -> &mut Env;

    /// load account. Returns (is_cold,is_new_account)
    fn load_account(&mut self, address: B160) -> Option<(bool, bool)>;
    /// Get environmental block hash.
    fn block_hash(&mut self, number: U256) -> Option<B256>;
    /// Get balance of address and if account is cold loaded.
    fn balance(&mut self, address: B160) -> Option<(U256, bool)>;
    /// Get code of address and if account is cold loaded.
    fn code(&mut self, address: B160) -> Option<(Bytecode, bool)>;
    /// Get code hash of address and if account is cold loaded.
    fn code_hash(&mut self, address: B160) -> Option<(B256, bool)>;
    /// Get storage value of address at index and if account is cold loaded.
    fn sload(&mut self, address: B160, index: U256) -> Option<(U256, bool)>;
    /// Set storage value of account address at index.
    /// Returns (original, present, new, sis_cold)
    fn sstore(
        &mut self,
        address: B160,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)>;
    /// Get the transient storage value of address at index.
    fn tload(&mut self, address: B160, index: U256) -> U256;
    /// Set the transient storage value of address at index.
    fn tstore(&mut self, address: B160, index: U256, value: U256);
    /// Create a log owned by address with given topics and data.
    fn log(&mut self, address: B160, topics: Vec<B256>, data: Bytes);
    /// Mark an address to be deleted, with funds transferred to target.
    fn selfdestruct(&mut self, address: B160, target: B160) -> Option<SelfDestructResult>;
    /// Invoke a create operation.
    fn create(
        &mut self,
        inputs: &mut CreateInputs,
    ) -> (InstructionResult, Option<B160>, Gas, Bytes);
    /// Invoke a call operation.
    fn call(&mut self, input: &mut CallInputs) -> (InstructionResult, Gas, Bytes);
}
