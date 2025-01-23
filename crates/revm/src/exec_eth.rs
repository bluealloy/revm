use crate::{ExecuteCommitEvm, ExecuteEvm};
use context::{Cfg, Context};
use context_interface::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    Block, Database, DatabaseGetter, Journal, Transaction,
};
use database_interface::DatabaseCommit;
use handler::{
    instructions::EthInstructionExecutor, EthContext, EthFrame, EthHandler, EthPrecompileProvider,
    MainnetHandler,
};
use interpreter::interpreter::EthInterpreter;
use primitives::Log;
use state::EvmState;
use std::vec::Vec;

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

    fn exec_previous(&mut self) -> Self::Output {
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

    fn exec_commit_previous(&mut self) -> Self::CommitOutput {
        transact_main(self).map(|r| {
            self.db().commit(r.state);
            r.result
        })
    }
}

/// Helper function that executed a transaction and commits the state.
pub fn transact_main<CTX: EthContext>(
    ctx: &mut CTX,
) -> Result<
    ResultAndState<HaltReason>,
    EVMError<<<CTX as DatabaseGetter>::Database as Database>::Error, InvalidTransaction>,
> {
    MainnetHandler::<
        CTX,
        _,
        EthFrame<CTX, _, _, _>,
        EthPrecompileProvider<CTX, _>,
        EthInstructionExecutor<EthInterpreter, CTX>,
    >::default()
    .run(ctx)
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

        let ok = ctx.exec_previous().unwrap();

        let auth_acc = ok.state.get(&auth).unwrap();
        assert_eq!(auth_acc.info.code, Some(Bytecode::new_eip7702(FFADDRESS)));
        assert_eq!(auth_acc.info.nonce, 1);
        assert_eq!(
            auth_acc.storage.get(&U256::from(1)).unwrap().present_value,
            U256::from(1)
        );
    }
}
