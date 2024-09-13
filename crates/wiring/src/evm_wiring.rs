use crate::{
    result::HaltReason,
    transaction::{Transaction, TransactionValidation},
    Block,
};
use core::{fmt::Debug, hash::Hash};
use database_interface::{Database, EmptyDB};
use specification::hardfork::SpecId;

/// The type that enumerates the chain's hardforks.
pub trait HardforkTrait: Clone + Copy + Default + PartialEq + Eq + Into<SpecId> {}

impl<HardforkT> HardforkTrait for HardforkT where
    HardforkT: Clone + Copy + Default + PartialEq + Eq + Into<SpecId>
{
}

pub trait HaltReasonTrait: Clone + Debug + PartialEq + Eq + From<HaltReason> {}

impl<HaltReasonT> HaltReasonTrait for HaltReasonT where
    HaltReasonT: Clone + Debug + PartialEq + Eq + From<HaltReason>
{
}

pub trait ChainSpec: Sized {
    /// Chain context type.
    type ChainContext: Sized + Default + Debug;

    /// The type that contains all block information.
    type Block: Block;

    /// The type that contains all transaction information.
    type Transaction: Transaction + TransactionValidation;

    /// The type that enumerates the chain's hardforks.
    type Hardfork: HardforkTrait;

    /// Halt reason type.
    type HaltReason: HaltReasonTrait;
}

pub trait EvmWiring: Sized {
    /// Chain specification type.
    type ChainSpec: ChainSpec;

    /// External context type
    type ExternalContext: Sized;

    /// Database type.
    type Database: Database;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EthereumChainSpec;

impl ChainSpec for EthereumChainSpec {
    type ChainContext = ();
    type Block = crate::default::block::BlockEnv;
    type Transaction = crate::default::TxEnv;
    type Hardfork = SpecId;
    type HaltReason = crate::result::HaltReason;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EthereumWiring<DB: Database, EXT> {
    phantom: core::marker::PhantomData<(DB, EXT)>,
}

impl<DB: Database, EXT: Debug> EvmWiring for EthereumWiring<DB, EXT> {
    type ChainSpec = EthereumChainSpec;
    type Database = DB;
    type ExternalContext = EXT;
}

pub type DefaultEthereumWiring = EthereumWiring<EmptyDB, ()>;
