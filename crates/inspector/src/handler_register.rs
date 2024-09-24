use crate::Inspector;
use core::cell::RefCell;
use revm::{
    bytecode::opcode,
    handler::register::EvmHandler,
    interpreter::{table::DynInstruction, InstructionResult, Interpreter},
    wiring::result::EVMResultGeneric,
    Context, EvmWiring, FrameOrResult, FrameResult, JournalEntry,
};
use std::{rc::Rc, sync::Arc, vec::Vec};

/// Provides access to an `Inspector` instance.
pub trait GetInspector<EvmWiringT: EvmWiring> {
    /// Returns the associated `Inspector`.
    fn get_inspector(&mut self) -> &mut impl Inspector<EvmWiringT>;
}

impl<EvmWiringT: EvmWiring, INSP: Inspector<EvmWiringT>> GetInspector<EvmWiringT> for INSP {
    #[inline]
    fn get_inspector(&mut self) -> &mut impl Inspector<EvmWiringT> {
        self
    }
}

/// Register Inspector handles that interact with Inspector instance.
///
///
/// # Note
///
/// Inspector handle register does not override any existing handlers, and it
/// calls them before (or after) calling Inspector. This means that it is safe
/// to use this register with any other register.
///
/// A few instructions handlers are wrapped twice once for `step` and `step_end`
/// and in case of Logs and Selfdestruct wrapper is wrapped again for the
/// `log` and `selfdestruct` calls.
pub fn inspector_handle_register<
    EvmWiringT: EvmWiring<ExternalContext: GetInspector<EvmWiringT>>,
