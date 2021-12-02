use bytes::Bytes;
use primitive_types::{H160, H256, U256};

use crate::{
    evm_impl::{EVMData, Handler},
    machine::Gas,
    subroutine::SubRoutine,
    CallContext, CreateScheme, Database, Env, Machine, Return, Transfer,
};
use auto_impl::auto_impl;

#[auto_impl(&mut, Box)]
pub trait Inspector<DB: Database> {
    fn initialize(&mut self, _data: &mut EVMData<'_,DB>) {}

    /// get opcode by calling `machine.contract.opcode(machine.program_counter())`.
    /// all other information can be obtained from machine.
    fn step(&mut self, machine: &mut Machine, data: &mut EVMData<'_, DB>, is_static: bool);

    /// Called after `step` when instruction is executed.
    fn step_end(&mut self, eval: Return, machine: &mut Machine);

    /// Called inside call_inner with `Return` you can dictate if you want to continue execution of
    /// this call `Return::Continue` or you want to override that and return from call.
    fn call(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: H160,
        context: &CallContext,
        transfer: &Transfer,
        input: &Bytes,
        gas_limit: u64,
        is_static: bool,
    ) -> (Return, Gas, Bytes);

    fn create(
        &mut self,
        data: &mut EVMData<'_, DB>,
        caller: H160,
        scheme: &CreateScheme,
        value: U256,
        init_code: &Bytes,
        gas: u64,
    ) -> (Return, Option<H160>, Gas, Bytes);

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
    fn initialize(&mut self, _data: &mut EVMData<'_,DB>) {}

    fn step(&mut self, _machine: &mut Machine, _data: &mut EVMData<'_, DB>, _is_static: bool) {}

    fn step_end(&mut self, _eval: Return, _machine: &mut Machine) {}

    fn call(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _call: H160,
        _context: &CallContext,
        _transfer: &Transfer,
        _input: &Bytes,
        _gas_limit: u64,
        _is_static: bool,
    ) -> (Return, Gas, Bytes) {
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn create(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _caller: H160,
        _scheme: &CreateScheme,
        _value: U256,
        _init_code: &Bytes,
        _gas: u64,
    ) -> (Return, Option<H160>, Gas, Bytes) {
        (Return::Continue, None, Gas::new(0), Bytes::new())
    }

    fn selfdestruct(&mut self) {}
}
