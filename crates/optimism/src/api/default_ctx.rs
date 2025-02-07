use crate::{L1BlockInfo, OpSpec, OpSpecId, OpTransaction};
use revm::{
    context::{BlockEnv, CfgEnv, TxEnv},
    database_interface::EmptyDB,
    Context, JournaledState, MainContext,
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
    use revm::{ExecuteEvm, InspectEvm};

    #[test]
    fn default_run_op() {
        let ctx = Context::op();
        // convert to optimism context
        let mut evm = ctx.build_op();
        // execute
        let _ = evm.transact_previous();
        // inspect
        let _ = evm.inspect_previous();
    }
}
