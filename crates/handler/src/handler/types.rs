use super::EthHandler;
use crate::{instructions::InstructionExecutor, EthPrecompileProvider, FrameContext, FrameResult};
use context::Context;
use context_interface::{
    result::{HaltReason, InvalidHeader, InvalidTransaction},
    Block, BlockGetter, Cfg, CfgGetter, Database, DatabaseGetter, ErrorGetter, Journal,
    JournalDBError, JournalGetter, PerformantContextAccess, Transaction, TransactionGetter,
};
use handler_interface::{Frame, PrecompileProvider};
use interpreter::{interpreter::EthInterpreter, FrameInput, Host};
use precompile::PrecompileErrors;
use primitives::Log;
use state::EvmState;
use std::vec::Vec;

pub struct MainnetHandler<CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS> {
    pub _phantom: core::marker::PhantomData<(CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS)>,
}

impl<CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS> EthHandler
    for MainnetHandler<CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS>
where
    CTX: EthContext,
    ERROR: EthError<CTX>,
    PRECOMPILES: PrecompileProvider<
        Context = CTX,
        Error = ERROR,
        Spec = <<CTX as CfgGetter>::Cfg as Cfg>::Spec,
    >,
    INSTRUCTIONS: InstructionExecutor<InterpreterTypes = EthInterpreter, CTX = CTX>,
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
    type HaltReason = HaltReason;
}

impl<CTX: Host + CfgGetter, ERROR, FRAME, INSTRUCTIONS: Default> Default
    for MainnetHandler<CTX, ERROR, FRAME, EthPrecompileProvider<CTX, ERROR>, INSTRUCTIONS>
{
    fn default() -> Self {
        Self {
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

pub trait EthError<CTX: JournalGetter>:
    From<InvalidTransaction> + From<InvalidHeader> + From<JournalDBError<CTX>> + From<PrecompileErrors>
{
}

impl<
        CTX: JournalGetter,
        T: From<InvalidTransaction>
            + From<InvalidHeader>
            + From<JournalDBError<CTX>>
            + From<PrecompileErrors>,
    > EthError<CTX> for T
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
