use revm::{
    context::{setters::ContextSetters, Evm},
    context_interface::ContextT,
    handler::{
        instructions::InstructionProvider, ContextTDbError, EthFrame, EvmT, Frame,
        FrameInitOrResult, PrecompileProvider,
    },
    interpreter::{
        interpreter::EthInterpreter, FrameInput, Interpreter, InterpreterAction, InterpreterResult,
        InterpreterTypes,
    },
    precompile::PrecompileErrors,
};

use crate::{inspect_instructions, Inspector, JournalExt};

/// Inspector EVM trait.
pub trait InspectorEvmT: EvmT {
    type Inspector;

    fn inspector(&mut self) -> &mut Self::Inspector;

    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector);

    fn run_inspect_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <Self::Instructions as InstructionProvider>::Output;
}

impl<CTX, INSP, I, P> InspectorEvmT for Evm<CTX, INSP, I, P>
where
    CTX: ContextT<Journal: JournalExt> + ContextSetters,
    I: InstructionProvider<Context = CTX, Output = InterpreterAction>,
    INSP: Inspector<CTX, I::InterpreterTypes>,
{
    type Inspector = INSP;

    fn inspector(&mut self) -> &mut Self::Inspector {
        &mut self.data.inspector
    }

    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        (&mut self.data.ctx, &mut self.data.inspector)
    }

    fn run_inspect_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <Self::Instructions as InstructionProvider>::Output {
        let context = &mut self.data.ctx;
        let instructions = &mut self.instruction;
        let inspector = &mut self.data.inspector;

        inspect_instructions(
            context,
            interpreter,
            inspector,
            instructions.instruction_table(),
        )
    }
}

// TODO move InspectorFrame here
pub trait InspectorFrameT: Frame {
    type IT: InterpreterTypes;

    fn run_inspect(&mut self, evm: &mut Self::Evm) -> Result<FrameInitOrResult<Self>, Self::Error>;

    fn interpreter(&mut self) -> &mut Interpreter<Self::IT>;

    fn frame_input(&self) -> &FrameInput;
}

impl<EVM, ERROR> InspectorFrameT for EthFrame<EVM, ERROR, EthInterpreter>
where
    EVM: EvmT<
            Context: ContextT,
            Precompiles: PrecompileProvider<Context = EVM::Context, Output = InterpreterResult>,
            Instructions: InstructionProvider<
                Context = EVM::Context,
                InterpreterTypes = EthInterpreter,
                Output = InterpreterAction,
            >,
        > + InspectorEvmT,
    ERROR: From<ContextTDbError<EVM::Context>> + From<PrecompileErrors>,
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
