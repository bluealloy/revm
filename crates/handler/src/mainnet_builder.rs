use crate::{instructions::EthInstructions, EthPrecompiles};
use context::{BlockEnv, Cfg, CfgEnv, Context, Evm, EvmData, Journal, TxEnv};
use context_interface::{Block, Database, JournalTr, Transaction};
use database_interface::EmptyDB;
use interpreter::interpreter::EthInterpreter;
use primitives::hardfork::SpecId;

pub type MainnetEvm<CTX, INSP = ()> =
    Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, EthPrecompiles<CTX>>;

pub trait MainBuilder: Sized {
    type Context;

    fn build_mainnet(self) -> MainnetEvm<Self::Context>;

    fn build_mainnet_with_inspector<INSP>(self, inspector: INSP)
        -> MainnetEvm<Self::Context, INSP>;
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> MainBuilder for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: JournalTr<Database = DB>,
{
    type Context = Self;

    fn build_mainnet(self) -> MainnetEvm<Self::Context> {
        Evm {
            data: EvmData {
                ctx: self,
                inspector: (),
            },
            instruction: EthInstructions::default(),
            precompiles: EthPrecompiles::default(),
        }
    }

    fn build_mainnet_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> MainnetEvm<Self::Context, INSP> {
        Evm {
            data: EvmData {
                ctx: self,
                inspector,
            },
            instruction: EthInstructions::default(),
            precompiles: EthPrecompiles::default(),
        }
    }
}

/// Trait used to initialize Context with default mainnet types.
pub trait MainContext {
    fn mainnet() -> Self;
}

impl MainContext for Context<BlockEnv, TxEnv, CfgEnv, EmptyDB, Journal<EmptyDB>, ()> {
    fn mainnet() -> Self {
        Context::new(EmptyDB::new(), SpecId::LATEST)
    }
}

#[cfg(test)]
mod test {
    use crate::ExecuteEvm;
    use crate::{MainBuilder, MainContext};
    use alloy_signer::SignerSync;
    use alloy_signer_local::PrivateKeySigner;
    use bytecode::{
        opcode::{PUSH1, SSTORE},
        Bytecode,
    };
    use context::Context;
    use context_interface::{transaction::Authorization, TransactionType};
    use database::{BenchmarkDB, EEADDRESS, FFADDRESS};
    use primitives::{hardfork::SpecId, TxKind, U256};

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

        let ok = evm.replay().unwrap();

        let auth_acc = ok.state.get(&signer.address()).unwrap();
        assert_eq!(auth_acc.info.code, Some(Bytecode::new_eip7702(FFADDRESS)));
        assert_eq!(auth_acc.info.nonce, 1);
        assert_eq!(
            auth_acc.storage.get(&U256::from(1)).unwrap().present_value,
            U256::from(1)
        );
    }
}
