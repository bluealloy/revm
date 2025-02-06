use context::{BlockEnv, Cfg, CfgEnv, Context, Evm, EvmData, JournaledState, TxEnv};
use context_interface::{Block, Database, Journal, Transaction};
use database_interface::EmptyDB;
use handler::{instructions::EthInstructions, noop::NoOpInspector, EthPrecompiles};
use interpreter::interpreter::EthInterpreter;
use primitives::Log;
use specification::hardfork::SpecId;
use state::EvmState;
use std::vec::Vec;

pub trait MainBuilder: Sized {
    type Context;

    fn build_mainnet(
        self,
    ) -> Evm<
        Self::Context,
        NoOpInspector,
        EthInstructions<EthInterpreter, Self::Context>,
        EthPrecompiles<Self::Context>,
    >;

    fn build_mainnet_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> Evm<
        Self::Context,
        INSP,
        EthInstructions<EthInterpreter, Self::Context>,
        EthPrecompiles<Self::Context>,
    >;
}

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

/// Trait used to initialize Context with default mainnet types.
pub trait MainContext {
    fn mainnet() -> Self;
}

impl MainContext for Context<BlockEnv, TxEnv, CfgEnv, EmptyDB, JournaledState<EmptyDB>, ()> {
    fn mainnet() -> Self {
        Context::new(EmptyDB::new(), SpecId::LATEST)
    }
}
