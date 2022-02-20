use bytes::Bytes;
use primitive_types::H160;

use crate::{evm_impl::EVMData, CallInputs, CreateInputs, Database, Gas, Interpreter, Return};
use auto_impl::auto_impl;

#[auto_impl(&mut, Box)]
pub trait Inspector<DB: Database> {
    fn initialize(&mut self, _data: &mut EVMData<'_, DB>) {}

    /// before interp get initialized this function is called. If returning something other them Return::Continue
    /// we are skipping execution of interp.
    fn initialize_interp(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        Return::Continue
    }

    /// get opcode by calling `interp.contract.opcode(interp.program_counter())`.
    /// all other information can be obtained from interp.
    fn step(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        Return::Continue
    }

    /// Called after `step` when instruction is executed.
    fn step_end(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
        _eval: Return,
    ) -> Return {
        Return::Continue
    }

    // TODO introduce some struct
    /// Called inside call_inner with `Return` you can dictate if you want to continue execution of
    /// this call `Return::Continue` or you want to override that and return from call.
    #[allow(clippy::too_many_arguments)]
    fn call(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &CallInputs,
        is_static: bool,
    ) -> (Return, Gas, Bytes);

    #[allow(clippy::too_many_arguments)]
    fn call_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &CallInputs,
        remaining_gas: Gas,
        ret: Return,
        out: &Bytes,
        is_static: bool,
    );

    fn create(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &CreateInputs,
    ) -> (Return, Option<H160>, Gas, Bytes);

    #[allow(clippy::too_many_arguments)]
    fn create_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &CreateInputs,
        ret: Return,
        address: Option<H160>,
        remaining_gas: Gas,
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

    fn initialize_interp(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        Return::Continue
    }

    fn step(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        Return::Continue
    }

    fn step_end(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
        _eval: Return,
    ) -> Return {
        Return::Continue
    }

    fn call(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CallInputs,
        _is_static: bool,
    ) -> (Return, Gas, Bytes) {
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CallInputs,
        _remaining_gas: Gas,
        _ret: Return,
        _out: &Bytes,
        _is_static: bool,
    ) {
    }

    fn create(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CreateInputs,
    ) -> (Return, Option<H160>, Gas, Bytes) {
        (Return::Continue, None, Gas::new(0), Bytes::new())
    }

    fn create_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CreateInputs,
        _ret: Return,
        _address: Option<H160>,
        _remaining_gas: Gas,
        _out: &Bytes,
    ) {
    }

    fn selfdestruct(&mut self) {}
}
