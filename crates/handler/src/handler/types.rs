use context::Context;
use context_interface::{
    result::{InvalidHeader, InvalidTransaction},
    Block, BlockGetter, Cfg, CfgGetter, Database, DatabaseGetter, ErrorGetter, Journal,
    JournalDBError, JournalGetter, PerformantContextAccess, Transaction, TransactionGetter,
};
use handler_interface::{Frame, PrecompileProvider};
use interpreter::{
    interpreter::{EthInstructionProvider, EthInterpreter, InstructionProvider},
    FrameInput, Host,
};
use precompile::PrecompileErrors;
use primitives::Log;
use specification::hardfork::SpecId;
use state::EvmState;

use crate::{EthPrecompileProvider, FrameContext, FrameResult};

use super::EthHandler;

pub struct EthHandlerImpl<CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS> {
    pub precompiles: PRECOMPILES,
    pub instructions: INSTRUCTIONS,
    pub _phantom: core::marker::PhantomData<(CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS)>,
}

impl<CTX: Host, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS>
    EthHandlerImpl<CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS>
where
    PRECOMPILES: PrecompileProvider<Context = CTX, Error = ERROR>,
    INSTRUCTIONS: InstructionProvider<WIRE = EthInterpreter, Host = CTX>,
{
    pub fn crete_frame_context(&self) -> FrameContext<PRECOMPILES, INSTRUCTIONS> {
        FrameContext {
            precompiles: self.precompiles.clone(),
            instructions: self.instructions.clone(),
        }
    }
}

impl<CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS> EthHandler
    for EthHandlerImpl<CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS>
where
    CTX: EthContext,
    ERROR: EthError<CTX, FRAME>,
    PRECOMPILES: PrecompileProvider<Context = CTX, Error = ERROR>,
    INSTRUCTIONS: InstructionProvider<WIRE = EthInterpreter, Host = CTX>,
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    FRAME: Frame<
        Context = CTX,
        Error = ERROR,
        FrameResult = FrameResult,
        FrameInit = FrameInput,
        FrameContext = FrameContext<PRECOMPILES, INSTRUCTIONS>,
    >,
{
    type Context = CTX;
    type Error = ERROR;
    type Frame = FRAME;
    type Precompiles = PRECOMPILES;
    type Instructions = INSTRUCTIONS;

    fn frame_context(
        &mut self,
        context: &mut Self::Context,
    ) -> <Self::Frame as Frame>::FrameContext {
        self.precompiles.set_spec(context.cfg().spec().into());
        self.crete_frame_context()
    }
}

impl<CTX: Host, ERROR, FRAME> Default
    for EthHandlerImpl<
        CTX,
        ERROR,
        FRAME,
        EthPrecompileProvider<CTX, ERROR>,
        EthInstructionProvider<EthInterpreter, CTX>,
    >
{
    fn default() -> Self {
        Self {
            precompiles: EthPrecompileProvider::new(SpecId::LATEST),
            instructions: EthInstructionProvider::new(),
            _phantom: core::marker::PhantomData,
        }
    }
}

pub trait EthContext:
    TransactionGetter
    + BlockGetter
    + DatabaseGetter
    + CfgGetter
    + PerformantContextAccess<Error = JournalDBError<Self>>
    + ErrorGetter<Error = JournalDBError<Self>>
    + JournalGetter<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>
    + Host
{
}

pub trait EthError<CTX: JournalGetter, FRAME: Frame>:
    From<InvalidTransaction>
    + From<InvalidHeader>
    + From<JournalDBError<CTX>>
    + From<<FRAME as Frame>::Error>
    + From<PrecompileErrors>
{
}

impl<
        CTX: JournalGetter,
        FRAME: Frame,
        T: From<InvalidTransaction>
            + From<InvalidHeader>
            + From<JournalDBError<CTX>>
            + From<<FRAME as Frame>::Error>
            + From<PrecompileErrors>,
    > EthError<CTX, FRAME> for T
{
}

impl<
        BLOCK: Block,
        TX: Transaction,
        CFG: Cfg,
        DB: Database,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
        CHAIN,
    > EthContext for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
}

impl<
        BLOCK: Block,
        TX: Transaction,
        CFG: Cfg,
        DB: Database,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
        CHAIN,
    > EthContext for &mut Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
}
