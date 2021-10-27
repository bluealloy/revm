use bytes::Bytes;
use primitive_types::{H160, H256, U256};

use crate::{opcode::Control, CallContext, CreateScheme, ExitReason, Machine, Transfer};

pub trait Inspector {
    // get opcode by calling `machine.contract.opcode(machine.program_counter())`.
    // all other information can be obtained from machine.
    fn step(&mut self, machine: &mut Machine);
    fn eval(&mut self, eval: &mut Control, machine: &mut Machine);

    fn load_account(&mut self, address: &H160);

    fn sload(&mut self, address: &H160, slot: &H256, value: &H256, is_cold: bool);

    fn sstore(
        &mut self,
        address: H160,
        slot: H256,
        new_value: H256,
        old_value: H256,
        original_value: H256,
        is_cold: bool,
    );

    fn call(
        &mut self,
        call: H160,
        context: &CallContext,
        transfer: &Transfer,
        input: &Bytes,
        gas_limit: u64,
        is_static: bool,
    );

    fn call_return(&mut self, exit: ExitReason);

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
}

#[derive(Clone, Copy)]
pub struct NoOpInspector();

impl Inspector for NoOpInspector {
    fn step(&mut self, _machine: &mut Machine) {}

    fn eval(&mut self, _eval: &mut Control, _machine: &mut Machine) {}

    fn load_account(&mut self, _address: &H160) {}

    fn sload(&mut self, _address: &H160, _slot: &H256, _value: &H256, _is_cold: bool) {}

    fn sstore(
        &mut self,
        _address: H160,
        _slot: H256,
        _new_value: H256,
        _old_value: H256,
        _original_value: H256,
        _is_cold: bool,
    ) {
    }

    fn call(
        &mut self,
        _call: H160,
        _context: &CallContext,
        _transfer: &Transfer,
        _input: &Bytes,
        _gas_limit: u64,
        _is_static: bool,
    ) {
    }

    fn call_return(&mut self, _exit: ExitReason) {}

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
