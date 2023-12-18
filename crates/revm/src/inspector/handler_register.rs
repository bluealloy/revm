use crate::{
    db::Database,
    handler::register::{EvmHandler, EvmInstructionTables},
    interpreter::{opcode, Interpreter, InterpreterResult},
    CallStackFrame, Evm, FrameOrResult, Inspector, JournalEntry,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use revm_interpreter::{opcode::BoxedInstruction, InstructionResult};

pub trait GetInspector<'a, DB: Database> {
    fn get_inspector(&mut self) -> &mut dyn Inspector<DB>;
}

/// Register Inspector handles that interact with Inspector instance.
///
///
/// # Note
///
/// Most of the functions are wrapped for Inspector usage expect
/// the SubCreate and SubCall calls that got overwritten.
///
/// Few instructions handlers are wrapped twice once for `step`` and `step_end`
/// and in case of Logs and Selfdestruct wrapper is wrapped again for the
/// `log` and `selfdestruct` calls.
///
/// `frame_return` is also wrapped so that Inspector could call `call_end` or `create_end`.
pub fn inspector_handle_register<'a, DB: Database, EXT: GetInspector<'a, DB>>(
    handler: &mut EvmHandler<'a, EXT, DB>,
) {
    let spec_id = handler.spec_id;
    // Every instruction inside flat table that is going to be wrapped by inspector calls.
    let table = handler
        .instruction_table
        .take()
        .expect("Handler must have instruction table");
    let mut table = match table {
        EvmInstructionTables::Plain(table) => table
            .into_iter()
            .map(|i| inspector_instruction(i))
            .collect::<Vec<_>>(),
        EvmInstructionTables::Boxed(table) => table
            .into_iter()
            .map(|i| inspector_instruction(i))
            .collect::<Vec<_>>(),
    };

    // Register inspector Log instruction.
    let mut inspect_log = |index: u8| {
        table.get_mut(index as usize).map(|i| {
            let old = core::mem::replace(i, Box::new(|_, _| ()));
            *i = Box::new(
                move |interpreter: &mut Interpreter, host: &mut Evm<'a, EXT, DB>| {
                    let old_log_len = host.context.evm.journaled_state.logs.len();
                    old(interpreter, host);
                    // check if log was added. It is possible that revert happened
                    // cause of gas or stack underflow.
                    if host.context.evm.journaled_state.logs.len() == old_log_len + 1 {
                        // clone log.
                        // TODO decide if we should remove this and leave the comment
                        // that log can be found as journaled_state.
                        let last_log = host
                            .context
                            .evm
                            .journaled_state
                            .logs
                            .last()
                            .unwrap()
                            .clone();
                        // call Inspector
                        host.context
                            .external
                            .get_inspector()
                            .log(&mut host.context.evm, &last_log);
                    }
                },
            )
        });
    };

    inspect_log(opcode::LOG0);
    inspect_log(opcode::LOG1);
    inspect_log(opcode::LOG2);
    inspect_log(opcode::LOG3);
    inspect_log(opcode::LOG4);

    // register selfdestruct function.
    table.get_mut(opcode::SELFDESTRUCT as usize).map(|i| {
        let old = core::mem::replace(i, Box::new(|_, _| ()));
        *i = Box::new(
            move |interpreter: &mut Interpreter, host: &mut Evm<'a, EXT, DB>| {
                // execute selfdestruct
                old(interpreter, host);
                // check if selfdestruct was successful and if journal entry is made.
                if let Some(JournalEntry::AccountDestroyed {
                    address,
                    target,
                    had_balance,
                    ..
                }) = host
                    .context
                    .evm
                    .journaled_state
                    .journal
                    .last()
                    .unwrap()
                    .last()
                {
                    host.context.external.get_inspector().selfdestruct(
                        *address,
                        *target,
                        *had_balance,
                    );
                }
            },
        )
    });

    // cast vector to array.
    handler.instruction_table = Some(EvmInstructionTables::Boxed(
        table.try_into().unwrap_or_else(|_| unreachable!()),
    ));

    // handle sub create
    handler.frame.sub_create = Arc::new(
        move |context, frame, mut inputs| -> Option<Box<CallStackFrame>> {
            let inspector = context.external.get_inspector();
            if let Some((result, address)) = inspector.create(&mut context.evm, &mut inputs) {
                frame.interpreter.insert_create_output(result, address);
                return None;
            }

            match context.evm.make_create_frame(spec_id, &inputs) {
                FrameOrResult::Frame(mut new_frame) => {
                    inspector.initialize_interp(&mut new_frame.interpreter, &mut context.evm);
                    Some(new_frame)
                }
                FrameOrResult::Result(result) => {
                    let (result, address) =
                        inspector.create_end(&mut context.evm, result, frame.created_address);
                    // insert result of the failed creation of create CallStackFrame.
                    frame.interpreter.insert_create_output(result, address);
                    None
                }
            }
        },
    );

    // handle sub call
    handler.frame.sub_call = Arc::new(
        move |context, mut inputs, frame, memory, return_memory_offset| -> Option<Box<_>> {
            // inspector handle
            let inspector = context.external.get_inspector();
            if let Some((result, range)) = inspector.call(&mut context.evm, &mut inputs) {
                frame.interpreter.insert_call_output(memory, result, range);
                return None;
            }
            match context
                .evm
                .make_call_frame(&inputs, return_memory_offset.clone())
            {
                FrameOrResult::Frame(mut new_frame) => {
                    inspector.initialize_interp(&mut new_frame.interpreter, &mut context.evm);
                    Some(new_frame)
                }
                FrameOrResult::Result(result) => {
                    // inspector handle
                    let result = inspector.call_end(&mut context.evm, result);
                    frame
                        .interpreter
                        .insert_call_output(memory, result, return_memory_offset);
                    None
                }
            }
        },
    );

    // return frame handle
    let old_handle = handler.frame.frame_return.clone();
    handler.frame.frame_return = Arc::new(
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
}

/// Outer closure that calls Inspector for every instruction.
pub fn inspector_instruction<
    'a,
    INSP: GetInspector<'a, DB>,
    DB: Database,
    Instruction: Fn(&mut Interpreter, &mut Evm<'a, INSP, DB>) + 'a,
>(
    instruction: Instruction,
) -> BoxedInstruction<'a, Evm<'a, INSP, DB>> {
    Box::new(
        move |interpreter: &mut Interpreter, host: &mut Evm<'a, INSP, DB>| {
            // SAFETY: as the PC was already incremented we need to subtract 1 to preserve the
            // old Inspector behavior.
            interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.sub(1) };

            host.context
                .external
                .get_inspector()
                .step(interpreter, &mut host.context.evm);
            if interpreter.instruction_result != InstructionResult::Continue {
                return;
            }

            // return PC to old value
            interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.add(1) };

            // execute instruction.
            instruction(interpreter, host);

            host.context
                .external
                .get_inspector()
                .step_end(interpreter, &mut host.context.evm);
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspectors::NoOpInspector;
    use crate::{db::EmptyDB, interpreter::opcode::*, primitives::BerlinSpec, Evm};

    #[test]
    fn test_make_boxed_instruction_table() {
        // test that this pattern builds.
        let inst: InstructionTable<Evm<'_, NoOpInspector, EmptyDB>> =
            make_instruction_table::<Evm<'_, _, _>, BerlinSpec>();
        let _test: BoxedInstructionTable<'_, Evm<'_, _, _>> =
            make_boxed_instruction_table::<'_, Evm<'_, NoOpInspector, EmptyDB>, BerlinSpec, _>(
                inst,
                inspector_instruction,
            );
    }
}
