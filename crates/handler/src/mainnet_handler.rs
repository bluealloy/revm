use super::{EvmTrError, Handler};
use crate::{EvmTr, Frame};
use context::{FrameResult, LocalContextTr};
use context_interface::{result::HaltReason, ContextTr, JournalTr};
use interpreter::interpreter_action::FrameInit;
use state::EvmState;

/// Mainnet handler that implements the default [`Handler`] trait for the Evm.
#[derive(Debug, Clone)]
pub struct MainnetHandler<CTX, ERROR, FRAME> {
    pub _phantom: core::marker::PhantomData<(CTX, ERROR, FRAME)>,
}

impl<EVM, ERROR, FRAME> Handler for MainnetHandler<EVM, ERROR, FRAME>
where
    EVM: EvmTr<
        Context: ContextTr<
            Journal: JournalTr<State = EvmState>,
            Local: LocalContextTr<Frame = FRAME>,
        >,
    >,
    ERROR: EvmTrError<EVM>,
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    FRAME: Frame<Evm = EVM, Error = ERROR, FrameResult = FrameResult, FrameInit = FrameInit>,
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
