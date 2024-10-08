use crate::{
    optimism_handle_register,
    transaction::{OpTransaction, OpTransactionType, OpTxTrait},
    L1BlockInfo, OpTransactionError, OptimismHaltReason, OptimismSpecId,
};
use core::marker::PhantomData;
use revm::{
    database_interface::Database,
    handler::register::HandleRegisters,
    wiring::default::{block::BlockEnv, TxEnv},
    wiring::EvmWiring,
    EvmHandler,
};

pub trait OptimismContextTrait {
    /// A reference to the cached L1 block info.
    fn l1_block_info(&self) -> Option<&L1BlockInfo>;

    /// A mutable reference to the cached L1 block info.
    fn l1_block_info_mut(&mut self) -> &mut Option<L1BlockInfo>;
}

/// Trait for an Optimism chain spec.
pub trait OptimismWiring:
    revm::EvmWiring<
    ChainContext: OptimismContextTrait,
    Hardfork = OptimismSpecId,
    HaltReason = OptimismHaltReason,
    Transaction: OpTxTrait<
        TransactionType = OpTransactionType,
        TransactionError = OpTransactionError,
    >,
>
{
}

impl<EvmWiringT> OptimismWiring for EvmWiringT where
    EvmWiringT: revm::EvmWiring<
        ChainContext: OptimismContextTrait,
        Hardfork = OptimismSpecId,
        HaltReason = OptimismHaltReason,
        Transaction: OpTxTrait<
            TransactionType = OpTransactionType,
            TransactionError = OpTransactionError,
        >,
    >
{
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OptimismEvmWiring<DB: Database, EXT> {
    _phantom: PhantomData<(DB, EXT)>,
}

impl<DB: Database, EXT> EvmWiring for OptimismEvmWiring<DB, EXT> {
    type Block = BlockEnv;
    type Database = DB;
    type ChainContext = Context;
    type ExternalContext = EXT;
    type Hardfork = OptimismSpecId;
    type HaltReason = OptimismHaltReason;
    type Transaction = OpTransaction<TxEnv>;
}

impl<DB: Database, EXT> revm::EvmWiring for OptimismEvmWiring<DB, EXT> {
    fn handler<'evm>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self>
    where
        DB: Database,
    {
        let mut handler = EvmHandler::mainnet_with_spec(hardfork);

        handler.append_handler_register(HandleRegisters::Plain(optimism_handle_register::<Self>));

        handler
    }
}

/// Context for the Optimism chain.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Context {
    l1_block_info: Option<L1BlockInfo>,
}

impl OptimismContextTrait for Context {
    fn l1_block_info(&self) -> Option<&L1BlockInfo> {
        self.l1_block_info.as_ref()
    }

    fn l1_block_info_mut(&mut self) -> &mut Option<L1BlockInfo> {
        &mut self.l1_block_info
    }
}
