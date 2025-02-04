use super::{EthHandler, EthTraitError};
use crate::FrameResult;
use auto_impl::auto_impl;
use context::{Context, ContextTrait, EvmTypesTrait};
use context_interface::{
    result::{HaltReason, InvalidHeader, InvalidTransaction},
    Block, BlockGetter, Cfg, CfgGetter, Database, DatabaseGetter, ErrorGetter, Journal,
    JournalDBError, JournalGetter, PerformantContextAccess, Transaction, TransactionGetter,
};
use handler_interface::Frame;
use interpreter::{FrameInput, Host};
use precompile::PrecompileErrors;
use primitives::Log;
use state::EvmState;
use std::vec::Vec;

pub struct MainnetHandler<CTX, ERROR, FRAME> {
    pub _phantom: core::marker::PhantomData<(CTX, ERROR, FRAME)>,
}

impl<CTX, ERROR, FRAME> EthHandler for MainnetHandler<CTX, ERROR, FRAME>
where
    CTX: EvmTypesTrait<Context: ContextTrait<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>>,
    ERROR: EthTraitError<CTX>,
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    FRAME: Frame<Context = CTX, Error = ERROR, FrameResult = FrameResult, FrameInit = FrameInput>,
{
    type Evm = CTX;
    type Error = ERROR;
    type Frame = FRAME;
    type HaltReason = HaltReason;
}

impl<CTX: ContextTrait + Host, ERROR, FRAME> Default for MainnetHandler<CTX, ERROR, FRAME> {
    fn default() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}

#[auto_impl(&mut)]
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
