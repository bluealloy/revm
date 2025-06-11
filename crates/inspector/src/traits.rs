use context::{result::FromStringError, ContextTr};
use handler::{
    instructions::InstructionProvider, ContextTrDbError, EthFrame, EvmTr, Frame, FrameInitOrResult,
    PrecompileProvider,
};
use interpreter::{
    interpreter::EthInterpreter, FrameInput, Interpreter, InterpreterResult, InterpreterTypes,
};

/// Inspector EVM trait. Extends the [`EvmTr`] trait with inspector related methods.
///
/// It contains execution of interpreter with [`crate::Inspector`] calls [`crate::Inspector::step`] and [`crate::Inspector::step_end`] calls.
///
/// It is used inside [`crate::InspectorHandler`] to extend evm with support for inspection.
pub trait InspectorEvmTr: EvmTr {
    type Inspector;

    /// Returns a mutable reference to the inspector.
    fn inspector(&mut self) -> &mut Self::Inspector;

    /// Returns a tuple of mutable references to the context and the inspector.
    ///
    /// Useful when you want to allow inspector to modify the context.
    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector);

    /// Runs the inspector on the interpreter.
    ///
    /// This function is called by the EVM when it needs to inspect the Interpreter loop.
    /// It is responsible for calling the inspector's methods and instructions from table.
    fn run_inspect_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output;
}

/// Traits that extends the Frame with additional functionality that is needed for inspection
///
/// It is implemented for [`EthFrame`] as default Ethereum frame implementation.
pub trait InspectorFrame: Frame {
    type IT: InterpreterTypes;

    /// It runs the frame in inspection mode.
    ///
    /// This will internally call [`InspectorEvmTr::run_inspect_interpreter`]
    fn run_inspect(&mut self, evm: &mut Self::Evm) -> Result<FrameInitOrResult<Self>, Self::Error>;

    /// Returns a mutable reference to the interpreter.
    fn interpreter(&mut self) -> &mut Interpreter<Self::IT>;

    /// Returns a reference to the frame input. Frame input is needed for call/create/eofcreate [`crate::Inspector`] methods
    fn frame_input(&self) -> &FrameInput;
}

/// Impl InspectorFrame for EthFrame.
impl<EVM, ERROR> InspectorFrame for EthFrame<EVM, ERROR, EthInterpreter>
where
    EVM: EvmTr<
            Context: ContextTr,
            Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
            Instructions: InstructionProvider<
                Context = EVM::Context,
                InterpreterTypes = EthInterpreter,
            >,
        > + InspectorEvmTr,
    ERROR: From<ContextTrDbError<EVM::Context>> + FromStringError,
{
    type IT = EthInterpreter;

    fn run_inspect(&mut self, evm: &mut Self::Evm) -> Result<FrameInitOrResult<Self>, Self::Error> {
        let interpreter = self.interpreter();
        let next_action = evm.run_inspect_interpreter(interpreter);
        self.process_next_action(evm, next_action)
    }

    fn interpreter(&mut self) -> &mut Interpreter<Self::IT> {
        &mut self.interpreter
    }

    fn frame_input(&self) -> &FrameInput {
        &self.input
    }
}
