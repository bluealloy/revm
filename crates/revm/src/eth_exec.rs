use crate::{exec::ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext};
use context::{BlockEnv, Cfg, CfgEnv, Context, ContextTrait, Evm, EvmData, JournaledState, TxEnv};
use context_interface::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    Block, Database, Journal, Transaction,
};
use database_interface::{DatabaseCommit, EmptyDB};
use handler::{
    inspector::Inspector, instructions::EthInstructions, noop::NoOpInspector, EthFrame, EthHandler,
    EthPrecompiles, MainnetHandler,
};
use interpreter::{interpreter::EthInterpreter, Host};
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
    type Context = Self;

    fn build_mainnet(
        self,
    ) -> Evm<
        Self::Context,
        NoOpInspector,
        EthInstructions<EthInterpreter, Self::Context>,
        EthPrecompiles<Self::Context>,
    > {
        Evm {
            data: EvmData {
                ctx: self,
                inspector: NoOpInspector {},
            },
            enabled_inspection: false,
            instruction: EthInstructions::default(),
            precompiles: EthPrecompiles::default(),
        }
    }

    fn build_mainnet_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> Evm<
        Self::Context,
        INSP,
        EthInstructions<EthInterpreter, Self::Context>,
        EthPrecompiles<Self::Context>,
    > {
        Evm {
            data: EvmData {
                ctx: self,
                inspector,
            },
            enabled_inspection: true,
            instruction: EthInstructions::default(),
            precompiles: EthPrecompiles::default(),
        }
    }
}

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

    fn exec_previous(&mut self) -> Self::Output {
        transact_main(self)
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

    fn exec_commit_previous(&mut self) -> Self::CommitOutput {
        transact_main(self).map(|r| {
            self.db().commit(r.state);
            r.result
        })
    }
}

pub fn transact_main<CTX: ContextTrait + Host, INSP>(
    evm: &mut Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, EthPrecompiles<CTX>>,
) -> Result<ResultAndState<HaltReason>, EVMError<<CTX::Db as Database>::Error, InvalidTransaction>>
where
    CTX: ContextTrait<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>,
    INSP: Inspector<CTX, EthInterpreter>,
{
    let mut t = MainnetHandler::<
        _,
        EVMError<<CTX::Db as Database>::Error, InvalidTransaction>,
        EthFrame<_, _, _>,
    > {
        _phantom: core::marker::PhantomData,
    };

    t.run(evm)
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
