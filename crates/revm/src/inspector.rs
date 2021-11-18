use bytes::Bytes;
use primitive_types::{H160, H256, U256};

use crate::{
    machine::Gas, subroutine::SubRoutine, CallContext, CreateScheme, Database, Env, Machine,
    Return, Transfer,
};
use auto_impl::auto_impl;

#[auto_impl(&mut, Box)]
pub trait Inspector<DB> {
    /// get opcode by calling `machine.contract.opcode(machine.program_counter())`.
    /// all other information can be obtained from machine.
    fn step(&mut self, machine: &mut Machine);

    /// Called after `step` when instruction is executed. 
    fn eval(&mut self, eval: Return, machine: &mut Machine);

    fn load_account(&mut self, address: &H160);

    fn sload(&mut self, address: &H160, slot: &U256, value: &U256, is_cold: bool);

    fn sstore(
        &mut self,
        address: H160,
        slot: U256,
        new_value: U256,
        old_value: U256,
        original_value: U256,
        is_cold: bool,
    );

    /// Called inside call_inner with `Return` you can dictate if you want to continue execution of
    /// this call `Return::Continue` or you want to override that and return from call.
    fn call(
        &mut self,
        env: &mut Env,
        subroutine: &mut SubRoutine,
        db: &mut DB,
        call: H160,
        context: &CallContext,
        transfer: &Transfer,
        input: &Bytes,
        gas_limit: u64,
        is_static: bool,
    ) -> (Return, Gas, Bytes);

    fn call_return(&mut self, exit: Return);

    fn create(
        &mut self,
        caller: H160,
        scheme: &CreateScheme,
        value: U256,
        init_code: &Bytes,
        gas: u64,
    );

    fn create_return(&mut self, address: H256);

    fn selfdestruct(&mut self);

    /// If needed you can override some of the spec configurations when running with inspector
    fn override_spec(&self) -> &OverrideSpec {
        &OVERRIDE_SPEC_DEFAULT
    }
}

const OVERRIDE_SPEC_DEFAULT: OverrideSpec = OverrideSpec {
    eip170_contract_code_size_limit: usize::MAX,
};
pub struct OverrideSpec {
    pub eip170_contract_code_size_limit: usize,
}

impl Default for OverrideSpec {
    fn default() -> Self {
        OVERRIDE_SPEC_DEFAULT
    }
}

#[derive(Clone, Copy)]
pub struct NoOpInspector();

impl<DB: Database> Inspector<DB> for NoOpInspector {
    fn step(&mut self, _machine: &mut Machine) {}

    fn eval(&mut self, _eval: Return, _machine: &mut Machine) {}

    fn load_account(&mut self, _address: &H160) {}

    fn sload(&mut self, _address: &H160, _slot: &U256, _value: &U256, _is_cold: bool) {}

    fn sstore(
        &mut self,
        _address: H160,
        _slot: U256,
        _new_value: U256,
        _old_value: U256,
        _original_value: U256,
        _is_cold: bool,
    ) {
    }

    fn call(
        &mut self,
        _env: &mut Env,
        _subroutine: &mut SubRoutine,
        _db: &mut DB,
        _call: H160,
        _context: &CallContext,
        _transfer: &Transfer,
        _input: &Bytes,
        _gas_limit: u64,
        _is_static: bool,
    ) -> (Return, Gas, Bytes) {
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn call_return(&mut self, _exit: Return) {}

    fn create(
        &mut self,
        _caller: H160,
        _scheme: &CreateScheme,
        _value: U256,
        _init_code: &Bytes,
        _gas: u64,
    ) {
    }

    fn create_return(&mut self, _address: H256) {}

    fn selfdestruct(&mut self) {}
}
