use crate::{
    gas::{self, COLD_ACCOUNT_ACCESS_COST, WARM_STORAGE_READ_COST},
    interpreter::{Interpreter, InterpreterAction},
    primitives::{Address, Bytes, Log, LogData, Spec, SpecId::*, B256, U256},
    CallContext, CallInputs, CallScheme, CreateInputs, CreateScheme, Host, InstructionResult,
    Transfer, MAX_INITCODE_SIZE,
};
use alloc::{boxed::Box, vec::Vec};

pub fn data_load<H: Host>(interpreter: &mut Interpreter, host: &mut H) {}

pub fn data_loadn<H: Host>(interpreter: &mut Interpreter, host: &mut H) {}

pub fn data_size<H: Host>(interpreter: &mut Interpreter, host: &mut H) {}

pub fn data_copy<H: Host>(interpreter: &mut Interpreter, host: &mut H) {}
