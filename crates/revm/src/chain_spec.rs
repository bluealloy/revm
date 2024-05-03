use crate::primitives::EthSpecId;

pub trait ChainSpec: Sized + 'static {
    /// The type that enumerates the chain's hardforks.
    type Hardfork: Default + PartialEq + Eq + Into<EthSpecId>;

    /// The type that enumerates chain-specific halt reasons.
    type HaltReason;
}

pub struct MainnetChainSpec;

impl ChainSpec for MainnetChainSpec {
    type Hardfork = crate::primitives::EthSpecId;
    type HaltReason = ();
}
