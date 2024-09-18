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

pub trait EvmWiring: Sized {
    /// External context type
    type ExternalContext: Sized;

    /// Chain context type.
    type ChainContext: Sized + Default + Debug;

    /// Database type.
    type Database: Database;

    /// The type that contains all block information.
    type Block: Block;

    /// The type that contains all transaction information.
    type Transaction: Transaction + TransactionValidation;

    /// The type that enumerates the chain's hardforks.
    type Hardfork: HardforkTrait;

    /// Halt reason type.
    type HaltReason: HaltReasonTrait;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EthereumWiring<DB: Database, EXT> {
    phantom: core::marker::PhantomData<(DB, EXT)>,
}

impl<DB: Database, EXT: Debug> EvmWiring for EthereumWiring<DB, EXT> {
    type Database = DB;
    type ExternalContext = EXT;
    type ChainContext = ();
    type Block = crate::default::block::BlockEnv;
    type Transaction = crate::default::TxEnv;
    type Hardfork = SpecId;
    type HaltReason = crate::result::HaltReason;
}

pub type DefaultEthereumWiring = EthereumWiring<EmptyDB, ()>;
