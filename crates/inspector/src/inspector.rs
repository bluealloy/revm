use crate::{InspectorEvmTr, InspectorFrame};
use auto_impl::auto_impl;
use context::{result::ResultAndState, ContextTr, Database, Journal, JournalEntry, Transaction};
use handler::{EvmTr, Frame, FrameInitOrResult, FrameOrResult, FrameResult, Handler, ItemOrResult};
use interpreter::{
    instructions::InstructionTable,
    interpreter::EthInterpreter,
    interpreter_types::{Jumps, LoopControl},
    CallInputs, CallOutcome, CreateInputs, CreateOutcome, EOFCreateInputs, FrameInput, Host,
    InitialAndFloorGas, InstructionResult, Interpreter, InterpreterAction, InterpreterTypes,
};
use primitives::{Address, Log, U256};
use state::EvmState;
use std::{vec, vec::Vec};

/// EVM hooks into execution.
#[auto_impl(&mut, Box)]
pub trait Inspector<INTR: InterpreterTypes = EthInterpreter> {
    type Context<'context>;

    /// Called before the interpreter is initialized.
    ///
    /// If `interp.instruction_result` is set to anything other than [InstructionResult::Continue] then the execution of the interpreter
    /// is skipped.
    #[inline]
    fn initialize_interp(
        &mut self,
        interp: &mut Interpreter<INTR>,
        context: &mut Self::Context<'_>,
    ) {
        let _ = interp;
        let _ = context;
    }

    /// Called on each step of the interpreter.
    ///
    /// Information about the current execution, including the memory, stack and more is available
    /// on `interp` (see [Interpreter]).
    ///
    /// # Example
    ///
    /// To get the current opcode, use `interp.current_opcode()`.
    #[inline]
    fn step(&mut self, interp: &mut Interpreter<INTR>, context: &mut Self::Context<'_>) {
        let _ = interp;
        let _ = context;
    }

    /// Called after `step` when the instruction has been executed.
    ///
    /// Setting `interp.instruction_result` to anything other than [InstructionResult::Continue] alters the execution
    /// of the interpreter.
    #[inline]
    fn step_end(&mut self, interp: &mut Interpreter<INTR>, context: &mut Self::Context<'_>) {
        let _ = interp;
        let _ = context;
    }

    /// Called when a log is emitted.
    #[inline]
    fn log(&mut self, interp: &mut Interpreter<INTR>, context: &mut Self::Context<'_>, log: Log) {
        let _ = interp;
        let _ = context;
        let _ = log;
    }

    /// Called whenever a call to a contract is about to start.
    ///
    /// InstructionResulting anything other than [InstructionResult::Continue] overrides the result of the call.
    #[inline]
    fn call(
        &mut self,
        context: &mut Self::Context<'_>,
        inputs: &mut CallInputs,
    ) -> Option<CallOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when a call to a contract has concluded.
    ///
    /// The returned [CallOutcome] is used as the result of the call.
    ///
    /// This allows the inspector to modify the given `result` before returning it.
    #[inline]
    fn call_end(
        &mut self,
        context: &mut Self::Context<'_>,
        inputs: &CallInputs,
        outcome: &mut CallOutcome,
    ) {
        let _ = context;
        let _ = inputs;
        let _ = outcome;
    }

    /// Called when a contract is about to be created.
    ///
    /// If this returns `Some` then the [CreateOutcome] is used to override the result of the creation.
    ///
    /// If this returns `None` then the creation proceeds as normal.
    #[inline]
    fn create(
        &mut self,
        context: &mut Self::Context<'_>,
        inputs: &mut CreateInputs,
    ) -> Option<CreateOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when a contract has been created.
    ///
    /// InstructionResulting anything other than the values passed to this function (`(ret, remaining_gas,
    /// address, out)`) will alter the result of the create.
    #[inline]
    fn create_end(
        &mut self,
        context: &mut Self::Context<'_>,
        inputs: &CreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        let _ = context;
        let _ = inputs;
        let _ = outcome;
    }

    /// Called when EOF creating is called.
    ///
    /// This can happen from create TX or from EOFCREATE opcode.
    fn eofcreate(
        &mut self,
        context: &mut Self::Context<'_>,
        inputs: &mut EOFCreateInputs,
    ) -> Option<CreateOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when eof creating has ended.
    fn eofcreate_end(
        &mut self,
        context: &mut Self::Context<'_>,
        inputs: &EOFCreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        let _ = context;
        let _ = inputs;
        let _ = outcome;
    }

