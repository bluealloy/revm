use crate::{evm::OpEvm, transaction::OpTxTrait, L1BlockInfo, OpSpec, OpTransaction};
use precompile::Log;
use revm::{
    context::{BlockEnv, Cfg, CfgEnv, TxEnv},
    context_interface::{Block, Journal},
    handler::{instructions::EthInstructions, noop::NoOpInspector},
    interpreter::interpreter::EthInterpreter,
    state::EvmState,
    Context, Database, JournaledState,
};

pub trait OpBuilder: Sized {
    type Context;

    fn build_op(
        self,
    ) -> OpEvm<Self::Context, NoOpInspector, EthInstructions<EthInterpreter, Self::Context>>;

    fn build_op_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> OpEvm<Self::Context, INSP, EthInstructions<EthInterpreter, Self::Context>>;
}

impl<BLOCK, TX, CFG, DB, JOURNAL> OpBuilder for Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>
where
    BLOCK: Block,
    TX: OpTxTrait,
    CFG: Cfg<Spec = OpSpec>,
    DB: Database,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
{
    type Context = Self;

    fn build_op(
        self,
    ) -> OpEvm<Self::Context, NoOpInspector, EthInstructions<EthInterpreter, Self::Context>> {
        OpEvm::new(self, NoOpInspector {})
    }

    fn build_op_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> OpEvm<Self::Context, INSP, EthInstructions<EthInterpreter, Self::Context>> {
        OpEvm::new(self, inspector)
    }
}

pub type OpContext<DB> =
    Context<BlockEnv, OpTransaction<TxEnv>, CfgEnv<OpSpec>, DB, JournaledState<DB>, L1BlockInfo>;
