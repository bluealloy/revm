use crate::{
    Bytecode, Bytes, CallInputs, CreateInputs, Env, Gas, Interpreter, Return, SelfDestructResult,
    B160, B256, U256,
};

/// EVM context host.
pub trait Host {
    fn step(&mut self, interp: &mut Interpreter, is_static: bool) -> Return;
    fn step_end(&mut self, interp: &mut Interpreter, is_static: bool, ret: Return) -> Return;

    fn env(&mut self) -> &mut Env;

    /// load account. Returns (is_cold,is_new_account)
    fn load_account(&mut self, address: B160) -> Option<(bool, bool)>;
    /// Get environmental block hash.
    fn block_hash(&mut self, number: U256) -> Option<B256>;
    /// Get balance of address.
    fn balance(&mut self, address: B160) -> Option<(U256, bool)>;
    /// Get code of address.
    fn code(&mut self, address: B160) -> Option<(Bytecode, bool)>;
    /// Get code hash of address.
    fn code_hash(&mut self, address: B160) -> Option<(B256, bool)>;
    /// Get storage value of address at index.
    fn sload(&mut self, address: B160, index: U256) -> Option<(U256, bool)>;
    /// Set storage value of address at index. Return if slot is cold/hot access.
    fn sstore(
        &mut self,
        address: B160,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)>;
    /// Create a log owned by address with given topics and data.
    fn log(&mut self, address: B160, topics: Vec<B256>, data: Bytes);
    /// Mark an address to be deleted, with funds transferred to target.
    fn selfdestruct(&mut self, address: B160, target: B160) -> Option<SelfDestructResult>;
    /// Invoke a create operation.
    fn create(&mut self, inputs: &mut CreateInputs) -> (Return, Option<B160>, Gas, Bytes);
    /// Invoke a call operation.
    fn call(&mut self, input: &mut CallInputs) -> (Return, Gas, Bytes);
}
