use crate::{transaction::OpTxTrait, L1BlockInfo, OpSpec, OpSpecId, OpTransaction};
use revm::{
    context::{BlockEnv, CfgEnv, TxEnv},
    context_interface::{Block, Cfg, Journal, Transaction},
    database_interface::EmptyDB,
    Context, Database, JournaledState, MainContext,
};

pub trait DefaultOp {
    fn op() -> Context<
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
    fn op() -> Self {
        Context::mainnet()
            .with_tx(OpTransaction::default())
            .with_cfg(CfgEnv::new().with_spec(OpSpec::Op(OpSpecId::BEDROCK)))
            .with_chain(L1BlockInfo::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::builder::OpBuilder;
    use revm::ExecuteEvm;

    #[test]
    fn default_than_into() {
        let ctx = Context::op();
        // convert to optimism context
        let mut op_ctx = ctx.build_op();
        //let _ = op_ctx.();
    }
}
