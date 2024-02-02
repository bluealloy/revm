use super::{CfgEnv, Env, SpecId};

/// Configuration environment with the chain spec id.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CfgEnvWithSpecId {
    /// Configuration environment.
    pub cfg_env: CfgEnv,
    /// Specification identification.
    pub spec_id: SpecId,
}

/// Evm environment with the chain spec id.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EnvWithSpecId {
    /// Evm enironment.
    pub env: Box<Env>,
    /// Specification identification.
    pub spec_id: SpecId,
}
