use auto_impl::auto_impl;
use core::mem::MaybeUninit;
use revm::{
    bytecode::opcode::OpCode,
    context::JournaledState,
    context_interface::{
        block::BlockSetter,
        journaled_state::{AccountLoad, Eip7702CodeLoad},
        transaction::TransactionSetter,
        BlockGetter, CfgGetter, DatabaseGetter, ErrorGetter, Journal, JournalDBError,
        JournalGetter, TransactionGetter,
    },
    database_interface::{Database, EmptyDB},
    handler::{
        EthExecution, EthFrame, EthHandler, EthPostExecution, EthPreExecution,
        EthPrecompileProvider, EthValidation, FrameResult,
    },
    handler_interface::{Frame, FrameOrResultGen, PrecompileProvider},
    interpreter::{
        instructions::host::{log, selfdestruct},
        interpreter::{EthInterpreter, InstructionProvider},
        interpreter_types::{Jumps, LoopControl},
        table::{self, CustomInstruction},
        CallInputs, CallOutcome, CreateInputs, CreateOutcome, EOFCreateInputs, FrameInput, Host,
        Instruction, InstructionResult, Interpreter, InterpreterTypes, SStoreResult,
        SelfDestructResult, StateLoad,
    },
    precompile::PrecompileErrors,
    primitives::{Address, Bytes, Log, B256, U256},
    state::EvmState,
    Context, Error, Evm, JournalEntry,
};
use std::{rc::Rc, vec::Vec};

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

#[derive(Clone)]
pub struct InspectorInstruction<IT: InterpreterTypes, HOST> {
    pub instruction: fn(&mut Interpreter<IT>, &mut HOST),
}

impl<IT: InterpreterTypes, HOST> CustomInstruction for InspectorInstruction<IT, HOST>
where
    HOST: InspectorCtx<IT = IT>,
{
    type Wire = IT;
    type Host = HOST;

    fn exec(&self, interpreter: &mut Interpreter<Self::Wire>, host: &mut Self::Host) {
        // SAFETY: As the PC was already incremented we need to subtract 1 to preserve the
        // old Inspector behavior.
        interpreter.bytecode.relative_jump(-1);

        // Call step.
        host.step(interpreter);
        if interpreter.control.instruction_result() != InstructionResult::Continue {
            return;
        }

        // Reset PC to previous value.
        interpreter.bytecode.relative_jump(1);

        // Execute instruction.
        (self.instruction)(interpreter, host);

        // Call step_end.
        host.step_end(interpreter);
    }

    fn from_base(instruction: Instruction<Self::Wire, Self::Host>) -> Self {
        Self { instruction }
    }
}

pub struct InspectorEthFrame<CTX, ERROR, PRECOMPILE>
where
    CTX: Host,
{
    // TODO : For now, hardcode the InstructionProvider. But in future this should be configurable as generic parameter.
    pub eth_frame: EthFrame<
        CTX,
        ERROR,
        EthInterpreter<()>,
        PRECOMPILE,
        InspectorInstructionProvider<EthInterpreter<()>, CTX>,
    >,
}

impl<CTX, ERROR, PRECOMPILE> Frame for InspectorEthFrame<CTX, ERROR, PRECOMPILE>
where
    CTX: TransactionGetter
        + ErrorGetter<Error = ERROR>
        + BlockGetter
        + JournalGetter
        + CfgGetter
        + JournalExtGetter
        + Host
        + InspectorCtx<IT = EthInterpreter>,
    ERROR: From<JournalDBError<CTX>> + From<PrecompileErrors>,
    PRECOMPILE: PrecompileProvider<Context = CTX, Error = ERROR>,
{
    type Context = CTX;
    type Error = ERROR;
    type FrameInit = FrameInput;
    type FrameResult = FrameResult;

    fn init_first(
        context: &mut CTX,
        mut frame_input: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        if let Some(output) = context.frame_start(&mut frame_input) {
            return Ok(FrameOrResultGen::Result(output));
        }
        let mut ret = EthFrame::init_first(context, frame_input)
            .map(|frame| frame.map_frame(|eth_frame| Self { eth_frame }));

        match &mut ret {
            Ok(FrameOrResultGen::Result(res)) => {
                context.frame_end(res);
            }
            Ok(FrameOrResultGen::Frame(frame)) => {
                context.initialize_interp(&mut frame.eth_frame.interpreter);
            }
            _ => (),
        }
        ret
    }

    fn final_return(
        context: &mut Self::Context,
        result: &mut Self::FrameResult,
    ) -> Result<(), Self::Error> {
        context.frame_end(result);
        Ok(())
    }

    fn init(
        &self,
        context: &mut CTX,
        mut frame_input: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        if let Some(output) = context.frame_start(&mut frame_input) {
            return Ok(FrameOrResultGen::Result(output));
        }
        let mut ret = self
            .eth_frame
            .init(context, frame_input)
            .map(|frame| frame.map_frame(|eth_frame| Self { eth_frame }));

        if let Ok(FrameOrResultGen::Frame(frame)) = &mut ret {
            context.initialize_interp(&mut frame.eth_frame.interpreter);
        }
        ret
    }

    fn run(
        &mut self,
        context: &mut CTX,
    ) -> Result<FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error> {
        self.eth_frame.run(context)
    }

    fn return_result(
        &mut self,
        context: &mut CTX,
        mut result: Self::FrameResult,
    ) -> Result<(), Self::Error> {
        context.frame_end(&mut result);
        self.eth_frame.return_result(context, result)
    }
}

pub type InspCtxType<INSP, DB, CTX> = InspectorContext<INSP, DB, CTX>;

pub type InspectorMainEvm<INSP, CTX, DB = EmptyDB> = Evm<
    Error<DB>,
    InspCtxType<INSP, DB, CTX>,
    EthHandler<
        InspCtxType<INSP, DB, CTX>,
        Error<DB>,
        EthValidation<InspCtxType<INSP, DB, CTX>, Error<DB>>,
        EthPreExecution<InspCtxType<INSP, DB, CTX>, Error<DB>>,
        InspectorEthExecution<InspCtxType<INSP, DB, CTX>, Error<DB>>,
    >,
>;

/// Function to create Inspector Handler.
pub fn inspector_handler<CTX: Host, ERROR, PRECOMPILE>() -> InspectorHandler<CTX, ERROR, PRECOMPILE>
{
    EthHandler::new(
        EthValidation::new(),
        EthPreExecution::new(),
        EthExecution::<_, _, InspectorEthFrame<_, _, PRECOMPILE>>::new(),
        EthPostExecution::new(),
    )
}

/// Composed type for Inspector Execution handler.
pub type InspectorEthExecution<CTX, ERROR, PRECOMPILE = EthPrecompileProvider<CTX, ERROR>> =
    EthExecution<CTX, ERROR, InspectorEthFrame<CTX, ERROR, PRECOMPILE>>;

/// Composed type for Inspector Handler.
pub type InspectorHandler<CTX, ERROR, PRECOMPILE> = EthHandler<
    CTX,
    ERROR,
    EthValidation<CTX, ERROR>,
    EthPreExecution<CTX, ERROR>,
    InspectorEthExecution<CTX, ERROR, PRECOMPILE>,
>;
