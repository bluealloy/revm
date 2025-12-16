use crate::{
    evm::MonadEvm,
    instructions::MonadInstructions,
    precompiles::MonadPrecompiles,
    MonadSpecId,
};
use revm::{
    context::Cfg,
    context_interface::{Block, JournalTr, Transaction},
    state::EvmState,
    Context, Database,
};

/// Type alias for default MonadEvm.
pub type DefaultMonadEvm<CTX, INSP = ()> =
    MonadEvm<CTX, INSP, MonadInstructions<CTX>, MonadPrecompiles>;

pub trait MonadBuilder: Sized {
    type Context;

    /// Build MonadEvm without inspector.
    fn build_monad(self) -> DefaultMonadEvm<Self::Context>;

    /// Build MonadEvm with inspector.
    fn build_monad_with_inspector<INSP>(self, inspector: INSP) -> DefaultMonadEvm<Self::Context, INSP>;
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> MonadBuilder for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg<Spec = MonadSpecId>,
    DB: Database,
    JOURNAL: JournalTr<Database = DB, State = EvmState>,
{
    type Context = Self;

    fn build_monad(self) -> DefaultMonadEvm<Self::Context> {
        MonadEvm::new(self, ())
    }

    fn build_monad_with_inspector<INSP>(self, inspector: INSP) -> DefaultMonadEvm<Self::Context, INSP> {
        MonadEvm::new(self, inspector)
    }
}