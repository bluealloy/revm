use crate::{ExecuteCommitEvm, ExecuteEvm};
use context::{Cfg, Context};
use context_interface::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    Block, Database, DatabaseGetter, Journal, Transaction,
};
use database_interface::DatabaseCommit;
use handler::{EthContext, EthFrame, EthHandler, EthHandlerImpl, EthPrecompileProvider};
use primitives::Log;
use state::EvmState;

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> ExecuteEvm for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
{
    type Output =
        Result<ResultAndState<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>>;

    fn exec_previous_tx(&mut self) -> Self::Output {
        transact_main(self)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> ExecuteCommitEvm
    for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database + DatabaseCommit,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
{
    type CommitOutput =
        Result<ExecutionResult<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>>;

    fn exec_commit_previous_tx(&mut self) -> Self::CommitOutput {
        transact_main_commit(self)
    }
}

/// Helper function that executed a transaction and commits the state.
pub fn transact_main<CTX: EthContext>(
    ctx: &mut CTX,
) -> Result<
    ResultAndState<HaltReason>,
    EVMError<<<CTX as DatabaseGetter>::Database as Database>::Error, InvalidTransaction>,
> {
    EthHandlerImpl::<CTX, _, EthFrame<CTX, _, _, _>, EthPrecompileProvider<CTX, _>, _>::default()
        .run(ctx)
}

pub fn transact_main_commit<CTX: EthContext>(
    ctx: &mut CTX,
) -> Result<
    ExecutionResult<HaltReason>,
    EVMError<<<CTX as DatabaseGetter>::Database as Database>::Error, InvalidTransaction>,
>
where
    <CTX as DatabaseGetter>::Database: DatabaseCommit,
{
    transact_main(ctx).map(|r| {
        ctx.db().commit(r.state);
        r.result
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use bytecode::{
        opcode::{PUSH1, SSTORE},
        Bytecode,
    };
    use context::TxEnv;
    use context_interface::TransactionType;
    use database::{BenchmarkDB, EEADDRESS, FFADDRESS};
    use primitives::{address, TxKind, U256};
    use specification::hardfork::SpecId;

    #[test]
    fn sanity_tx_ref() {
        let delegate = address!("0000000000000000000000000000000000000000");
        let caller = address!("0000000000000000000000000000000000000001");
        let auth = address!("0000000000000000000000000000000000000100");

        let mut tx1 = TxEnv::default();
        tx1.tx_type = TransactionType::Eip7702.into();
        tx1.gas_limit = 100_000;
        tx1.authorization_list = vec![(Some(auth), U256::from(1), 0, delegate)];
        tx1.caller = caller;
        tx1.kind = TxKind::Call(auth);

        let mut tx2 = TxEnv::default();
        tx2.tx_type = TransactionType::Legacy.into();
        // `nonce` was bumped from 0 to 1
        tx2.nonce = 1;

        let mut ctx = Context::default();

        let _ = ctx.exec(tx1).unwrap();
        let _ = ctx.exec(tx2).unwrap();
    }

    #[test]
    fn sanity_eip7702_tx() {
        let auth = address!("0000000000000000000000000000000000000100");

        let bytecode = Bytecode::new_legacy([PUSH1, 0x01, PUSH1, 0x01, SSTORE].into());

        let mut ctx = Context::default()
            .modify_cfg_chained(|cfg| cfg.spec = SpecId::PRAGUE)
            .with_db(BenchmarkDB::new_bytecode(bytecode))
            .modify_tx_chained(|tx| {
                tx.tx_type = TransactionType::Eip7702.into();
                tx.gas_limit = 100_000;
                tx.authorization_list = vec![(Some(auth), U256::from(0), 0, FFADDRESS)];
                tx.caller = EEADDRESS;
                tx.kind = TxKind::Call(auth);
            });

        let ok = ctx.exec_previous_tx().unwrap();

        let auth_acc = ok.state.get(&auth).unwrap();
        assert_eq!(auth_acc.info.code, Some(Bytecode::new_eip7702(FFADDRESS)));
        assert_eq!(auth_acc.info.nonce, 1);
        assert_eq!(
            auth_acc.storage.get(&U256::from(1)).unwrap().present_value,
            U256::from(1)
        );
    }
}
