use crate::{
    db::Database,
    handler::Handler,
    inspector_instruction,
    interpreter::{InterpreterResult, SelfDestructResult},
    CallStackFrame, Evm, FrameOrResult, Inspector,
};
use alloc::sync::Arc;
use revm_interpreter::opcode::{BoxedInstructionTable, InstructionTable, InstructionTables};

pub trait GetInspector<'a, DB: Database> {
    fn get_inspector(&mut self) -> &mut dyn Inspector<DB>;
}

/// Wants
/// List of function that would modify the handler
/// Functions need to be Spec aware. Generic over Spec.
/// They dont need to be tied to one structure, so they need to be generic over trait.
///
/// Problems:
/// Trait Remove it
///
pub type EvmHandler<'a, EXT, DB> = Handler<'a, Evm<'a, EXT, DB>, EXT, DB>;

#[derive(Default)]
pub enum RawInstructionTable<'a, EXT, DB: Database> {
    #[default]
    Default,
    PlainRaw(InstructionTable<Evm<'a, EXT, DB>>),
    BoxedRaw(BoxedInstructionTable<'a, Evm<'a, EXT, DB>>),
}

impl<'a, EXT, DB: Database> RawInstructionTable<'a, EXT, DB> {
    pub fn into_arc(self) -> InstructionTables<'a, Evm<'a, EXT, DB>> {
        match self {
            Self::Default => unimplemented!("Default instruction table is not supported"),
            Self::PlainRaw(table) => InstructionTables::Plain(Arc::new(table)),
            Self::BoxedRaw(table) => InstructionTables::Boxed(Arc::new(table)),
        }
    }
}

// Note that
pub type HandleRegister<'a, EXT, DB> =
    Box<dyn Fn(&mut EvmHandler<'a, EXT, DB>, &mut RawInstructionTable<'a, EXT, DB>)>;

pub enum Register<'a, EXT, DB: Database> {
    Plain(fn(&'a mut EvmHandler<'a, EXT, DB>, &'a mut RawInstructionTable<'a, EXT, DB>)),
    Box(Box<dyn Fn(&'a mut EvmHandler<'a, EXT, DB>, &'a mut RawInstructionTable<'a, EXT, DB>)>),
}

pub fn inspector_handle_register<'a, DB: Database, EXT: GetInspector<'a, DB>>(
    handler: &'a mut EvmHandler<'a, EXT, DB>,
    instruction_table: &'a mut RawInstructionTable<'a, EXT, DB>,
) {
    let spec_id = handler.spec_id;
    let taken_table = core::mem::take(instruction_table);
    // Every instruction inside flat table that is going to be wrapped by inspector calls.
    match taken_table {
        RawInstructionTable::PlainRaw(table) => {
            *instruction_table = RawInstructionTable::BoxedRaw(
                table
                    .into_iter()
                    .map(|i| inspector_instruction(i))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap_or_else(|_| unreachable!()),
            );
        }
        RawInstructionTable::BoxedRaw(table) => {
            *instruction_table = RawInstructionTable::BoxedRaw(
                table
                    .into_iter()
                    .map(|i| inspector_instruction(i))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap_or_else(|_| unreachable!()),
            );
        }
        _ => unreachable!(),
    }

    // handle sub create
    handler.frame_sub_create = Arc::new(
        move |context, frame, mut inputs| -> Option<Box<CallStackFrame>> {
            if let Some((result, address)) = context
                .external
                .get_inspector()
                .create(&mut context.evm, &mut inputs)
            {
                frame.interpreter.insert_create_output(result, address);
                return None;
            }

            match context.evm.make_create_frame(spec_id, &inputs) {
                FrameOrResult::Frame(new_frame) => Some(new_frame),
                FrameOrResult::Result(result) => {
                    let (result, address) = context.external.get_inspector().create_end(
                        &mut context.evm,
                        result,
                        frame.created_address,
                    );
                    // insert result of the failed creation of create CallStackFrame.
                    frame.interpreter.insert_create_output(result, address);
                    None
                }
            }
        },
    );

    // handle sub call
    handler.frame_sub_call = Arc::new(
        move |context, mut inputs, frame, memory, return_memory_offset| -> Option<Box<_>> {
            // inspector handle
            let inspector = &mut context.external.get_inspector();
            if let Some((result, range)) = inspector.call(&mut context.evm, &mut inputs) {
                frame.interpreter.insert_call_output(memory, result, range);
                return None;
            }
            match context
                .evm
                .make_call_frame(&inputs, return_memory_offset.clone())
            {
                FrameOrResult::Frame(new_frame) => Some(new_frame),
                FrameOrResult::Result(result) => {
                    // inspector handle
                    let result = context
                        .external
                        .get_inspector()
                        .call_end(&mut context.evm, result);
                    frame
                        .interpreter
                        .insert_call_output(memory, result, return_memory_offset);
                    None
                }
            }
        },
    );

    // return frame handle
    let old_handle = handler.frame_return.clone();
    handler.frame_return = Arc::new(
        move |context, mut child, parent, memory, mut result| -> Option<InterpreterResult> {
            let inspector = &mut context.external.get_inspector();
            result = if child.is_create {
                let (result, address) =
                    inspector.create_end(&mut context.evm, result, child.created_address);
                child.created_address = address;
                result
            } else {
                inspector.call_end(&mut context.evm, result)
            };
            old_handle(context, child, parent, memory, result)
        },
    );

    // handle log
    let old_handle = handler.host_log.clone();
    handler.host_log = Arc::new(move |context, address, topics, data| {
        context
            .external
            .get_inspector()
            .log(&mut context.evm, &address, &topics, &data);
        old_handle(context, address, topics, data)
    });

    // selfdestruct handle
    let old_handle = handler.host_selfdestruct.clone();
    handler.host_selfdestruct = Arc::new(
        move |context, address, target| -> Option<SelfDestructResult> {
            let inspector = &mut context.external.get_inspector();
            let acc = context.evm.journaled_state.state.get(&address).unwrap();
            inspector.selfdestruct(address, target, acc.info.balance);
            old_handle(context, address, target)
        },
    );
}
