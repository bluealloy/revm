use crate::{
    db::Database,
    handler::Handler,
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
    CallStackFrame, Context, Evm, EvmContext, FrameOrResult, Inspector,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use auto_impl::auto_impl;
use core::{fmt, marker::PhantomData, ops::Range};

/// Register external handles.
pub trait RegisterHandler<'a, DB: Database> {
    fn register_handler<SPEC: Spec>(
        &self,
        handler: Handler<'a, Evm<'a, SPEC, Self, DB>, Self, DB>,
    ) -> Handler<'a, Evm<'a, SPEC, Self, DB>, Self, DB>
    where
        DB: 'a,
        Self: Sized,
    {
        handler
    }
}

/// Default registered handler that produces default mainnet handler.
#[derive(Default)]
pub struct MainnetHandle {}

impl<'a, DB: Database> RegisterHandler<'a, DB> for MainnetHandle {}

pub struct InspectorHandle<'a, DB: Database, INS: Inspector<DB>> {
    pub inspector: &'a mut INS,
    pub _phantomdata: PhantomData<DB>,
}

impl<'handler, DB: Database, INS: Inspector<DB>> RegisterHandler<'handler, DB>
    for InspectorHandle<'handler, DB, INS>
{
    fn register_handler<SPEC: Spec>(
        &self,
        mut handler: Handler<'handler, Evm<'handler, SPEC, Self, DB>, Self, DB>,
    ) -> Handler<'handler, Evm<'handler, SPEC, Self, DB>, Self, DB>
    where
        Self: Sized,
        DB: 'handler,
    {
        // let instruction_table = make_boxed_instruction_table::<
        //     'handler,
        //     Evm<'handler, SPEC, InspectorHandle<'handler, DB, INS>, DB>,
        //     SPEC,
        //     _,
        // >(
        //     make_instruction_table::<
        //         Evm<'handler, SPEC, InspectorHandle<'handler, DB, INS>, DB>,
        //         SPEC,
        //     >(),
        //     inspector_instruction::<SPEC, INS, DB>,
        // );

        let flat_table = make_instruction_table::<
            Evm<'handler, SPEC, InspectorHandle<'handler, DB, INS>, DB>,
            SPEC,
        >();

        let table = core::array::from_fn(|i| inspector_instruction(flat_table[i]));

        let table = InstructionTables::Boxed(Arc::new(table));

        handler.instruction_table = table;

        // return frame handle
        let old_handle = handler.frame_return.clone();
        handler.frame_return = Arc::new(
            move |context, mut child, parent, memory, mut result| -> Option<InterpreterResult> {
                let inspector = &mut context.external.inspector;
                result = if child.is_create {
                    let (result, address) =
                        inspector.create_end(&mut context.evm, result, child.created_address);
                    child.created_address = address;
                    result
                } else {
                    inspector.call_end(&mut context.evm, result)
                };
                let output = old_handle(context, child, parent, memory, result);
                output
            },
        );

        handler
    }
}

// pub struct ExternalData<DB: Database> {
//     pub flagg: bool,
//     pub phantom: PhantomData<DB>,
// }

// impl<DB: Database> RegisterHandler<DB> for ExternalData<DB> {
//     fn register_handler<'a, SPEC: Spec>(
//         &self,
//         mut handler: Handler<'a, Evm<'a, SPEC, Self, DB>, Self, DB>,
//     ) -> Handler<'a, Evm<'a, SPEC, Self, DB>, Self, DB>
//     where
//         DB: 'a,
//     {
//         let old_handle = handler.reimburse_caller.clone();
//         handler.reimburse_caller = Arc::new(move |data, gas| old_handle(data, gas));
//         handler
//     }
// }