    /// Called when a contract has been self-destructed with funds transferred to target.
    #[inline]
    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        let _ = contract;
        let _ = target;
        let _ = value;
    }
}

#[auto_impl(&mut, Box)]
pub trait JournalExt {
    fn logs(&self) -> &[Log];

    fn last_journal(&self) -> &[JournalEntry];

    fn evm_state(&self) -> &EvmState;

    fn evm_state_mut(&mut self) -> &mut EvmState;
}

impl<DB: Database> JournalExt for Journal<DB> {
    #[inline]
    fn logs(&self) -> &[Log] {
        &self.logs
    }

    #[inline]
    fn last_journal(&self) -> &[JournalEntry] {
        self.journal.last().expect("Journal is never empty")
    }

    #[inline]
    fn evm_state(&self) -> &EvmState {
        &self.state
    }

    #[inline]
    fn evm_state_mut(&mut self) -> &mut EvmState {
        &mut self.state
    }
}

pub trait InspectorHandler: Handler
where
    Self::Evm: InspectorEvmTr<
        Inspector: for<'context> Inspector<
            Self::IT,
            Context<'context> = <<Self as Handler>::Evm as EvmTr>::Context,
        >,
    >,
    Self::Frame: InspectorFrame<IT = Self::IT>,
{
    type IT: InterpreterTypes;

    fn inspect_run(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        match self.inspect_run_without_catch_error(evm) {
            Ok(output) => Ok(output),
            Err(e) => self.catch_error(evm, e),
        }
    }

    fn inspect_run_without_catch_error(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        let init_and_floor_gas = self.validate(evm)?;
        let eip7702_refund = self.pre_execution(evm)? as i64;
        let exec_result = self.inspect_execution(evm, &init_and_floor_gas);
        self.post_execution(evm, exec_result?, init_and_floor_gas, eip7702_refund)
    }

    fn inspect_execution(
        &mut self,
        evm: &mut Self::Evm,
        init_and_floor_gas: &InitialAndFloorGas,
    ) -> Result<FrameResult, Self::Error> {
        let gas_limit = evm.ctx().tx().gas_limit() - init_and_floor_gas.initial_gas;

        // Create first frame action
        let first_frame_input = self.first_frame_input(evm, gas_limit)?;
        let first_frame = self.inspect_first_frame_init(evm, first_frame_input)?;

        let mut frame_result = match first_frame {
            ItemOrResult::Item(frame) => self.inspect_run_exec_loop(evm, frame)?,
            ItemOrResult::Result(result) => result,
        };

        self.last_frame_result(evm, &mut frame_result)?;
        Ok(frame_result)
    }

    /* FRAMES */
    fn inspect_first_frame_init(
        &mut self,
        evm: &mut Self::Evm,
        mut frame_input: <Self::Frame as Frame>::FrameInit,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        let (ctx, inspector) = evm.ctx_inspector();
        if let Some(output) = frame_start(ctx, inspector, &mut frame_input) {
            return Ok(ItemOrResult::Result(output));
        }
        let mut ret = self.first_frame_init(evm, frame_input.clone());

        // only if new frame is created call initialize_interp hook.
        if let Ok(ItemOrResult::Item(frame)) = &mut ret {
            let (context, inspector) = evm.ctx_inspector();
            inspector.initialize_interp(frame.interpreter(), context);
        } else if let Ok(ItemOrResult::Result(result)) = &mut ret {
            let (context, inspector) = evm.ctx_inspector();
            frame_end(context, inspector, &frame_input, result);
        }
        ret
    }

    #[inline]
    fn inspect_frame_call(
        &mut self,
        frame: &mut Self::Frame,
        evm: &mut Self::Evm,
    ) -> Result<FrameInitOrResult<Self::Frame>, Self::Error> {
        frame.run_inspect(evm)
    }

    fn inspect_run_exec_loop(
        &mut self,
        evm: &mut Self::Evm,
        frame: Self::Frame,
    ) -> Result<FrameResult, Self::Error> {
        let mut frame_stack: Vec<Self::Frame> = vec![frame];
        loop {
            let frame = frame_stack.last_mut().unwrap();
            let call_or_result = self.inspect_frame_call(frame, evm)?;

            let result = match call_or_result {
                ItemOrResult::Item(mut init) => {
                    let (context, inspector) = evm.ctx_inspector();
                    if let Some(output) = frame_start(context, inspector, &mut init) {
                        output
                    } else {
                        match self.frame_init(frame, evm, init.clone())? {
                            ItemOrResult::Item(mut new_frame) => {
                                // only if new frame is created call initialize_interp hook.
                                let (context, inspector) = evm.ctx_inspector();
                                inspector.initialize_interp(new_frame.interpreter(), context);
                                frame_stack.push(new_frame);
                                continue;
                            }
                            // Dont pop the frame as new frame was not created.
                            ItemOrResult::Result(mut result) => {
                                let (context, inspector) = evm.ctx_inspector();
                                frame_end(context, inspector, &init, &mut result);
                                result
                            }
                        }
                    }
                }
                ItemOrResult::Result(mut result) => {
                    let (context, inspector) = evm.ctx_inspector();
                    frame_end(context, inspector, frame.frame_input(), &mut result);

                    // Pop frame that returned result
                    frame_stack.pop();
                    result
                }
            };

            let Some(frame) = frame_stack.last_mut() else {
                return Ok(result);
            };

            self.frame_return_result(frame, evm, result)?;
        }
    }
}

