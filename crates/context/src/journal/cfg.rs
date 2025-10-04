//! Module containing the [`CfgInner`] that is part of [`crate::Cfg`].

use context_interface::Cfg;
use primitives::hardfork::SpecId;

/// Configuration for the journal.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JournalCfg {
    /// Whether BAL is enabled. If true Account with fill the data for BalBuilder that is used to
    /// create BAL. If false, BAL creating is going to be skipped and BAL returned from Database
    /// is not going to be read.x§
    pub bal: bool,
    /// Whether state clear is enabled. If true, state clear is going to be applied to the state.
    pub state_clear: bool,
    /// EIP-6780 (Cancun hard-fork): selfdestruct only if contract is created in the same tx
    pub selfdestruct_only_in_same_tx: bool,
}

impl JournalCfg {
    /// Creates a new [`JournalCfg`] from a [`Cfg`].
    pub fn new(cfg: impl Cfg) -> Self {
        Self {
            bal: cfg.bal_enabled(),
            state_clear: cfg.spec().into().is_enabled_in(SpecId::SPURIOUS_DRAGON),
            selfdestruct_only_in_same_tx: cfg.spec().into().is_enabled_in(SpecId::CANCUN),
        }
    }

    /// Returns whether BAL is enabled.
    pub fn bal(&self) -> bool {
        self.bal
    }

    /// Returns whether state clear is enabled.
    pub fn state_clear(&self) -> bool {
        self.state_clear
    }

    /// Returns whether selfdestruct only in same tx is enabled.
    pub fn selfdestruct_only_in_same_tx(&self) -> bool {
        self.selfdestruct_only_in_same_tx
    }
}
