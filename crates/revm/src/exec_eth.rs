use crate::{
    exec::{MainBuilder, MainContext},
    ExecuteCommitEvm, ExecuteEvm,
};
use context::{BlockEnv, Cfg, CfgEnv, Context, JournaledState, TxEnv, MEVM};
use context_interface::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    Block, Database, DatabaseGetter, Journal, Transaction,
};
use database_interface::{DatabaseCommit, EmptyDB};
use handler::{
    instructions::EthInstructionExecutor, EthContext, EthFrame, EthHandler, EthPrecompileProvider,
    FrameContext, MainnetHandler,
};
use interpreter::interpreter::EthInterpreter;
use primitives::Log;
use specification::hardfork::SpecId;
use state::EvmState;
use std::vec::Vec;

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> MainBuilder for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
{
    type FrameContext = FrameContext<
        EthPrecompileProvider<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
        EthInstructionExecutor<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
    >;

    fn build_mainnet(self) -> MEVM<Self, Self::FrameContext> {
        MEVM {
            ctx: self,
            frame_ctx: FrameContext::new(
                EthPrecompileProvider::default(),
                EthInstructionExecutor::default(),
            ),
        }
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> ExecuteEvm
    for MEVM<
        Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
        FrameContext<
            EthPrecompileProvider<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
            EthInstructionExecutor<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
        >,
    >
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
    for MEVM<
        Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
        FrameContext<
            EthPrecompileProvider<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
            EthInstructionExecutor<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
        >,
    >
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
            self.ctx.db().commit(r.state);
            r.result
        })
    }
}

pub fn transact_main<CTX: EthContext>(
    evm: &mut MEVM<
        CTX,
        FrameContext<EthPrecompileProvider<CTX>, EthInstructionExecutor<EthInterpreter, CTX>>,
    >,
) -> Result<
    ResultAndState<HaltReason>,
    EVMError<<<CTX as DatabaseGetter>::Database as Database>::Error, InvalidTransaction>,
> {
    MainnetHandler::<CTX, _, EthFrame<CTX, _, _, _>, _>::default().run(evm)
}

impl MainContext for Context<BlockEnv, TxEnv, CfgEnv, EmptyDB, JournaledState<EmptyDB>, ()> {
    fn mainnet() -> Self {
        Context::new(EmptyDB::new(), SpecId::LATEST)
    }
}

#[cfg(test)]
mod test {
    use super::*;
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

        let ok = evm.exec_previous().unwrap();

        let auth_acc = ok.state.get(&auth).unwrap();
        assert_eq!(auth_acc.info.code, Some(Bytecode::new_eip7702(FFADDRESS)));
        assert_eq!(auth_acc.info.nonce, 1);
        assert_eq!(
            auth_acc.storage.get(&U256::from(1)).unwrap().present_value,
            U256::from(1)
        );
    }
}