fn frame_start<'context, CTX, INTR: InterpreterTypes>(
    context: &'context mut CTX,
    inspector: &mut impl Inspector<INTR, Context<'context> = CTX>,
    frame_input: &mut FrameInput,
) -> Option<FrameResult> {
    match frame_input {
        FrameInput::Call(i) => {
            if let Some(output) = inspector.call(context, i) {
                return Some(FrameResult::Call(output));
            }
        }
        FrameInput::Create(i) => {
            if let Some(output) = inspector.create(context, i) {
                return Some(FrameResult::Create(output));
            }
        }
        FrameInput::EOFCreate(i) => {
            if let Some(output) = inspector.eofcreate(context, i) {
                return Some(FrameResult::EOFCreate(output));
            }
        }
    }
    None
}

fn frame_end<'context, CTX, INTR: InterpreterTypes>(
    context: &'context mut CTX,
    inspector: &mut impl Inspector<INTR, Context<'context> = CTX>,
    frame_input: &FrameInput,
    frame_output: &mut FrameResult,
) {
    match frame_output {
        FrameResult::Call(outcome) => {
            let FrameInput::Call(i) = frame_input else {
                panic!("FrameInput::Call expected");
            };
            inspector.call_end(context, i, outcome);
        }
        FrameResult::Create(outcome) => {
            let FrameInput::Create(i) = frame_input else {
                panic!("FrameInput::Create expected");
            };
            inspector.create_end(context, i, outcome);
        }
        FrameResult::EOFCreate(outcome) => {
            let FrameInput::EOFCreate(i) = frame_input else {
                panic!("FrameInput::EofCreate expected");
            };
            inspector.eofcreate_end(context, i, outcome);
        }
    }
}

pub fn inspect_instructions<'context, CTX, IT>(
    context: &'context mut CTX,
    interpreter: &mut Interpreter<IT>,
    mut inspector: impl Inspector<IT, Context<'context> = CTX>,
    instructions: &InstructionTable<IT, CTX>,
) -> InterpreterAction
where
    CTX: ContextTr<Journal: JournalExt> + Host,
    IT: InterpreterTypes,
{
    interpreter.reset_control();

    let mut log_num = context.journal().logs().len();
    // Main loop
    while interpreter.control.instruction_result().is_continue() {
        // Get current opcode.
        let opcode = interpreter.bytecode.opcode();

        // Call Inspector step.
        inspector.step(interpreter, context);
        if interpreter.control.instruction_result() != InstructionResult::Continue {
            break;
        }

        // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        interpreter.bytecode.relative_jump(1);

        // Execute instruction.
        instructions[opcode as usize](interpreter, context);

        // check if new log is added
        let new_log = context.journal().logs().len();
        if log_num < new_log {
            // as there is a change in log number this means new log is added
            let log = context.journal().logs().last().unwrap().clone();
            inspector.log(interpreter, context, log);
            log_num = new_log;
        }

        // Call step_end.
        inspector.step_end(interpreter, context);
    }

    let next_action = interpreter.take_next_action();

    // handle selfdestruct
    if let InterpreterAction::Return { result } = &next_action {
        if result.result == InstructionResult::SelfDestruct {
            match context.journal().last_journal().last() {
                Some(JournalEntry::AccountDestroyed {
                    address,
                    target,
                    had_balance,
                    ..
                }) => {
                    inspector.selfdestruct(*address, *target, *had_balance);
                }
                Some(JournalEntry::BalanceTransfer {
                    from, to, balance, ..
                }) => {
                    inspector.selfdestruct(*from, *to, *balance);
                }
                _ => {}
            }
        }
    }

    next_action
}
