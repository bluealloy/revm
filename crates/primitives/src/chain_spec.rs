use cfg_if::cfg_if;

use crate::{Block, InvalidTransaction, SpecId, Transaction};

use core::{
    fmt::{Debug, Display},
    hash::Hash,
};

/// The type that enumerates the chain's hardforks.
pub trait HardforkTrait: Clone + Copy + Default + PartialEq + Eq + Into<SpecId> {}

cfg_if! {
    if #[cfg(feature = "serde")] {
        /// The type that enumerates chain-specific halt reasons.
        pub trait HaltReasonTrait: Clone + Debug + PartialEq + Eq + Hash + From<crate::HaltReason> + serde::de::DeserializeOwned + serde::Serialize {}
    } else {
        /// The type that enumerates chain-specific halt reasons.
        pub trait HaltReasonTrait: Clone + Debug + PartialEq + Eq + Hash + From<crate::HaltReason> {}
    }
}

pub trait ChainSpec: Clone + Debug + Default + Sized + 'static {
    /// The type that contains all block information.
    type Block: Block;

    /// The type that contains all transaction information.
    type Transaction: Transaction;

    /// The type that enumerates the chain's hardforks.
    type Hardfork: HardforkTrait;

    /// Halt reason type.
    type HaltReason: HaltReasonTrait;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EthChainSpec;

impl ChainSpec for EthChainSpec {
    type Block = crate::BlockEnv;
    type Hardfork = SpecId;
    type HaltReason = crate::HaltReason;
    type Transaction = crate::TxEnv;
}
