use crate::{exec_inspect::ExecuteCommitEvm, ExecuteEvm};
use context::setters::ContextSetters;
use context::Evm;
use context_interface::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    ContextTr, Database, Journal,
};
use database_interface::DatabaseCommit;
use handler::{
    instructions::EthInstructions, EthFrame, HandlerTr, MainnetHandler, PrecompileProvider,
};
use interpreter::interpreter::EthInterpreter;

use interpreter::InterpreterResult;
use primitives::Log;
use state::EvmState;
use std::vec::Vec;

impl<CTX, INSP, PRECOMPILES> ExecuteEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILES>
where
    CTX: ContextSetters + ContextTr<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>,
    PRECOMPILES: PrecompileProvider<Context = CTX, Output = InterpreterResult>,
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

impl<CTX, INSP, PRECOMPILES> ExecuteCommitEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILES>
where
    CTX: ContextSetters
        + ContextTr<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>, Db: DatabaseCommit>,
    PRECOMPILES: PrecompileProvider<Context = CTX, Output = InterpreterResult>,
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{MainBuilder, MainContext};
    use alloy_eip7702::Authorization;
    use alloy_signer::SignerSync;
    use alloy_signer_local::PrivateKeySigner;
    use bytecode::{
        opcode::{PUSH1, SSTORE},
        Bytecode,
    };
    use context::Context;
    use context_interface::TransactionType;
    use database::{BenchmarkDB, EEADDRESS, FFADDRESS};
    use primitives::{TxKind, U256};
    use specification::hardfork::SpecId;

    #[test]
    fn sanity_eip7702_tx() {
        let signer = PrivateKeySigner::random();
        let auth = Authorization {
            chain_id: U256::ZERO,
            nonce: 0,
            address: FFADDRESS,
        };
        let signature = signer.sign_hash_sync(&auth.signature_hash()).unwrap();
        let auth = auth.into_signed(signature);

        let bytecode = Bytecode::new_legacy([PUSH1, 0x01, PUSH1, 0x01, SSTORE].into());

        let ctx = Context::mainnet()
            .modify_cfg_chained(|cfg| cfg.spec = SpecId::PRAGUE)
            .with_db(BenchmarkDB::new_bytecode(bytecode))
            .modify_tx_chained(|tx| {
                tx.tx_type = TransactionType::Eip7702.into();
                tx.gas_limit = 100_000;
                tx.authorization_list = vec![auth];
                tx.caller = EEADDRESS;
                tx.kind = TxKind::Call(signer.address());
            });

        let mut evm = ctx.build_mainnet();

        let ok = evm.transact_previous().unwrap();

        let auth_acc = ok.state.get(&signer.address()).unwrap();
        assert_eq!(auth_acc.info.code, Some(Bytecode::new_eip7702(FFADDRESS)));
        assert_eq!(auth_acc.info.nonce, 1);
        assert_eq!(
            auth_acc.storage.get(&U256::from(1)).unwrap().present_value,
            U256::from(1)
        );
    }
}
