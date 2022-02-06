use bytes::Bytes;
use primitive_types::{H160, U256};

use crate::{
    evm_impl::EVMData, machine::Gas, CallContext, CreateScheme, Database, Machine, Return, Transfer,
};
use auto_impl::auto_impl;

#[auto_impl(&mut, Box)]
pub trait Inspector<DB: Database> {
    fn initialize(&mut self, _data: &mut EVMData<'_, DB>) {}

    /// before machine get initialized this function is called. If returning something other them Return::Continue
    /// we are skipping execution of machine.
    fn initialize_machine(
        &mut self,
        _machine: &mut Machine,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        Return::Continue
    }

    /// get opcode by calling `machine.contract.opcode(machine.program_counter())`.
    /// all other information can be obtained from machine.
    fn step(
        &mut self,
        _machine: &mut Machine,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        Return::Continue
    }

    /// Called after `step` when instruction is executed.
    fn step_end(&mut self, _eval: Return, _machine: &mut Machine) -> Return {
        Return::Continue
    }

    // TODO introduce some struct
    /// Called inside call_inner with `Return` you can dictate if you want to continue execution of
    /// this call `Return::Continue` or you want to override that and return from call.
    #[allow(clippy::too_many_arguments)]
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

    #[allow(clippy::too_many_arguments)]
    fn call_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: H160,
        context: &CallContext,
        transfer: &Transfer,
        input: &Bytes,
        gas_limit: u64,
        remaining_gas: u64,
        ret: Return,
        out: &Bytes,
        is_static: bool,
    );

    fn create(
        &mut self,
        data: &mut EVMData<'_, DB>,
        caller: H160,
        scheme: &CreateScheme,
        value: U256,
        init_code: &Bytes,
        gas_limit: u64,
    ) -> (Return, Option<H160>, Gas, Bytes);

    #[allow(clippy::too_many_arguments)]
    fn create_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        caller: H160,
        scheme: &CreateScheme,
        value: U256,
        init_code: &Bytes,
        ret: Return,
        address: Option<H160>,
        gas_limit: u64,
        remaining_gas: u64,
        out: &Bytes,
    );

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
    fn initialize(&mut self, _data: &mut EVMData<'_, DB>) {}

    fn initialize_machine(
        &mut self,
        _machine: &mut Machine,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        Return::Continue
    }

    fn step(
        &mut self,
        _machine: &mut Machine,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        Return::Continue
    }

    fn step_end(&mut self, _eval: Return, _machine: &mut Machine) -> Return {
        Return::Continue
    }

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

    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _call: H160,
        _context: &CallContext,
        _transfer: &Transfer,
        _input: &Bytes,
        _gas_limit: u64,
        _remaining_gas: u64,
        _ret: Return,
        _out: &Bytes,
        _is_static: bool,
    ) {
    }

    fn create(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _caller: H160,
        _scheme: &CreateScheme,
        _value: U256,
        _init_code: &Bytes,
        _gas_limit: u64,
    ) -> (Return, Option<H160>, Gas, Bytes) {
        (Return::Continue, None, Gas::new(0), Bytes::new())
    }

    fn create_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _caller: H160,
        _scheme: &CreateScheme,
        _value: U256,
        _init_code: &Bytes,
        _ret: Return,
        _address: Option<H160>,
        _gas_limit: u64,
        _remaining_gas: u64,
        _out: &Bytes,
    ) {
    }

    fn selfdestruct(&mut self) {}
}
