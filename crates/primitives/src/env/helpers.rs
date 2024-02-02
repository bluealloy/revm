use core::ops::{Deref, DerefMut};

use super::{BlockEnv, CfgEnv, Env, SpecId, TxEnv};

/// Configuration environment with the chain spec id.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CfgEnvWithSpecId {
    /// Configuration environment.
    pub cfg_env: CfgEnv,
    /// Specification identification.
    pub spec_id: SpecId,
}

impl CfgEnvWithSpecId {
    /// Returns new `CfgEnvWithSpecId` instance.
    pub fn new(cfg_env: CfgEnv, spec_id: SpecId) -> Self {
        Self { cfg_env, spec_id }
    }
}

impl DerefMut for CfgEnvWithSpecId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cfg_env
    }
}

impl Deref for CfgEnvWithSpecId {
    type Target = CfgEnv;

    fn deref(&self) -> &Self::Target {
        &self.cfg_env
    }
}

/// Evm environment with the chain spec id.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EnvWithSpecId {
    /// Evm enironment.
    pub env: Box<Env>,
    /// Specification identification.
    pub spec_id: SpecId,
}

impl EnvWithSpecId {
    /// Returns new `EnvWithSpecId` instance.
    pub fn new(env: Box<Env>, spec_id: SpecId) -> Self {
        Self { env, spec_id }
    }

    /// Takes `CfgEnvWithSpecId` and returns new `EnvWithSpecId` instance.
    pub fn new_with_cfg_env(cfg: CfgEnvWithSpecId, block: BlockEnv, tx: TxEnv) -> Self {
        Self::new(
            Box::new(Env {
                cfg: cfg.cfg_env,
                block,
                tx,
            }),
            cfg.spec_id,
        )
    }
}

impl DerefMut for EnvWithSpecId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.env
    }
}

impl Deref for EnvWithSpecId {
    type Target = Env;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}