>(
    handler: &mut EvmHandler<'_, EvmWiringT>,
) {
    let table = &mut handler.instruction_table;

    // Update all instructions to call inspector step and step_end.
    table.update_all(inspector_instruction);

    // Register inspector LOG* instructions.
    for opcode in opcode::LOG0..=opcode::LOG4 {
        table.update_boxed(opcode, move |prev, interpreter, host| {
            let prev_log_len = host.evm.journaled_state.logs.len();
            prev(interpreter, host);
            // check if log was added. It is possible that revert happened
            // cause of gas or stack underflow.
            if host.evm.journaled_state.logs.len() == prev_log_len + 1 {
                // clone log.
                // TODO decide if we should remove this and leave the comment
                // that log can be found as journaled_state.
                let last_log = host.evm.journaled_state.logs.last().unwrap().clone();
                // call Inspector
                host.external
                    .get_inspector()
                    .log(interpreter, &mut host.evm, &last_log);
            }
        });
    }

    // Register selfdestruct function.
    table.update_boxed(opcode::SELFDESTRUCT, |prev, interpreter, host| {
        // execute selfdestruct
        prev(interpreter, host);
        // check if selfdestruct was successful and if journal entry is made.
        match host.evm.journaled_state.journal.last().unwrap().last() {
            Some(JournalEntry::AccountDestroyed {
                address,
                target,
                had_balance,
                ..
            }) => {
                host.external
                    .get_inspector()
                    .selfdestruct(*address, *target, *had_balance);
            }
            Some(JournalEntry::BalanceTransfer {
                from, to, balance, ..
            }) => {
                host.external
                    .get_inspector()
                    .selfdestruct(*from, *to, *balance);
            }
            _ => {}
        }
    });

    // call and create input stack shared between handlers. They are used to share
    // inputs in *_end Inspector calls.
    let call_input_stack = Rc::<RefCell<Vec<_>>>::default();
    let create_input_stack = Rc::<RefCell<Vec<_>>>::default();
    let eofcreate_input_stack = Rc::<RefCell<Vec<_>>>::default();

    // Create handler
    let create_input_stack_inner = create_input_stack.clone();
    let prev_handle = handler.execution.create.clone();
    handler.execution.create = Arc::new(
        move |ctx, mut inputs| -> EVMResultGeneric<FrameOrResult, EvmWiringT> {
            let inspector = ctx.external.get_inspector();
            // call inspector create to change input or return outcome.
            if let Some(outcome) = inspector.create(&mut ctx.evm, &mut inputs) {
                create_input_stack_inner.borrow_mut().push(inputs.clone());
                return Ok(FrameOrResult::Result(FrameResult::Create(outcome)));
            }
            create_input_stack_inner.borrow_mut().push(inputs.clone());

            let mut frame_or_result = prev_handle(ctx, inputs);
            if let Ok(FrameOrResult::Frame(frame)) = &mut frame_or_result {
                ctx.external
                    .get_inspector()
                    .initialize_interp(frame.interpreter_mut(), &mut ctx.evm)
            }
            frame_or_result
        },
    );

    // Call handler
    let call_input_stack_inner = call_input_stack.clone();
    let prev_handle = handler.execution.call.clone();
    handler.execution.call = Arc::new(move |ctx, mut inputs| {
        // Call inspector to change input or return outcome.
        let outcome = ctx.external.get_inspector().call(&mut ctx.evm, &mut inputs);
        call_input_stack_inner.borrow_mut().push(inputs.clone());
        if let Some(outcome) = outcome {
            return Ok(FrameOrResult::Result(FrameResult::Call(outcome)));
        }

        let mut frame_or_result = prev_handle(ctx, inputs);
        if let Ok(FrameOrResult::Frame(frame)) = &mut frame_or_result {
            ctx.external
                .get_inspector()
                .initialize_interp(frame.interpreter_mut(), &mut ctx.evm)
        }
        frame_or_result
    });

    // Calls inspector `eofcreate` and `initialize_interp` functions. Queues the inputs for the `eofcreate_end`` function.
    // Calls the old handler, and in case of inspector returning outcome,
    // returns the outcome without executing eofcreate.
    let eofcreate_input_stack_inner = eofcreate_input_stack.clone();
    let prev_handle = handler.execution.eofcreate.clone();
    handler.execution.eofcreate = Arc::new(move |ctx, mut inputs| {
        // Call inspector to change input or return outcome.
        let outcome = ctx
            .external
            .get_inspector()
            .eofcreate(&mut ctx.evm, &mut inputs);
        eofcreate_input_stack_inner
            .borrow_mut()
            .push(inputs.clone());
        if let Some(outcome) = outcome {
            return Ok(FrameOrResult::Result(FrameResult::EOFCreate(outcome)));
        }

        let mut frame_or_result = prev_handle(ctx, inputs);
        if let Ok(FrameOrResult::Frame(frame)) = &mut frame_or_result {
            ctx.external
                .get_inspector()
                .initialize_interp(frame.interpreter_mut(), &mut ctx.evm)
        }
        frame_or_result
    });

    // Pops eofcreate input from the stack and calls inspector `eofcreate_end` function.
    // preserve the old handler and calls it with the outcome.
    let eofcreate_input_stack_inner = eofcreate_input_stack.clone();
    let prev_handle = handler.execution.insert_eofcreate_outcome.clone();
    handler.execution.insert_eofcreate_outcome = Arc::new(move |ctx, frame, mut outcome| {
        let create_inputs = eofcreate_input_stack_inner.borrow_mut().pop().unwrap();
        outcome = ctx
            .external
            .get_inspector()
            .eofcreate_end(&mut ctx.evm, &create_inputs, outcome);
        prev_handle(ctx, frame, outcome)
    });

    // call outcome
    let call_input_stack_inner = call_input_stack.clone();
    let prev_handle = handler.execution.insert_call_outcome.clone();
    handler.execution.insert_call_outcome =
        Arc::new(move |ctx, frame, shared_memory, mut outcome| {
            let call_inputs = call_input_stack_inner.borrow_mut().pop().unwrap();
            outcome = ctx
                .external
                .get_inspector()
                .call_end(&mut ctx.evm, &call_inputs, outcome);
            prev_handle(ctx, frame, shared_memory, outcome)
        });

    // create outcome
    let create_input_stack_inner = create_input_stack.clone();
    let prev_handle = handler.execution.insert_create_outcome.clone();
    handler.execution.insert_create_outcome = Arc::new(move |ctx, frame, mut outcome| {
        let create_inputs = create_input_stack_inner.borrow_mut().pop().unwrap();
        outcome = ctx
            .external
            .get_inspector()
            .create_end(&mut ctx.evm, &create_inputs, outcome);
        prev_handle(ctx, frame, outcome)
    });

    // last frame outcome
    let prev_handle = handler.execution.last_frame_return.clone();
    handler.execution.last_frame_return = Arc::new(move |ctx, frame_result| {
        let inspector = ctx.external.get_inspector();
        match frame_result {
            FrameResult::Call(outcome) => {
                let call_inputs = call_input_stack.borrow_mut().pop().unwrap();
                *outcome = inspector.call_end(&mut ctx.evm, &call_inputs, outcome.clone());
            }
            FrameResult::Create(outcome) => {
                let create_inputs = create_input_stack.borrow_mut().pop().unwrap();
                *outcome = inspector.create_end(&mut ctx.evm, &create_inputs, outcome.clone());
            }
            FrameResult::EOFCreate(outcome) => {
                let eofcreate_inputs = eofcreate_input_stack.borrow_mut().pop().unwrap();
                *outcome =
                    inspector.eofcreate_end(&mut ctx.evm, &eofcreate_inputs, outcome.clone());
            }
        }
        prev_handle(ctx, frame_result)
    });
}

