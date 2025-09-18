use crate::{frame::EthFrame, instructions::EthInstructions, EthPrecompiles};
use context::{BlockEnv, Cfg, CfgEnv, Context, Evm, FrameStack, Journal, TxEnv};
use context_interface::{Block, Database, JournalTr, Transaction};
use database_interface::EmptyDB;
use interpreter::interpreter::EthInterpreter;
use primitives::hardfork::SpecId;

/// Type alias for a mainnet EVM instance with standard Ethereum components.
pub type MainnetEvm<CTX, INSP = ()> =
    Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, EthPrecompiles, EthFrame<EthInterpreter>>;

/// Type alias for a mainnet context with standard Ethereum environment types.
pub type MainnetContext<DB> = Context<BlockEnv, TxEnv, CfgEnv, DB, Journal<DB>, ()>;

/// Trait for building mainnet EVM instances from contexts.
pub trait MainBuilder: Sized {
    /// The context type that will be used in the EVM.
    type Context;

    /// Builds a mainnet EVM instance without an inspector.
    fn build_mainnet(self) -> MainnetEvm<Self::Context>;

    /// Builds a mainnet EVM instance with the provided inspector.
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
            ctx: self,
            inspector: (),
            instruction: EthInstructions::default(),
            precompiles: EthPrecompiles::default(),
            frame_stack: FrameStack::new_prealloc(8),
        }
    }

    fn build_mainnet_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> MainnetEvm<Self::Context, INSP> {
        Evm {
            ctx: self,
            inspector,
            instruction: EthInstructions::default(),
            precompiles: EthPrecompiles::default(),
            frame_stack: FrameStack::new_prealloc(8),
        }
    }
}

/// Trait used to initialize Context with default mainnet types.
pub trait MainContext {
    /// Creates a new mainnet context with default configuration.
    fn mainnet() -> Self;
}

impl MainContext for Context<BlockEnv, TxEnv, CfgEnv, EmptyDB, Journal<EmptyDB>, ()> {
    fn mainnet() -> Self {
        Context::new(EmptyDB::new(), SpecId::default())
    }
}

#[cfg(test)]
mod test {
    use crate::ExecuteEvm;
    use crate::{MainBuilder, MainContext};
    use alloy_signer::{Either, SignerSync};
    use alloy_signer_local::PrivateKeySigner;
    use bytecode::{
        opcode::{PUSH1, SSTORE},
        Bytecode,
    };
    use context::{Context, TxEnv};
    use context_interface::transaction::Authorization;
    use database::{BenchmarkDB, EEADDRESS, FFADDRESS};
    use primitives::{hardfork::SpecId, TxKind, U256};
    use primitives::{StorageKey, StorageValue};

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
            .with_db(BenchmarkDB::new_bytecode(bytecode));

        let mut evm = ctx.build_mainnet();

        let state = evm
            .transact(
                TxEnv::builder()
                    .gas_limit(100_000)
                    .authorization_list(vec![Either::Left(auth)])
                    .caller(EEADDRESS)
                    .gas_priority_fee(Some(0))
                    .kind(TxKind::Call(signer.address()))
                    .build()
                    .unwrap(),
            )
            .unwrap()
            .state;

        let auth_acc = state.get(&signer.address()).unwrap();
        assert_eq!(auth_acc.info.code, Some(Bytecode::new_eip7702(FFADDRESS)));
        assert_eq!(auth_acc.info.nonce, 1);
        assert_eq!(
            auth_acc
                .storage
                .get(&StorageKey::from(1))
                .unwrap()
                .present_value,
            StorageValue::from(1)
        );
    }
}
