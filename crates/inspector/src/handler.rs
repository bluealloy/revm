use crate::{Inspector, InspectorEvmTr, InspectorFrame, JournalExt};
use context::{result::ResultAndState, ContextTr, JournalEntry, Transaction};
use handler::{EvmTr, Frame, FrameInitOrResult, FrameOrResult, FrameResult, Handler, ItemOrResult};
use interpreter::{
    instructions::InstructionTable,
    interpreter_types::{Jumps, LoopControl},
    FrameInput, Host, InitialAndFloorGas, InstructionResult, Interpreter, InterpreterAction,
    InterpreterTypes,
};

use std::{vec, vec::Vec};

/// Trait that extends [`Handler`] with inspection functionality.
///
/// Similar how [`Handler::run`] method serves as the entry point,
/// [`InspectorHandler::inspect_run`] method serves as the entry point for inspection.
///
/// Notice that when inspection is run it skips few functions from handler, this can be
/// a problem if custom EVM is implemented and some of skipped functions have changed logic.
/// For custom EVM, those changed functions would need to be also changed in [`InspectorHandler`].
///
/// List of functions that are skipped in [`InspectorHandler`]:
/// * [`Handler::run`] replaced with [`InspectorHandler::inspect_run`]
/// * [`Handler::run_without_catch_error`] replaced with [`InspectorHandler::inspect_run_without_catch_error`]
/// * [`Handler::execution`] replaced with [`InspectorHandler::inspect_execution`]
/// * [`Handler::first_frame_init`] replaced with [`InspectorHandler::inspect_first_frame_init`]
/// * [`Handler::frame_call`] replaced with [`InspectorHandler::inspect_frame_call`]
/// * [`Handler::run_exec_loop`] replaced with [`InspectorHandler::inspect_run_exec_loop`]
pub trait InspectorHandler: Handler
where
    Self::Evm:
        InspectorEvmTr<Inspector: Inspector<<<Self as Handler>::Evm as EvmTr>::Context, Self::IT>>,
    Self::Frame: InspectorFrame<IT = Self::IT>,
{
    type IT: InterpreterTypes;

    /// Entry point for inspection.
    ///
    /// This method is acts as [`Handler::run`] method for inspection.
    fn inspect_run(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        match self.inspect_run_without_catch_error(evm) {
            Ok(output) => Ok(output),
            Err(e) => self.catch_error(evm, e),
        }
    }

    /// Run inspection without catching error.
    ///
    /// This method is acts as [`Handler::run_without_catch_error`] method for inspection.
    fn inspect_run_without_catch_error(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        let init_and_floor_gas = self.validate(evm)?;
        let eip7702_refund = self.pre_execution(evm)? as i64;
        let exec_result = self.inspect_execution(evm, &init_and_floor_gas);
        self.post_execution(evm, exec_result?, init_and_floor_gas, eip7702_refund)
    }

    /// Run execution loop with inspection support
    ///
    /// This method acts as [`Handler::execution`] method for inspection.
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

    /// Initialize first frame.
    ///
    /// This method replaces the [`Handler::first_frame_init`] method from [`Handler`].
    ///
    /// * It calls [`Inspector::call`]/[`Inspector::create`]/[`Inspector::eofcreate`] methods to allow inspection of
    ///   the frame and its modification.
    /// * If new frame is created a [`Inspector::initialize_interp`] method will be called.
    /// * If creation of new frame returns the result, the [`Inspector`] `_end` methods will be called.
    fn inspect_first_frame_init(
        &mut self,
        evm: &mut Self::Evm,
        mut frame_input: <Self::Frame as Frame>::FrameInit,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        let (ctx, inspector) = evm.ctx_inspector();
        if let Some(mut output) = frame_start(ctx, inspector, &mut frame_input) {
            frame_end(ctx, inspector, &frame_input, &mut output);
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

    /// Run inspection on frame.
    ///
    /// This method acts as [`Handler::frame_call`] method for inspection.
    ///
    /// Internally it will call [`Inspector::step`], [`Inspector::step_end`] for each instruction.
    /// And [`Inspector::log`],[`Inspector::selfdestruct`] for each log and selfdestruct instruction.
    #[inline]
    fn inspect_frame_call(
        &mut self,
        frame: &mut Self::Frame,
        evm: &mut Self::Evm,
    ) -> Result<FrameInitOrResult<Self::Frame>, Self::Error> {
        frame.run_inspect(evm)
    }

    /// Run inspection on execution loop.
    ///
    /// This method acts as [`Handler::run_exec_loop`] method for inspection.
    ///
    /// It will call:
    /// * [`InspectorHandler::inspect_frame_call`] to inspect Interpreter execution loop.
    /// * [`Inspector::call`],[`Inspector::create`],[`Inspector::eofcreate`] to inspect call, create and eofcreate.
    /// * [`Inspector::call_end`],[`Inspector::create_end`],[`Inspector::eofcreate_end`] to inspect call, create and eofcreate end.
    /// * [`Inspector::initialize_interp`] to inspect initialized interpreter.
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
                    if let Some(mut output) = frame_start(context, inspector, &mut init) {
                        frame_end(context, inspector, &init, &mut output);
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

pub fn frame_start<CTX, INTR: InterpreterTypes>(
    context: &mut CTX,
    inspector: &mut impl Inspector<CTX, INTR>,
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

pub fn frame_end<CTX, INTR: InterpreterTypes>(
    context: &mut CTX,
    inspector: &mut impl Inspector<CTX, INTR>,
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

/// Run Interpreter loop with inspection support.
///
/// This function is used to inspect the Interpreter loop.
/// It will call [`Inspector::step`] and [`Inspector::step_end`] after each instruction.
/// And [`Inspector::log`],[`Inspector::selfdestruct`] for each log and selfdestruct instruction.
pub fn inspect_instructions<CTX, IT>(
    context: &mut CTX,
    interpreter: &mut Interpreter<IT>,
    mut inspector: impl Inspector<CTX, IT>,
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
            match context.journal().journal().last() {
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
