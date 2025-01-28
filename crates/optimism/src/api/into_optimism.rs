use crate::{
    context::OpContext, transaction::OpTxTrait, L1BlockInfo, OpSpec, OpSpecId, OpTransaction,
};
use revm::{
    context::{BlockEnv, CfgEnv, TxEnv},
    context_interface::{Block, Cfg, Journal, Transaction},
    database_interface::EmptyDB,
    Context, Database, JournaledState,
};

pub trait IntoOptimism<
    BLOCK: Block,
    TX: OpTxTrait = OpTransaction<TxEnv>,
    CFG: Cfg<Spec = OpSpec> = CfgEnv<OpSpec>,
    DB: Database = EmptyDB,
    JOURNAL: Journal<Database = DB> = JournaledState<DB>,
>
{
    fn into_optimism(self) -> OpContext<BLOCK, TX, CFG, DB, JOURNAL>;
}

impl<BLOCK: Block, TX: Transaction, DB: Database, JOURNAL: Journal<Database = DB>>
    IntoOptimism<BLOCK, OpTransaction<TX>, CfgEnv<OpSpec>, DB, JOURNAL>
    for Context<BLOCK, OpTransaction<TX>, CfgEnv<OpSpec>, DB, JOURNAL, L1BlockInfo>
{
    fn into_optimism(self) -> OpContext<BLOCK, OpTransaction<TX>, CfgEnv<OpSpec>, DB, JOURNAL> {
        OpContext(self)
    }
}

pub trait DefaultOp {
    fn default_op() -> Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpec>,
        EmptyDB,
        JournaledState<EmptyDB>,
        L1BlockInfo,
    >;
}

impl DefaultOp
    for Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpec>,
        EmptyDB,
        JournaledState<EmptyDB>,
        L1BlockInfo,
    >
{
    fn default_op() -> Self {
        Context::default()
            .with_tx(OpTransaction::default())
            .with_cfg(CfgEnv::new().with_spec(OpSpec::Op(OpSpecId::BEDROCK)))
            .with_chain(L1BlockInfo::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use revm::ExecuteEvm;

    #[test]
    fn default_than_into() {
        let ctx = Context::default_op();
        // convert to optimism context
        let mut op_ctx = ctx.into_optimism();
        let _ = op_ctx.exec_previous();
    }
}
