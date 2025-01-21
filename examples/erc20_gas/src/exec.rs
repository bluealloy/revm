use crate::handler::Erc20MainetHandler;
use revm::{
    context_interface::{
        result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
        DatabaseGetter,
    },
    database_interface::{Database, DatabaseCommit},
    handler::handler::{EthContext, EthHandler},
};

pub fn transact_erc20evm<DB: Database, CTX: EthContext + DatabaseGetter<Database = DB>>(
    ctx: &mut CTX,
) -> Result<ResultAndState<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>> {
    Erc20MainetHandler::<CTX, _>::new().run(ctx)
}

pub fn transact_erc20evm_commit<
    DB: Database + DatabaseCommit,
    CTX: EthContext + DatabaseGetter<Database = DB>,
>(
    ctx: &mut CTX,
) -> Result<ExecutionResult<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>> {
    transact_erc20evm(ctx).map(|r| {
        ctx.db().commit(r.state);
        r.result
    })
}
