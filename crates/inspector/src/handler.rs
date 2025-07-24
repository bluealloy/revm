use crate::{Inspector, InspectorEvmTr, JournalExt};
use context::{result::ExecutionResult, ContextTr, JournalEntry, Transaction};
use handler::{evm::FrameTr, EvmTr, FrameResult, Handler, ItemOrResult};
use interpreter::{
    instructions::InstructionTable,
    interpreter_types::{Jumps, LoopControl},
    FrameInput, Host, InitialAndFloorGas, InstructionContext, InstructionResult, Interpreter,
    InterpreterAction, InterpreterTypes,
};
use state::bytecode::opcode;

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
/// * [`Handler::run_exec_loop`] replaced with [`InspectorHandler::inspect_run_exec_loop`]
///   * `run_exec_loop` calls `inspect_frame_init` and `inspect_frame_run` that call inspector inside.
pub trait InspectorHandler: Handler
where
    Self::Evm:
        InspectorEvmTr<Inspector: Inspector<<<Self as Handler>::Evm as EvmTr>::Context, Self::IT>>,
{
    /// The interpreter types used by this handler.
    type IT: InterpreterTypes;

    /// Entry point for inspection.
    ///
    /// This method is acts as [`Handler::run`] method for inspection.
    fn inspect_run(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ExecutionResult<Self::HaltReason>, Self::Error> {
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
    ) -> Result<ExecutionResult<Self::HaltReason>, Self::Error> {
        let init_and_floor_gas = self.validate(evm)?;
        let eip7702_refund = self.pre_execution(evm)? as i64;
        let mut frame_result = self.inspect_execution(evm, &init_and_floor_gas)?;
        self.post_execution(evm, &mut frame_result, init_and_floor_gas, eip7702_refund)?;
        self.execution_result(evm, frame_result)
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

        // Run execution loop
        let mut frame_result = self.inspect_run_exec_loop(evm, first_frame_input)?;

        // Handle last frame result
        self.last_frame_result(evm, &mut frame_result)?;
        Ok(frame_result)
    }

    /* FRAMES */

    /// Run inspection on execution loop.
    ///
    /// This method acts as [`Handler::run_exec_loop`] method for inspection.
    ///
    /// It will call:
    /// * [`Inspector::call`],[`Inspector::create`] to inspect call, create and eofcreate.
    /// * [`Inspector::call_end`],[`Inspector::create_end`] to inspect call, create and eofcreate end.
    /// * [`Inspector::initialize_interp`] to inspect initialized interpreter.
    fn inspect_run_exec_loop(
        &mut self,
        evm: &mut Self::Evm,
        first_frame_input: <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameInit,
    ) -> Result<FrameResult, Self::Error> {
        let res = evm.inspect_frame_init(first_frame_input)?;

        if let ItemOrResult::Result(frame_result) = res {
            return Ok(frame_result);
        }

        loop {
            let call_or_result = evm.inspect_frame_run()?;

            let result = match call_or_result {
                ItemOrResult::Item(init) => {
                    match evm.inspect_frame_init(init)? {
                        ItemOrResult::Item(_) => {
                            continue;
                        }
                        // Do not pop the frame since no new frame was created
                        ItemOrResult::Result(result) => result,
                    }
                }
                ItemOrResult::Result(result) => result,
            };

            if let Some(result) = evm.frame_return_result(result)? {
                return Ok(result);
            }
        }
    }
}

/// Handles the start of a frame by calling the appropriate inspector method.
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
        FrameInput::Empty => unreachable!(),
    }
    None
}

/// Handles the end of a frame by calling the appropriate inspector method.
pub fn frame_end<CTX, INTR: InterpreterTypes>(
    context: &mut CTX,
    inspector: &mut impl Inspector<CTX, INTR>,
    frame_input: &FrameInput,
    frame_output: &mut FrameResult,
) {
    match frame_output {
        FrameResult::Call(outcome) => {
            let FrameInput::Call(i) = frame_input else {
                panic!("FrameInput::Call expected {frame_input:?}");
            };
            inspector.call_end(context, i, outcome);
        }
        FrameResult::Create(outcome) => {
            let FrameInput::Create(i) = frame_input else {
                panic!("FrameInput::Create expected {frame_input:?}");
            };
            inspector.create_end(context, i, outcome);
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
    loop {
        inspector.step(interpreter, context);

        // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        let opcode = interpreter.bytecode.opcode();
        interpreter.bytecode.relative_jump(1);

        // Execute instruction.
        let instruction_context = InstructionContext {
            interpreter,
            host: context,
        };
        instructions[opcode as usize](instruction_context);

        if (opcode::LOG0..=opcode::LOG4).contains(&opcode) {
            inspect_log(interpreter, context, &mut inspector);
        }

        let done = interpreter.bytecode.is_end();
        if done {
            interpreter.bytecode.revert_to_previous_pointer();
        }

        inspector.step_end(interpreter, context);

        if done {
            break;
        }
    }

    let next_action = interpreter.take_next_action();

    // Handle selfdestruct.
    if let InterpreterAction::Return(result) = &next_action {
        if result.result == InstructionResult::SelfDestruct {
            inspect_selfdestruct(context, &mut inspector);
        }
    }

    next_action
}

#[inline(never)]
#[cold]
fn inspect_log<CTX, IT>(
    interpreter: &mut Interpreter<IT>,
    context: &mut CTX,
    inspector: &mut impl Inspector<CTX, IT>,
) where
    CTX: ContextTr<Journal: JournalExt> + Host,
    IT: InterpreterTypes,
{
    // `LOG*` instruction reverted.
    if interpreter
        .bytecode
        .action()
        .as_ref()
        .is_some_and(|x| x.is_return())
    {
        return;
    }

    let log = context.journal_mut().logs().last().unwrap().clone();
    inspector.log(interpreter, context, log);
}

#[inline(never)]
#[cold]
fn inspect_selfdestruct<CTX, IT>(context: &mut CTX, inspector: &mut impl Inspector<CTX, IT>)
where
    CTX: ContextTr<Journal: JournalExt> + Host,
    IT: InterpreterTypes,
{
    if let Some(
        JournalEntry::AccountDestroyed {
            address: contract,
            target: to,
            had_balance: balance,
            ..
        }
        | JournalEntry::BalanceTransfer {
            from: contract,
            to,
            balance,
            ..
        },
    ) = context.journal_mut().journal().last()
    {
        inspector.selfdestruct(*contract, *to, *balance);
    }
}
