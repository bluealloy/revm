use revm::{
    context::result::{EVMError, HaltReason, InvalidTransaction},
    context_interface::{ContextTr, JournalTr},
    handler::{
        evm::FrameTr, instructions::InstructionProvider, EvmTr, FrameResult, Handler,
        PrecompileProvider,
    },
    inspector::{Inspector, InspectorEvmTr, InspectorHandler},
    interpreter::{interpreter::EthInterpreter, interpreter_action::FrameInit, InterpreterResult},
    state::EvmState,
    Database,
};

/// Custom handler for MyEvm that defines transaction execution behavior.
///
/// This handler demonstrates how to customize EVM execution by implementing
/// the Handler trait. It can be extended to add custom validation, modify
/// gas calculations, or implement protocol-specific behavior while maintaining
/// compatibility with the standard EVM execution flow.
#[derive(Debug)]
pub struct MyHandler<EVM> {
    /// Phantom data to maintain the EVM type parameter.
    /// This field exists solely to satisfy Rust's type system requirements
    /// for generic parameters that aren't directly used in the struct fields.
    pub _phantom: core::marker::PhantomData<EVM>,
}

impl<EVM> Default for MyHandler<EVM> {
    fn default() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<EVM> Handler for MyHandler<EVM>
where
    EVM: EvmTr<
        Context: ContextTr<Journal: JournalTr<State = EvmState>>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
        Frame: FrameTr<FrameResult = FrameResult, FrameInit = FrameInit>,
    >,
{
    type Evm = EVM;
    type Error = EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>;
    type HaltReason = HaltReason;

    fn reward_beneficiary(
        &self,
        _evm: &mut Self::Evm,
        _exec_result: &mut FrameResult,
    ) -> Result<(), Self::Error> {
        // Skip beneficiary reward
        Ok(())
    }
}

impl<EVM> InspectorHandler for MyHandler<EVM>
where
    EVM: InspectorEvmTr<
        Inspector: Inspector<<<Self as Handler>::Evm as EvmTr>::Context, EthInterpreter>,
        Context: ContextTr<Journal: JournalTr<State = EvmState>>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
{
    type IT = EthInterpreter;
}
