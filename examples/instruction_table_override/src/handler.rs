use crate::instructions::CustomInstructionExecutor;
use revm::{
    context_interface::{result::HaltReason, CfgGetter},
    handler::{EthContext, EthError, EthFrame, EthHandler, EthPrecompileProvider, FrameContext},
    interpreter::{interpreter::EthInterpreter, Host},
    precompile::PrecompileErrors,
};

// Our custom handler
pub struct CustomOpcodeHandler<CTX, ERROR> {
    _phantom: std::marker::PhantomData<(CTX, ERROR)>,
}

impl<CTX, ERROR> CustomOpcodeHandler<CTX, ERROR> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<CTX: CfgGetter + Host, ERROR: From<PrecompileErrors>> Default
    for CustomOpcodeHandler<CTX, ERROR>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<CTX, ERROR> EthHandler for CustomOpcodeHandler<CTX, ERROR>
where
    CTX: EthContext,
    ERROR: EthError<CTX>,
{
    type Context = CTX;
    type Error = ERROR;
    type Precompiles = EthPrecompileProvider<CTX, Self::Error>;
    type Instructions = CustomInstructionExecutor<EthInterpreter, Self::Context>;
    type Frame =
        EthFrame<CTX, ERROR, EthInterpreter, FrameContext<Self::Precompiles, Self::Instructions>>;
    type HaltReason = HaltReason;
}
