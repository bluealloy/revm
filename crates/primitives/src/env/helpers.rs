use super::{BlockEnv, CfgEnv, Env, SpecId, TxEnv};
use alloc::boxed::Box;
use core::ops::{Deref, DerefMut};

/// Configuration environment with the chain spec id.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CfgEnvWithSpecId {
    /// Configuration environment.
    pub cfg_env: CfgEnv,
    /// Specification identification.
    pub spec_id: SpecId,
    /// Optimism related field, it will append the Optimism handle register to the EVM.
    #[cfg(feature = "optimism")]
    pub is_optimism: bool,
}

impl CfgEnvWithSpecId {
    /// Returns new `CfgEnvWithSpecId` instance.
    pub fn new(cfg_env: CfgEnv, spec_id: SpecId) -> Self {
        Self {
            cfg_env,
            spec_id,
            #[cfg(feature = "optimism")]
            is_optimism: false,
        }
    }

    /// Enables the optimism feature.
    #[cfg(feature = "optimism")]
    pub fn enable_optimism(&mut self) {
        self.is_optimism = true;
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
    #[cfg(feature = "optimism")]
    pub is_optimism: bool,
}

impl EnvWithSpecId {
    /// Returns new `EnvWithSpecId` instance.
    pub fn new(env: Box<Env>, spec_id: SpecId) -> Self {
        Self {
            env,
            spec_id,
            #[cfg(feature = "optimism")]
            is_optimism: false,
        }
    }

    /// Takes `CfgEnvWithSpecId` and returns new `EnvWithSpecId` instance.
    pub fn new_with_cfg_env(cfg: CfgEnvWithSpecId, block: BlockEnv, tx: TxEnv) -> Self {
        #[cfg(feature = "optimism")]
        {
            let mut new = Self::new(
                Box::new(Env {
                    cfg: cfg.cfg_env,
                    block,
                    tx,
                }),
                cfg.spec_id,
            );
            if cfg.is_optimism {
                new.enable_optimism()
            }
            new
        }

        #[cfg(not(feature = "optimism"))]
        Self::new(
            Box::new(Env {
                cfg: cfg.cfg_env,
                block,
                tx,
            }),
            cfg.spec_id,
        )
    }

    /// Enables the optimism handle register.
    #[cfg(feature = "optimism")]
    pub fn enable_optimism(&mut self) {
        self.is_optimism = true;
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
