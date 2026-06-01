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
        let spec = self.cfg.spec().into();
        Evm {
            ctx: self,
            inspector: (),
            instruction: EthInstructions::new_mainnet_with_spec(spec),
            precompiles: EthPrecompiles::new(spec),
            frame_stack: FrameStack::new_prealloc(8),
        }
    }

    fn build_mainnet_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> MainnetEvm<Self::Context, INSP> {
        let spec = self.cfg.spec().into();
        Evm {
            ctx: self,
            inspector,
            instruction: EthInstructions::new_mainnet_with_spec(spec),
            precompiles: EthPrecompiles::new(spec),
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
    use crate::{ExecuteEvm, MainBuilder, MainContext};
    use alloy_signer::{Either, SignerSync};
    use alloy_signer_local::PrivateKeySigner;
    use bytecode::{
        opcode::{PUSH1, SSTORE},
        Bytecode,
    };
    use context::{Context, TxEnv};
    use context_interface::transaction::Authorization;
    use database::{BenchmarkDB, EEADDRESS, FFADDRESS};
    use primitives::{hardfork::SpecId, StorageKey, StorageValue, TxKind, U256};

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
            .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SpecId::PRAGUE))
            .with_db(BenchmarkDB::new_bytecode(bytecode));

        let mut evm = ctx.build_mainnet();

        let state = evm
            .transact(
                TxEnv::builder()
                    .gas_limit(100_000)
                    .authorization_list(vec![Either::Left(auth)])
                    .caller(EEADDRESS)
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

    #[test]
    fn tip1060_gas_token_mint_consume_and_revert() {
        use context_interface::{
            host::{storage_gas_token_slot, GasTokenOp},
            ContextTr, Host, JournalTr,
        };
        use primitives::Address;

        let token = Address::with_last_byte(0xAA);
        let account = Address::with_last_byte(0xBB);
        let slot = storage_gas_token_slot(account);

        let mut ctx = Context::mainnet().modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::PRAGUE);
            cfg.storage_gas_token_contract = Some(token);
        });

        // Mint: counter 0 -> 1 (no consume, always writes).
        let res = ctx
            .sstore_state_gas_token(token, account, GasTokenOp::Mint, false)
            .unwrap();
        assert!(!res.data.consumed);
        assert_eq!(res.data.counter.unwrap().new_value, U256::from(1));
        assert_eq!(ctx.sload(token, slot).unwrap().data, U256::from(1));

        // Mint again: 1 -> 2.
        ctx.sstore_state_gas_token(token, account, GasTokenOp::Mint, false)
            .unwrap();
        assert_eq!(ctx.sload(token, slot).unwrap().data, U256::from(2));

        // Consume: 2 -> 1, token consumed.
        let res = ctx
            .sstore_state_gas_token(token, account, GasTokenOp::Consume, false)
            .unwrap();
        assert!(res.data.consumed);
        assert_eq!(ctx.sload(token, slot).unwrap().data, U256::from(1));

        // The mint is journaled: a reverted checkpoint rolls the counter back.
        let cp = ctx.journal_mut().checkpoint();
        ctx.sstore_state_gas_token(token, account, GasTokenOp::Mint, false)
            .unwrap();
        assert_eq!(ctx.sload(token, slot).unwrap().data, U256::from(2));
        ctx.journal_mut().checkpoint_revert(cp);
        assert_eq!(ctx.sload(token, slot).unwrap().data, U256::from(1));

        // Drain to zero, then consume on an empty counter: no write, nothing consumed.
        ctx.sstore_state_gas_token(token, account, GasTokenOp::Consume, false)
            .unwrap();
        assert_eq!(ctx.sload(token, slot).unwrap().data, U256::ZERO);
        let res = ctx
            .sstore_state_gas_token(token, account, GasTokenOp::Consume, false)
            .unwrap();
        assert!(!res.data.consumed);
        assert!(res.data.counter.is_none());
        assert_eq!(ctx.sload(token, slot).unwrap().data, U256::ZERO);
    }
}
