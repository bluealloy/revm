use super::{EvmTrError, Handler};
use crate::{EvmTr, Frame, FrameResult};
use context::JournalOutput;
use context_interface::{result::HaltReason, ContextTr, JournalTr};
use interpreter::FrameInput;

pub struct MainnetHandler<CTX, ERROR, FRAME> {
    pub _phantom: core::marker::PhantomData<(CTX, ERROR, FRAME)>,
}

impl<EVM, ERROR, FRAME> Handler for MainnetHandler<EVM, ERROR, FRAME>
where
    EVM: EvmTr<Context: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>>>,
    ERROR: EvmTrError<EVM>,
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    FRAME: Frame<Evm = EVM, Error = ERROR, FrameResult = FrameResult, FrameInit = FrameInput>,
{
    type Evm = EVM;
    type Error = ERROR;
    type Frame = FRAME;
    type HaltReason = HaltReason;
}

impl<CTX, ERROR, FRAME> Default for MainnetHandler<CTX, ERROR, FRAME> {
    fn default() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}
