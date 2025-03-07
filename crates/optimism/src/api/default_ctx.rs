use crate::{L1BlockInfo, OpSpecId, OpTransaction};
use revm::{
    context::{BlockEnv, CfgEnv, TxEnv},
    database_interface::EmptyDB,
    Context, JournaledState, MainContext,
};

pub trait DefaultOp {
    fn op() -> Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpecId>,
        EmptyDB,
        JournaledState<EmptyDB>,
        L1BlockInfo,
    >;
}

impl DefaultOp
    for Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpecId>,
        EmptyDB,
        JournaledState<EmptyDB>,
        L1BlockInfo,
    >
{
    fn op() -> Self {
        Context::mainnet()
            .with_tx(OpTransaction::default())
            .with_cfg(CfgEnv::new_with_spec(OpSpecId::BEDROCK))
            .with_chain(L1BlockInfo::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::builder::OpBuilder;
    use inspector::{InspectEvm, NoOpInspector};
    use revm::ExecuteEvm;

    #[test]
    fn default_run_op() {
        let ctx = Context::op();
        // convert to optimism context
        let mut evm = ctx.build_op_with_inspector(NoOpInspector {});
        // execute
        let _ = evm.replay();
        // inspect
        let _ = evm.inspect_previous();
    }
}
