use crate::{evm::OpEvm, transaction::OpTxTr, L1BlockInfo, OpSpecId};
use revm::{
    context::{Cfg, JournalOutput},
    context_interface::{Block, JournalTr},
    handler::instructions::EthInstructions,
    interpreter::interpreter::EthInterpreter,
    Context, Database,
};

/// Trait that allows for optimism OpEvm to be built.
pub trait OpBuilder: Sized {
    /// Type of the context.
    type Context;

    /// Build the op.
    fn build_op(self) -> OpEvm<Self::Context, (), EthInstructions<EthInterpreter, Self::Context>>;

    /// Build the op with an inspector.
    fn build_op_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> OpEvm<Self::Context, INSP, EthInstructions<EthInterpreter, Self::Context>>;
}

impl<BLOCK, TX, CFG, DB, JOURNAL> OpBuilder for Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>
where
    BLOCK: Block,
    TX: OpTxTr,
    CFG: Cfg<Spec = OpSpecId>,
    DB: Database,
    JOURNAL: JournalTr<Database = DB, FinalOutput = JournalOutput>,
{
    type Context = Self;

    fn build_op(self) -> OpEvm<Self::Context, (), EthInstructions<EthInterpreter, Self::Context>> {
        OpEvm::new(self, ())
    }

    fn build_op_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> OpEvm<Self::Context, INSP, EthInstructions<EthInterpreter, Self::Context>> {
        OpEvm::new(self, inspector)
    }
}
