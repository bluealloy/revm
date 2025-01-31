use crate::{
    exec::{InspectorFrameContext, InstructionGetter},
    inspector_context::InspectorInnerCtx,
    journal::{JournalExt, JournalExtGetter},
};
use auto_impl::auto_impl;
use revm::{
    context_interface::{CfgGetter, Journal},
    database_interface::Database,
    handler::{
        handler::{EthContext, EthError, EthHandler, FrameContextTrait},
        instructions::InstructionExecutor,
        EthFrame, FrameResult,
    },
    handler_interface::{Frame, ItemOrResult},
    interpreter::{
        CallInputs, CallOutcome, CreateInputs, CreateOutcome, EOFCreateInputs, FrameInput, Host,
        Interpreter, InterpreterTypes,
    },
    primitives::{Address, Log, U256},
    Context,
};

/// EVM [Interpreter] callbacks.
#[auto_impl(&mut, Box)]
pub trait Inspector<CTX, INTR: InterpreterTypes> {
    /// Called before the interpreter is initialized.
    ///
    /// If `interp.instruction_result` is set to anything other than [revm::interpreter::InstructionResult::Continue] then the execution of the interpreter
    /// is skipped.
    #[inline]
    fn initialize_interp(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
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
    fn step(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let _ = interp;
        let _ = context;
    }

    /// Called after `step` when the instruction has been executed.
    ///
    /// Setting `interp.instruction_result` to anything other than [revm::interpreter::InstructionResult::Continue] alters the execution
    /// of the interpreter.
    #[inline]
    fn step_end(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let _ = interp;
        let _ = context;
    }

    /// Called when a log is emitted.
    #[inline]
    fn log(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX, log: &Log) {
        let _ = interp;
        let _ = context;
        let _ = log;
    }

    /// Called whenever a call to a contract is about to start.
    ///
    /// InstructionResulting anything other than [revm::interpreter::InstructionResult::Continue] overrides the result of the call.
    #[inline]
    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
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
    fn call_end(&mut self, context: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {
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
    fn create(&mut self, context: &mut CTX, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
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
        context: &mut CTX,
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
        context: &mut CTX,
        inputs: &mut EOFCreateInputs,
    ) -> Option<CreateOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when eof creating has ended.
    fn eofcreate_end(
        &mut self,
        context: &mut CTX,
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

/// Provides access to an `Inspector` instance.
pub trait GetInspector<CTX, INTR: InterpreterTypes> {
    /// Returns the associated `Inspector`.
    fn get_inspector(&mut self) -> &mut impl Inspector<CTX, INTR>;
}

#[auto_impl(&mut, Box)]
pub trait InspectorCtx {
    type IT: InterpreterTypes;

    fn step(&mut self, interp: &mut Interpreter<Self::IT>);
    fn step_end(&mut self, interp: &mut Interpreter<Self::IT>);
    fn initialize_interp(&mut self, interp: &mut Interpreter<Self::IT>);
    fn frame_start(&mut self, frame_input: &mut FrameInput) -> Option<FrameResult>;
    fn frame_end(&mut self, frame_output: &mut FrameResult);
    fn inspector_selfdestruct(&mut self, contract: Address, target: Address, value: U256);
    fn inspector_log(&mut self, interp: &mut Interpreter<Self::IT>, log: &Log);
}

impl<CTX, INTR: InterpreterTypes, INSP: Inspector<CTX, INTR>> GetInspector<CTX, INTR> for INSP {
    #[inline]
    fn get_inspector(&mut self) -> &mut impl Inspector<CTX, INTR> {
        self
    }
}

impl<BLOCK, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB> + JournalExt, CHAIN>
    JournalExtGetter for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type JournalExt = JOURNAL;

    fn journal_ext(&self) -> &Self::JournalExt {
        &self.journaled_state
    }
}

pub struct InspectorHandlerImpl<CTX, ERROR, FRAME, HANDLER, FRAMECTX> {
    pub handler: HANDLER,
    _phantom: core::marker::PhantomData<(CTX, ERROR, FRAME, FRAMECTX)>,
}

impl<CTX, ERROR, FRAME, HANDLER, FRAMECTX>
    InspectorHandlerImpl<CTX, ERROR, FRAME, HANDLER, FRAMECTX>
{
    pub fn new(handler: HANDLER) -> Self {
        Self {
            handler,
            _phantom: core::marker::PhantomData,
        }
    }
}

pub trait FrameInterpreterGetter {
    type IT: InterpreterTypes;

    fn interpreter(&mut self) -> &mut Interpreter<Self::IT>;
}

impl<CTX, ERROR, IW: InterpreterTypes, FRAMECTX> FrameInterpreterGetter
    for EthFrame<CTX, ERROR, IW, FRAMECTX>
{
    type IT = IW;

    fn interpreter(&mut self) -> &mut Interpreter<Self::IT> {
        &mut self.interpreter
    }
}

impl<CTX, ERROR, FRAME, HANDLER, FRAMECTX, INTR> EthHandler
    for InspectorHandlerImpl<CTX, ERROR, FRAME, HANDLER, FRAMECTX>
where
    CTX: EthContext
        + InspectorCtx<IT = INTR>
        + JournalExtGetter
        + InspectorInnerCtx<Context: Host>
        + CfgGetter<Cfg = <FRAMECTX::Context as CfgGetter>::Cfg>,
    INTR: InterpreterTypes,
    ERROR: EthError<CTX>,
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    FRAME: Frame<
            Context = CTX,
            Error = ERROR,
            FrameResult = FrameResult,
            FrameInit = FrameInput,
            FrameContext = InspectorFrameContext<CTX, FRAMECTX>,
        > + FrameInterpreterGetter<IT = INTR>,
    FRAMECTX: FrameContextTrait<
        Context = <CTX as InspectorInnerCtx>::Context,
        Instructions: InstructionExecutor
                          + InstructionGetter<Context = <CTX as InspectorInnerCtx>::Context>,
    >,
    HANDLER: EthHandler<Context = CTX, Error = ERROR, Frame = FRAME>,
{
    type Context = CTX;
    type Error = ERROR;
    type Frame = FRAME;
    type FrameContext = InspectorFrameContext<CTX, FRAMECTX>;
    type HaltReason = <HANDLER as EthHandler>::HaltReason;

    fn frame_init_first(
        &mut self,
        context: &mut Self::Context,
        frame_context: &mut <<Self as EthHandler>::Frame as Frame>::FrameContext,
        mut frame_input: <<Self as EthHandler>::Frame as Frame>::FrameInit,
    ) -> Result<
        ItemOrResult<
            <Self as EthHandler>::Frame,
            <<Self as EthHandler>::Frame as Frame>::FrameResult,
        >,
        Self::Error,
    > {
        if let Some(output) = context.frame_start(&mut frame_input) {
            return Ok(ItemOrResult::Result(output));
        }
        let mut ret = self
            .handler
            .frame_init_first(context, frame_context, frame_input);

        // only if new frame is created call initialize_interp hook.
        if let Ok(ItemOrResult::Item(frame)) = &mut ret {
            context.initialize_interp(frame.interpreter());
        }
        ret
    }

    fn frame_init(
        &mut self,
        frame: &Self::Frame,
        context: &mut Self::Context,
        frame_context: &mut <<Self as EthHandler>::Frame as Frame>::FrameContext,
        mut frame_input: <<Self as EthHandler>::Frame as Frame>::FrameInit,
    ) -> Result<
        ItemOrResult<
            <Self as EthHandler>::Frame,
            <<Self as EthHandler>::Frame as Frame>::FrameResult,
        >,
        Self::Error,
    > {
        if let Some(output) = context.frame_start(&mut frame_input) {
            return Ok(ItemOrResult::Result(output));
        }
        let mut ret = self
            .handler
            .frame_init(frame, context, frame_context, frame_input);

        // only if new frame is created call initialize_interp hook.
        if let Ok(ItemOrResult::Item(frame)) = &mut ret {
            context.initialize_interp(frame.interpreter());
        }
        ret
    }

    fn frame_return_result(
        &mut self,
        frame: &mut Self::Frame,
        context: &mut Self::Context,
        frame_context: &mut <<Self as EthHandler>::Frame as Frame>::FrameContext,
        mut result: <<Self as EthHandler>::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        context.frame_end(&mut result);
        self.handler
            .frame_return_result(frame, context, frame_context, result)
    }

    fn last_frame_result(
        &self,
        context: &mut Self::Context,
        frame_context: &mut <Self::Frame as Frame>::FrameContext,
        frame_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        context.frame_end(frame_result);
        self.handler
            .last_frame_result(context, frame_context, frame_result)
    }
}
