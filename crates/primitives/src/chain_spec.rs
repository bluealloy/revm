use cfg_if::cfg_if;

use crate::{Block, SpecId, Transaction};

use core::{fmt::Debug, hash::Hash};

/// The type that enumerates the chain's hardforks.
pub trait HardforkTrait: Clone + Copy + Default + PartialEq + Eq + Into<SpecId> {}

impl<HardforkT> HardforkTrait for HardforkT where
    HardforkT: Clone + Copy + Default + PartialEq + Eq + Into<SpecId>
{
}

/// The type that enumerates chain-specific halt reasons.
pub trait HaltReasonTrait: Clone + Debug + PartialEq + Eq + From<crate::HaltReason> {}

impl<HaltReasonT> HaltReasonTrait for HaltReasonT where
    HaltReasonT: Clone + Debug + PartialEq + Eq + From<crate::HaltReason>
{
}

pub trait TransactionValidation {
    cfg_if! {
        if #[cfg(feature = "std")] {
            /// An error that occurs when validating a transaction.
            type ValidationError: Debug + std::error::Error;
        } else {
            /// An error that occurs when validating a transaction.
            type ValidationError: Debug;
        }
    }
}

pub trait EvmWiring: Sized + 'static {
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
pub struct EthEvmWiring;

impl EvmWiring for EthEvmWiring {
    type Block = crate::BlockEnv;
    type Hardfork = SpecId;
    type HaltReason = crate::HaltReason;
    type Transaction = crate::TxEnv;
}
