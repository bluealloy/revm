use crate::{
    db::Database,
    handler::register::{EvmHandler, EvmInstructionTables},
    interpreter::{
        opcode, opcode::BoxedInstruction, CallInputs, InstructionResult, Interpreter,
        InterpreterResult,
    },
    primitives::TransactTo,
    CallStackFrame, Evm, FrameData, FrameOrResult, Inspector, JournalEntry,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use revm_interpreter::CreateInputs;

pub trait GetInspector<'a, DB: Database> {
    fn get_inspector(&mut self) -> &mut dyn Inspector<DB>;
}

/// Register Inspector handles that interact with Inspector instance.
///
///
/// # Note
///
/// Handles that are overwritten:
/// * SubCreate
/// * SubCall
/// * CreateFirstFrame
///
///
/// Few instructions handlers are wrapped twice once for `step` and `step_end`
/// and in case of Logs and Selfdestruct wrapper is wrapped again for the
/// `log` and `selfdestruct` calls.
///
/// `frame_return` is also wrapped so that Inspector could call `call_end` or `create_end`.
///
/// `create_first_frame` is also wrapped so that Inspector could call `call` and `crate` on it.
/// While return for first frame is handled by `frame_return`.
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
        if let Some(i) = table.get_mut(index as usize) {
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
        }
    };

    inspect_log(opcode::LOG0);
    inspect_log(opcode::LOG1);
    inspect_log(opcode::LOG2);
    inspect_log(opcode::LOG3);
    inspect_log(opcode::LOG4);

    // wrap first frame create and main frame return.
    handler.execution_loop.create_first_frame =
        Arc::new(move |context, gas_limit| -> FrameOrResult {
            // call inner handling of call/create
            let mut first_frame = match context.evm.env.tx.transact_to {
                TransactTo::Call(_) => {
                    let mut call_inputs = CallInputs::new(&context.evm.env.tx, gas_limit).unwrap();
                    // call inspector and return of inspector returns result.
                    if let Some(output) = context
                        .external
                        .get_inspector()
                        .call(&mut context.evm, &mut call_inputs)
                    {
                        return FrameOrResult::Result(output.0);
                    }
                    // first call frame does not have return range.
                    context.evm.make_call_frame(&call_inputs, 0..0)
                }
                TransactTo::Create(_) => {
                    let mut create_inputs =
                        CreateInputs::new(&context.evm.env.tx, gas_limit).unwrap();
                    if let Some(output) = context
                        .external
                        .get_inspector()
                        .create(&mut context.evm, &mut create_inputs)
                    {
                        return FrameOrResult::Result(output.0);
                    };
                    context.evm.make_create_frame(spec_id, &create_inputs)
                }
            };

            // call initialize interpreter from inspector.
            if let FrameOrResult::Frame(ref mut frame) = first_frame {
                context
                    .external
                    .get_inspector()
                    .initialize_interp(&mut frame.interpreter, &mut context.evm);
            }

            first_frame
        });

    // register selfdestruct function.
    if let Some(i) = table.get_mut(opcode::SELFDESTRUCT as usize) {
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
    }

    // cast vector to array.
    handler.instruction_table = Some(EvmInstructionTables::Boxed(
        table.try_into().unwrap_or_else(|_| unreachable!()),
    ));

    // handle sub create
    handler.execution_loop.sub_create = Arc::new(
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
                        inspector.create_end(&mut context.evm, result, frame.created_address());
                    // insert result of the failed creation of create CallStackFrame.
                    frame.interpreter.insert_create_output(result, address);
                    None
                }
            }
        },
    );

    // handle sub call
    handler.execution_loop.sub_call = Arc::new(
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
    let old_handle = handler.execution_loop.frame_return.clone();
    handler.execution_loop.frame_return = Arc::new(
        move |context, mut child, parent, memory, mut result| -> Option<InterpreterResult> {
            let inspector = &mut context.external.get_inspector();
            result = match &mut child.frame_data {
                FrameData::Create { created_address } => {
                    let (result, address) =
                        inspector.create_end(&mut context.evm, result, Some(*created_address));
                    if let Some(address) = address {
                        *created_address = address;
                    }
                    result
                }
                FrameData::Call { .. } => inspector.call_end(&mut context.evm, result),
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
    use crate::{
        db::EmptyDB,
        inspector::GetInspector,
        inspectors::NoOpInspector,
        interpreter::{opcode::*, CallInputs, CreateInputs, Interpreter, InterpreterResult},
        primitives::{Address, BerlinSpec},
        Database, Evm, EvmContext, Inspector,
    };
    use core::ops::Range;

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

    #[derive(Default, Debug)]
    struct StackInspector {
        initialize_interp_called: bool,
        step: u32,
        step_end: u32,
        call: bool,
        call_end: bool,
    }

    impl<DB: Database> GetInspector<'_, DB> for StackInspector {
        fn get_inspector(&mut self) -> &mut dyn Inspector<DB> {
            self
        }
    }

    impl<DB: Database> Inspector<DB> for StackInspector {
        fn initialize_interp(&mut self, _interp: &mut Interpreter, _context: &mut EvmContext<DB>) {
            if self.initialize_interp_called {
                unreachable!("initialize_interp should not be called twice")
            }
            self.initialize_interp_called = true;
        }

        fn step(&mut self, _interp: &mut Interpreter, _context: &mut EvmContext<DB>) {
            self.step += 1;
        }

        fn step_end(&mut self, _interp: &mut Interpreter, _context: &mut EvmContext<DB>) {
            self.step_end += 1;
        }

        fn call(
            &mut self,
            _context: &mut EvmContext<DB>,
            _call: &mut CallInputs,
        ) -> Option<(InterpreterResult, Range<usize>)> {
            if self.call {
                unreachable!("call should not be called twice")
            }
            self.call = true;
            None
        }

        fn call_end(
            &mut self,
            _context: &mut EvmContext<DB>,
            result: InterpreterResult,
        ) -> InterpreterResult {
            if self.call_end {
                unreachable!("call_end should not be called twice")
            }
            self.call_end = true;
            result
        }

        fn create(
            &mut self,
            _context: &mut EvmContext<DB>,
            _call: &mut CreateInputs,
        ) -> Option<(InterpreterResult, Option<Address>)> {
            None
        }

        fn create_end(
            &mut self,
            _context: &mut EvmContext<DB>,
            result: InterpreterResult,
            address: Option<Address>,
        ) -> (InterpreterResult, Option<Address>) {
            (result, address)
        }
    }

    #[test]
    fn test_gas_inspector() {
        use crate::{
            db::BenchmarkDB,
            inspector::inspector_handle_register,
            interpreter::opcode,
            primitives::{address, Bytecode, Bytes, TransactTo},
            Evm,
        };

        let contract_data: Bytes = Bytes::from(vec![
            opcode::PUSH1,
            0x1,
            opcode::PUSH1,
            0xb,
            opcode::PUSH1,
            0x1,
            opcode::PUSH1,
            0x1,
            opcode::PUSH1,
            0x1,
            opcode::CREATE,
            opcode::STOP,
        ]);
        let bytecode = Bytecode::new_raw(contract_data);

        let mut evm: Evm<'_, StackInspector, BenchmarkDB> = Evm::builder()
            .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
            .with_external_context(StackInspector::default())
            .modify_tx_env(|tx| {
                tx.clear();
                tx.caller = address!("1000000000000000000000000000000000000000");
                tx.transact_to =
                    TransactTo::Call(address!("0000000000000000000000000000000000000000"));
                tx.gas_limit = 21100;
            })
            .append_handler_register(inspector_handle_register)
            .build();

        // run evm.
        evm.transact().unwrap();

        let inspector = evm.into_context().external;

        assert_eq!(inspector.step, 6);
        assert_eq!(inspector.step_end, 6);
        assert!(inspector.initialize_interp_called);
        assert!(inspector.call);
        assert!(inspector.call_end);
    }
}