fn inspector_instruction<EvmWiringT>(
    prev: &DynInstruction<'_, Context<EvmWiringT>>,
    interpreter: &mut Interpreter,
    host: &mut Context<EvmWiringT>,
) where
    EvmWiringT: EvmWiring,
    EvmWiringT::ExternalContext: GetInspector<EvmWiringT>,
{
    // SAFETY: as the PC was already incremented we need to subtract 1 to preserve the
    // old Inspector behavior.
    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.sub(1) };

    // Call step.
    host.external
        .get_inspector()
        .step(interpreter, &mut host.evm);
    if interpreter.instruction_result != InstructionResult::Continue {
        return;
    }

    // Reset PC to previous value.
    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.add(1) };

    // Execute instruction.
    prev(interpreter, host);

    // Call step_end.
    host.external
        .get_inspector()
        .step_end(interpreter, &mut host.evm);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inspector_handle_register, inspectors::NoOpInspector};
    use database::BenchmarkDB;
    use revm::{
        bytecode::{opcode, Bytecode},
        database_interface::EmptyDB,
        interpreter::{CallInputs, CallOutcome, CreateInputs, CreateOutcome},
        primitives::{address, Bytes, TxKind},
        wiring::{DefaultEthereumWiring, EthereumWiring, EvmWiring as PrimitiveEvmWiring},
        Evm, EvmContext, EvmWiring,
    };

    type TestEvmWiring = DefaultEthereumWiring;

    #[derive(Default, Debug)]
    struct StackInspector {
        initialize_interp_called: bool,
        step: u32,
        step_end: u32,
        call: bool,
        call_end: bool,
    }

    impl<EvmWiringT: EvmWiring> Inspector<EvmWiringT> for StackInspector {
        fn initialize_interp(
            &mut self,
            _interp: &mut Interpreter,
            _context: &mut EvmContext<EvmWiringT>,
        ) {
            if self.initialize_interp_called {
                unreachable!("initialize_interp should not be called twice")
            }
            self.initialize_interp_called = true;
        }

        fn step(&mut self, _interp: &mut Interpreter, _context: &mut EvmContext<EvmWiringT>) {
            self.step += 1;
        }

        fn step_end(&mut self, _interp: &mut Interpreter, _context: &mut EvmContext<EvmWiringT>) {
            self.step_end += 1;
        }

        fn call(
            &mut self,
            context: &mut EvmContext<EvmWiringT>,
            _call: &mut CallInputs,
        ) -> Option<CallOutcome> {
            if self.call {
                unreachable!("call should not be called twice")
            }
            self.call = true;
            assert_eq!(context.journaled_state.depth(), 0);
            None
        }

        fn call_end(
            &mut self,
            context: &mut EvmContext<EvmWiringT>,
            _inputs: &CallInputs,
            outcome: CallOutcome,
        ) -> CallOutcome {
            if self.call_end {
                unreachable!("call_end should not be called twice")
            }
            assert_eq!(context.journaled_state.depth(), 0);
            self.call_end = true;
            outcome
        }

        fn create(
            &mut self,
            context: &mut EvmContext<EvmWiringT>,
            _call: &mut CreateInputs,
        ) -> Option<CreateOutcome> {
            assert_eq!(context.journaled_state.depth(), 0);
            None
        }

        fn create_end(
            &mut self,
            context: &mut EvmContext<EvmWiringT>,
            _inputs: &CreateInputs,
            outcome: CreateOutcome,
        ) -> CreateOutcome {
            assert_eq!(context.journaled_state.depth(), 0);
            outcome
        }
    }

    #[test]
    fn test_inspector_handlers() {
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

        let mut evm = Evm::<EthereumWiring<BenchmarkDB, StackInspector>>::builder()
            .with_default_ext_ctx()
            .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
            .with_external_context(StackInspector::default())
            .modify_tx_env(|tx| {
                *tx = <TestEvmWiring as PrimitiveEvmWiring>::Transaction::default();

                tx.caller = address!("1000000000000000000000000000000000000000");
                tx.transact_to = TxKind::Call(address!("0000000000000000000000000000000000000000"));
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

    #[test]
    fn test_inspector_reg() {
        let mut noop = NoOpInspector;
        let _evm: Evm<'_, EthereumWiring<EmptyDB, &mut NoOpInspector>> = Evm::builder()
            .with_default_db()
            .with_external_context(&mut noop)
            .append_handler_register(inspector_handle_register)
            .build();
    }
}
