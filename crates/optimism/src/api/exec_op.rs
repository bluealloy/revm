use revm::{
    context_interface::{
        result::{EVMError, ExecutionResult, ResultAndState},
        Cfg, CfgGetter, DatabaseGetter,
    },
    handler::{instructions::EthInstructionExecutor, EthContext, EthFrame, EthHandler},
    interpreter::interpreter::EthInterpreter,
    Database, DatabaseCommit,
};

use crate::{
    handler::{precompiles::OpPrecompileProvider, OpHandler},
    transaction::abstraction::OpTxGetter,
    L1BlockInfoGetter, OpHaltReason, OpSpec, OpTransactionError,
};

/// Helper function that executed a transaction and commits the state.
pub fn transact_op<CTX: EthContext + OpTxGetter + L1BlockInfoGetter>(
    ctx: &mut CTX,
) -> Result<
    ResultAndState<OpHaltReason>,
    EVMError<<<CTX as DatabaseGetter>::Database as Database>::Error, OpTransactionError>,
>
where
    <CTX as CfgGetter>::Cfg: Cfg<Spec = OpSpec>,
{
    let mut op = OpHandler::<
        CTX,
        _,
        EthFrame<CTX, _, _, _>,
        OpPrecompileProvider<CTX, _>,
        EthInstructionExecutor<EthInterpreter, CTX>,
    >::new();
    op.run(ctx)
}

pub fn transact_op_commit<CTX: EthContext + OpTxGetter + L1BlockInfoGetter>(
    ctx: &mut CTX,
) -> Result<
    ExecutionResult<OpHaltReason>,
    EVMError<<<CTX as DatabaseGetter>::Database as Database>::Error, OpTransactionError>,
>
where
    <CTX as DatabaseGetter>::Database: DatabaseCommit,
    <CTX as CfgGetter>::Cfg: Cfg<Spec = OpSpec>,
{
    transact_op(ctx).map(|r| {
        ctx.db().commit(r.state);
        r.result
    })
}
