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

pub fn handle_frame_return<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<'_, EXT, DB>,
    mut child_stack_frame: Box<CallStackFrame>,
    parent_stack_frame: Option<&mut Box<CallStackFrame>>,
    shared_memory: &mut SharedMemory,
    mut result: InterpreterResult,
) -> Option<InterpreterResult> {
    // TODO
    // if let Some(inspector) = self.inspector.as_mut() {
    //     result = if child_stack_frame.is_create {
    //         let (result, address) = inspector.create_end(
    //             &mut self.context.evm,
    //             result,
    //             child_stack_frame.created_address,
    //         );
    //         child_stack_frame.created_address = address;
    //         result
    //     } else {
    //         inspector.call_end(&mut self.context.evm, result)
    //     };
    // }

    // break from loop if this is last CallStackFrame.
    let Some(parent_stack_frame) = parent_stack_frame else {
        let result = if child_stack_frame.is_create {
            context
                .evm
                .create_return::<SPEC>(result, child_stack_frame)
                .0
        } else {
            context.evm.call_return(result, child_stack_frame)
        };

        return Some(result);
    };

    if child_stack_frame.is_create {
        let (result, address) = context.evm.create_return::<SPEC>(result, child_stack_frame);
        parent_stack_frame
            .interpreter
            .insert_create_output(result, Some(address))
    } else {
        let subcall_memory_return_offset = child_stack_frame.subcall_return_memory_range.clone();
        let result = context.evm.call_return(result, child_stack_frame);

        parent_stack_frame.interpreter.insert_call_output(
            shared_memory,
            result,
            subcall_memory_return_offset,
        )
    }
    None
}
