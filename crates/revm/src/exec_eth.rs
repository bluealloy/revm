use crate::{exec::MainContext, ExecuteEvm};
use context::{BlockEnv, Cfg, CfgEnv, Context, ContextTrait, Evm, JournaledState, TxEnv};
use context_interface::{
    result::{EVMError, HaltReason, InvalidTransaction, ResultAndState},
    Block, Database, Journal, Transaction,
};
use database_interface::EmptyDB;
use handler::{
    instructions::EthInstructionExecutor, EthFrame, EthHandler, EthPrecompileProvider,
    MainnetHandler,
};
use interpreter::{interpreter::EthInterpreter, Host};
use primitives::Log;
use specification::hardfork::SpecId;
use state::EvmState;
use std::vec::Vec;

// impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> MainBuilder for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
// where
//     BLOCK: Block,
//     TX: Transaction,
//     CFG: Cfg,
//     DB: Database,
//     JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
// {
//     type FrameContext = FrameContext<
//         EthPrecompileProvider<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
//         EthInstructionExecutor<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
//     >;

//     fn build_mainnet(self) -> MEVM<Self, Self::FrameContext, ()> {
//         MEVM {
//             ctx: self,
//             frame_ctx: FrameContext::new(
//                 EthPrecompileProvider::default(),
//                 EthInstructionExecutor::default(),
//             ),
//             inspector: (),
//         }
//     }
// }

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, INSP> ExecuteEvm
    for Evm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
        INSP,
        EthInstructionExecutor<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
        EthPrecompileProvider<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
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

// impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, INSP> ExecuteCommitEvm
//     for MEVM<
//         Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
//         FrameContext<
//             EthPrecompileProvider<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
//             EthInstructionExecutor<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
//         >,
//         INSP,
//     >
// where
//     BLOCK: Block,
//     TX: Transaction,
//     CFG: Cfg,
//     DB: Database + DatabaseCommit,
//     JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
// {
//     type CommitOutput =
//         Result<ExecutionResult<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>>;

//     fn exec_commit_previous(&mut self) -> Self::CommitOutput {
//         transact_main(self).map(|r| {
//             self.ctx.db().commit(r.state);
//             r.result
//         })
//     }
// }

pub fn transact_main<CTX: ContextTrait + Host, INSP>(
    evm: &mut Evm<
        CTX,
        INSP,
        EthInstructionExecutor<EthInterpreter, CTX>,
        EthPrecompileProvider<CTX>,
    >,
) -> Result<ResultAndState<HaltReason>, EVMError<<CTX::Db as Database>::Error, InvalidTransaction>>
where
    CTX: ContextTrait<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>,
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
    use context::Ctx;
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

        let mut evm = Evm {
            ctx: Ctx { ctx, inspector: () },
            instruction: EthInstructionExecutor::default(),
            precompiles: EthPrecompileProvider::default(),
        };

        let ok = transact_main(&mut evm).unwrap();

        // let mut evm = ctx.build_mainnet();

        // let ok = evm.exec_previous().unwrap();

        let auth_acc = ok.state.get(&auth).unwrap();
        assert_eq!(auth_acc.info.code, Some(Bytecode::new_eip7702(FFADDRESS)));
        assert_eq!(auth_acc.info.nonce, 1);
        assert_eq!(
            auth_acc.storage.get(&U256::from(1)).unwrap().present_value,
            U256::from(1)
        );
    }
}
