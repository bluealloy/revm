use cfg_if::cfg_if;

use crate::{InvalidTransaction, SpecId};

use core::{
    fmt::{Debug, Display},
    hash::Hash,
};

pub trait ChainSpec: Clone + Debug + Sized + 'static {
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

    type TransactionValidationError: Debug + Display;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EthChainSpec;

impl ChainSpec for EthChainSpec {
    type Hardfork = SpecId;
    type HaltReason = crate::HaltReason;
    type TransactionValidationError = InvalidTransaction;
}
