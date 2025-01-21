use context_interface::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    DatabaseGetter,
};
use database_interface::{Database, DatabaseCommit};
use handler::{
    handler::{EthContext, EthHandler, EthHandlerImpl},
    EthFrame, EthPrecompileProvider,
};

pub fn transact_main<DB: Database, CTX: EthContext + DatabaseGetter<Database = DB>>(
    ctx: &mut CTX,
) -> Result<ResultAndState<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>> {
    EthHandlerImpl::<CTX, _, EthFrame<CTX, _, _, _>, EthPrecompileProvider<CTX, _>, _>::default()
        .run(ctx)
}

pub fn transact_main_commit<
    DB: Database + DatabaseCommit,
    CTX: EthContext + DatabaseGetter<Database = DB>,
>(
    ctx: &mut CTX,
) -> Result<ExecutionResult<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>> {
    transact_main(ctx).map(|r| {
        ctx.db().commit(r.state);
        r.result
    })
}

/*

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        handler::mainnet::{EthExecution, EthPostExecution, EthPreExecution, EthValidation},
        EvmHandler,
    };
    use bytecode::{
        opcode::{PUSH1, SSTORE},
        Bytecode,
    };
    use core::{fmt::Debug, hash::Hash};
    use database::BenchmarkDB;
    use database_interface::Database;
    use interpreter::table::InstructionTables;
    use primitives::{address, TxKind, U256};
    use specification::{
        eip7702::{Authorization, RecoveredAuthorization, Signature},
        hardfork::{Spec, SpecId},
        spec_to_generic,
    };
    use transaction::TransactionType;
    use context_interface::{
        default::{self, block::BlockEnv, Env, TxEnv},
        result::{EVMErrorWiring, HaltReason},
        EthereumWiring, EvmWiring as InnerEvmWiring,
    };

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
    struct CEthereumWiring<'a, DB: Database, EXT> {
        phantom: core::marker::PhantomData<&'a (DB, EXT)>,
    }

    impl<'a, DB: Database, EXT: Debug> InnerEvmWiring for CEthereumWiring<'a, DB, EXT> {
        type Database = DB;
        type ExternalContext = EXT;
        type ChainContext = ();
        type Block = default::block::BlockEnv;
        type Transaction = &'a default::TxEnv;
        type Hardfork = SpecId;
        type HaltReason = HaltReason;
    }

    impl<'a, DB: Database, EXT: Debug> EvmWiring for CEthereumWiring<'a, DB, EXT> {
        fn handler<'evm>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self>
        where
            DB: Database,
            'a: 'evm,
        {
            spec_to_generic!(
                hardfork,
                EvmHandler {
                    spec_id: hardfork,
                    //instruction_table: InstructionTables::new_plain::<SPEC>(),
                    registers: Vec::new(),
                    pre_execution:
                        EthPreExecution::<Context<Self>, EVMErrorWiring<Self>>::new_boxed(
                            SPEC::SPEC_ID
                        ),
                    validation: EthValidation::<Context<Self>, EVMErrorWiring<Self>>::new_boxed(
                        SPEC::SPEC_ID
                    ),
                    post_execution: EthPostExecution::<
                        Context<Self>,
                        EVMErrorWiring<Self>,
                        HaltReason,
                    >::new_boxed(SPEC::SPEC_ID),
                    execution: EthExecution::<Context<Self>, EVMErrorWiring<Self>>::new_boxed(
                        SPEC::SPEC_ID
                    ),
                }
            )
        }
    }

    //pub type DefaultEthereumWiring = EthereumWiring<EmptyDB, ()>;

    #[test]
    fn sanity_tx_ref() {
        let delegate = address!("0000000000000000000000000000000000000000");
        let caller = address!("0000000000000000000000000000000000000001");
        let auth = address!("0000000000000000000000000000000000000100");

        let mut tx = TxEnv::default();
        tx.tx_type = TransactionType::Eip7702;
        tx.gas_limit = 100_000;
        tx.authorization_list = vec![RecoveredAuthorization::new_unchecked(
            Authorization {
                chain_id: U256::from(1),
                address: delegate,
                nonce: 0,
            }
            .into_signed(Signature::test_signature()),
            Some(auth),
        )]
        .into();
        tx.caller = caller;
        tx.kind = TxKind::Call(auth);

        let mut tx2 = TxEnv::default();
        tx2.tx_type = TransactionType::Legacy;
        // `nonce` was bumped from 0 to 1
        tx2.nonce = 1;

        let mut evm = EvmBuilder::new_with(
            BenchmarkDB::default(),
            (),
            Env::boxed(CfgEnv::default(), BlockEnv::default(), &tx),
            CEthereumcontext_interface::handler(SpecId::LATEST),
        )
        .build();

        let _ = evm.transact().unwrap();

        let mut evm = evm
            .modify()
            .modify_tx_env(|t| {
                *t = &tx2;
            })
            .build();

        let _ = evm.transact().unwrap();
    }

    #[test]
    fn sanity_eip7702_tx() {
        let delegate = address!("0000000000000000000000000000000000000000");
        let caller = address!("0000000000000000000000000000000000000001");
        let auth = address!("0000000000000000000000000000000000000100");

        let bytecode = Bytecode::new_legacy([PUSH1, 0x01, PUSH1, 0x01, SSTORE].into());

        let mut evm = Evm::<EthereumWiring<BenchmarkDB, ()>>::builder()
            .with_spec_id(SpecId::PRAGUE)
            .with_db(BenchmarkDB::new_bytecode(bytecode))
            .with_default_ext_context()
            .modify_tx_env(|tx| {
                tx.tx_type = TransactionType::Eip7702;
                tx.gas_limit = 100_000;
                tx.authorization_list = vec![RecoveredAuthorization::new_unchecked(
                    Authorization {
                        chain_id: U256::from(1),
                        address: delegate,
                        nonce: 0,
                    }
                    .into_signed(Signature::test_signature()),
                    Some(auth),
                )]
                .into();
                tx.caller = caller;
                tx.kind = TxKind::Call(auth);
            })
            .build();

        let ok = evm.transact().unwrap();

        let auth_acc = ok.state.get(&auth).unwrap();
        assert_eq!(auth_acc.info.code, Some(Bytecode::new_eip7702(delegate)));
        assert_eq!(auth_acc.info.nonce, 1);
        assert_eq!(
            auth_acc.storage.get(&U256::from(1)).unwrap().present_value,
            U256::from(1)
        );
    }
}

*/
