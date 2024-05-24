use cfg_if::cfg_if;

use crate::{Block, InvalidTransaction, SpecId, Transaction};

use core::{
    fmt::{Debug, Display},
    hash::Hash,
};

pub trait ChainSpec: Clone + Debug + Default + Sized + 'static {
    /// The type that contains all block information.
    type Block: Block + Clone + Debug + Default + PartialEq + Eq;

    /// The type that enumerates the chain's hardforks.
    type Hardfork: Clone + Copy + Default + PartialEq + Eq + Into<SpecId>;

    cfg_if! {
        if #[cfg(feature = "serde")] {
            /// The type that enumerates chain-specific halt reasons.
            type HaltReason: Clone + Debug + PartialEq + Eq + Hash + From<crate::HaltReason> + serde::de::DeserializeOwned + serde::Serialize;
        } else {
            /// The type that enumerates chain-specific halt reasons.
            type HaltReason: Clone + Debug + PartialEq + Eq + Hash + From<crate::HaltReason>;
        }
    }

    /// The type that contains all transaction information.
    type Transaction: Clone + Debug + Default + PartialEq + Eq + Transaction;
    /// The error type that can be returned when validating a transaction.
    type TransactionValidationError: Debug + Display;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EthChainSpec;

impl ChainSpec for EthChainSpec {
    type Block = crate::BlockEnv;
    type Hardfork = SpecId;
    type HaltReason = crate::HaltReason;
    type Transaction = crate::TxEnv;
    type TransactionValidationError = InvalidTransaction;
}
