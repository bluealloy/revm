use crate::{
    db::Database,
    handler::Handler,
    handler::RegisterHandler,
    inspector_instruction,
    interpreter::{
        gas::initial_tx_gas,
        opcode::{make_boxed_instruction_table, make_instruction_table, InstructionTables},
        CallContext, CallInputs, CallScheme, CreateInputs, Host, Interpreter, InterpreterAction,
        InterpreterResult, SelfDestructResult, SharedMemory, Transfer,
    },
    journaled_state::JournaledState,
    precompile::Precompiles,
    primitives::{
        specification, Address, Bytecode, Bytes, EVMError, EVMResult, Env, InvalidTransaction, Log,
        Output, Spec, SpecId::*, TransactTo, B256, U256,
    },
    CallStackFrame, Context, EvmContext, FrameOrResult, Inspector,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use auto_impl::auto_impl;
use core::{fmt, marker::PhantomData, ops::Range};

pub fn handle_host_log<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<'_, EXT, DB>,
    address: Address,
    topics: Vec<B256>,
    data: Bytes,
) {
    // TODO register inspector handle
    // if let Some(inspector) = self.inspector.as_mut() {
    //     inspector.log(&mut self.context.evm, &address, &topics, &data);
    // }
    let log = Log {
        address,
        topics,
        data,
    };
    context.evm.journaled_state.log(log);
}

pub fn handle_selfdestruct<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<'_, EXT, DB>,
    address: Address,
    target: Address,
) -> Option<SelfDestructResult> {
    // TODO register inspector handle
    // if let Some(inspector) = self.inspector.as_mut() {
    //     let acc = self
    //         .context
    //         .evm
    //         .journaled_state
    //         .state
    //         .get(&address)
    //         .unwrap();
    //     inspector.selfdestruct(address, target, acc.info.balance);
    // }

    context
        .evm
        .journaled_state
        .selfdestruct(address, target, &mut context.evm.db)
        .map_err(|e| context.evm.error = Some(e))
        .ok()
}
