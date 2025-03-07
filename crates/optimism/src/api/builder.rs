use crate::{evm::OpEvm, transaction::OpTxTr, L1BlockInfo, OpSpecId, OpTransaction};
use revm::{
    context::{BlockEnv, Cfg, CfgEnv, JournalOutput, TxEnv},
    context_interface::{Block, JournalTr},
    handler::instructions::EthInstructions,
    interpreter::interpreter::EthInterpreter,
    Context, Database, Journal,
};

pub trait OpBuilder: Sized {
    type Context;

    fn build_op(self) -> OpEvm<Self::Context, (), EthInstructions<EthInterpreter, Self::Context>>;

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

pub type OpContext<DB> =
    Context<BlockEnv, OpTransaction<TxEnv>, CfgEnv<OpSpecId>, DB, Journal<DB>, L1BlockInfo>;
