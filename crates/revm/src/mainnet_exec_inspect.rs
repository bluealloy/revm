use crate::{exec_inspect::ExecuteCommitEvm, ExecuteEvm};
use crate::{InspectCommitEvm, InspectEvm};
use context::setters::ContextSetters;
use context::Evm;
use context_interface::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    ContextTrait, Database, Journal,
};
use database_interface::DatabaseCommit;
use handler::inspector::JournalExt;
use handler::{handler::EvmTrait, inspector::EthInspectorHandler};
use handler::{
    inspector::Inspector, instructions::EthInstructions, EthFrame, EthHandler, EthPrecompiles,
    MainnetHandler,
};
use interpreter::interpreter::EthInterpreter;

use primitives::Log;
use state::EvmState;
use std::vec::Vec;

impl<CTX, INSP> ExecuteEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, EthPrecompiles<CTX>>
where
    CTX: ContextSetters
        + ContextTrait<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)> + JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
{
    type Output = Result<
        ResultAndState<HaltReason>,
        EVMError<<CTX::Db as Database>::Error, InvalidTransaction>,
    >;

    fn transact_previous(&mut self) -> Self::Output {
        let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>>::default();
        t.run(self)
    }
}

impl<CTX, INSP> ExecuteCommitEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, EthPrecompiles<CTX>>
where
    CTX: ContextSetters
        + ContextTrait<
            Journal: Journal<FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
            Db: DatabaseCommit,
        >,
    INSP: Inspector<CTX, EthInterpreter>,
{
    type CommitOutput = Result<
        ExecutionResult<HaltReason>,
        EVMError<<CTX::Db as Database>::Error, InvalidTransaction>,
    >;

    fn transact_commit_previous(&mut self) -> Self::CommitOutput {
        self.transact_previous().map(|r| {
            self.db().commit(r.state);
            r.result
        })
    }
}

impl<CTX, INSP> InspectEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, EthPrecompiles<CTX>>
where
    CTX: ContextSetters
        + ContextTrait<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)> + JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.data.inspector = inspector;
    }

    fn inspect_previous(&mut self) -> Self::Output {
        let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>> {
            _phantom: core::marker::PhantomData,
        };

        t.inspect_run(self)
    }
}

impl<CTX, INSP> InspectCommitEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, EthPrecompiles<CTX>>
where
    CTX: ContextSetters
        + ContextTrait<
            Journal: Journal<FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
            Db: DatabaseCommit,
        >,
    INSP: Inspector<CTX, EthInterpreter>,
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
    use context::Context;
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
