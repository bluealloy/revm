use bytes::Bytes;
use revm::{Gas, Inspector, Return};

pub struct Controller {
    
    pub pc: usize,
}

impl<DB> Inspector<DB> for Controller {
    fn step(&mut self, machine: &mut revm::Machine) {
        
    }

    fn eval(&mut self, eval: revm::Return, machine: &mut revm::Machine) {
        
    }

    fn load_account(&mut self, address: &primitive_types::H160) {
        
    }

    fn sload(
        &mut self,
        address: &primitive_types::H160,
        slot: &primitive_types::U256,
        value: &primitive_types::U256,
        is_cold: bool,
    ) {
    }

    fn sstore(
        &mut self,
        address: primitive_types::H160,
        slot: primitive_types::U256,
        new_value: primitive_types::U256,
        old_value: primitive_types::U256,
        original_value: primitive_types::U256,
        is_cold: bool,
    ) {
        
    }

    fn call(
        &mut self,
        env: &mut revm::Env,
        subroutine: &mut revm::SubRoutine,
        _: &mut DB,
        call: primitive_types::H160,
        context: &revm::CallContext,
        transfer: &revm::Transfer,
        input: &bytes::Bytes,
        gas_limit: u64,
        is_static: bool,
    ) -> (Return, Gas, Bytes) {
        (Return::Continue,Gas::new(0),Bytes::new())
    }

    fn call_return(&mut self, exit: revm::Return) {
    }

    fn create(
        &mut self,
        caller: primitive_types::H160,
        scheme: &revm::CreateScheme,
        value: primitive_types::U256,
        init_code: &bytes::Bytes,
        gas: u64,
    ) {
    }

    fn create_return(&mut self, address: primitive_types::H256) {
    }

    fn selfdestruct(&mut self) {
    }
}
