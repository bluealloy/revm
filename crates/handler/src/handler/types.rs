use super::{EthHandler, EthTraitError, EvmTypesTrait};
use crate::{
    inspector::{EthInspectorHandler, Inspector, InspectorFrame},
    FrameResult,
};
use context_interface::{result::HaltReason, ContextGetters, Journal};
use handler_interface::Frame;
use interpreter::{interpreter::EthInterpreter, FrameInput};

use primitives::Log;
use state::EvmState;
use std::vec::Vec;

pub struct MainnetHandler<CTX, ERROR, FRAME> {
    pub _phantom: core::marker::PhantomData<(CTX, ERROR, FRAME)>,
}

impl<EVM, ERROR, FRAME> EthHandler for MainnetHandler<EVM, ERROR, FRAME>
where
    EVM: EvmTypesTrait<
        Context: ContextGetters<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>,
    >,
    ERROR: EthTraitError<EVM>,
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    FRAME: Frame<Context = EVM, Error = ERROR, FrameResult = FrameResult, FrameInit = FrameInput>,
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

impl<CTX, ERROR, FRAME> EthInspectorHandler for MainnetHandler<CTX, ERROR, FRAME>
where
    CTX: EvmTypesTrait<
        Context: ContextGetters<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>,
        Inspector: Inspector<<<Self as EthHandler>::Evm as EvmTypesTrait>::Context, EthInterpreter>,
    >,
    ERROR: EthTraitError<CTX>,
    FRAME: Frame<Context = CTX, Error = ERROR, FrameResult = FrameResult, FrameInit = FrameInput>
        + InspectorFrame<IT = EthInterpreter, FrameInput = FrameInput>,
{
    type IT = EthInterpreter;
}

// impl<
//         BLOCK: Block,
//         TX: Transaction,
//         CFG: Cfg,
//         DB: Database,
//         JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
//         CHAIN,
//     > EthContext for &mut Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
// {
// }
