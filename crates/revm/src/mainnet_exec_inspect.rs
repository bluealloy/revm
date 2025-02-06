use crate::{exec_inspect::ExecuteCommitEvm, ExecuteEvm};
use crate::{InspectCommitEvm, InspectEvm};
use context::{Cfg, Context, Evm};
use context_interface::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    Block, ContextTrait, Database, Journal, Transaction,
};
use database_interface::DatabaseCommit;
use handler::{handler::EvmTrait, inspector::EthInspectorHandler};
use handler::{
    inspector::Inspector, instructions::EthInstructions, EthFrame, EthHandler, EthPrecompiles,
    MainnetHandler,
};
use interpreter::interpreter::EthInterpreter;

use primitives::Log;
use state::EvmState;
use std::vec::Vec;

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, INSP> ExecuteEvm
    for Evm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
        EthPrecompiles<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
    >
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>, EthInterpreter>,
{
    type Output =
        Result<ResultAndState<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>>;

    fn transact_previous(&mut self) -> Self::Output {
        let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>>::default();
        t.run(self)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, INSP> ExecuteCommitEvm
    for Evm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
        EthPrecompiles<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
    >
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database + DatabaseCommit,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>, EthInterpreter>,
{
    type CommitOutput =
        Result<ExecutionResult<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>>;

    fn transact_commit_previous(&mut self) -> Self::CommitOutput {
        self.transact_previous().map(|r| {
            self.db().commit(r.state);
            r.result
        })
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, INSP> InspectEvm
    for Evm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
        EthPrecompiles<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
    >
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>, EthInterpreter>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.data.inspector = inspector;
    }

    fn inspect_previous(&mut self) -> Self::Output {
        let mut t = MainnetHandler::<_, EVMError<DB::Error, InvalidTransaction>, EthFrame<_, _, _>> {
            _phantom: core::marker::PhantomData,
        };

        t.inspect_run(self)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, INSP> InspectCommitEvm
    for Evm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
        EthPrecompiles<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
    >
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database + DatabaseCommit,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>, EthInterpreter>,
{
    fn inspect_commit_previous(&mut self) -> Self::CommitOutput {
        self.inspect_previous().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{MainBuilder, MainContext};
    use bytecode::{
        opcode::{PUSH1, SSTORE},
        Bytecode,
    };
    use context_interface::TransactionType;
    use database::{BenchmarkDB, EEADDRESS, FFADDRESS};
    use primitives::{address, TxKind, U256};
    use specification::hardfork::SpecId;

    #[test]
    fn sanity_eip7702_tx() {
        let auth = address!("0000000000000000000000000000000000000100");

        let bytecode = Bytecode::new_legacy([PUSH1, 0x01, PUSH1, 0x01, SSTORE].into());

        let ctx = Context::mainnet()
            .modify_cfg_chained(|cfg| cfg.spec = SpecId::PRAGUE)
            .with_db(BenchmarkDB::new_bytecode(bytecode))
            .modify_tx_chained(|tx| {
                tx.tx_type = TransactionType::Eip7702.into();
                tx.gas_limit = 100_000;
                tx.authorization_list = vec![(Some(auth), U256::from(0), 0, FFADDRESS)];
                tx.caller = EEADDRESS;
                tx.kind = TxKind::Call(auth);
            });

        let mut evm = ctx.build_mainnet();

        let ok = evm.transact_previous().unwrap();

        let auth_acc = ok.state.get(&auth).unwrap();
        assert_eq!(auth_acc.info.code, Some(Bytecode::new_eip7702(FFADDRESS)));
        assert_eq!(auth_acc.info.nonce, 1);
        assert_eq!(
            auth_acc.storage.get(&U256::from(1)).unwrap().present_value,
            U256::from(1)
        );
    }
}
