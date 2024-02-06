use super::{BlockEnv, CfgEnv, Env, SpecId, TxEnv};
use alloc::boxed::Box;
use core::ops::{Deref, DerefMut};

/// Handler configuration fields. It is used to configure the handler.
/// It contains specification id and the Optimism related field if
/// optimism feature is enabled.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct HandlerCfg {
    /// Specification identification.
    pub spec_id: SpecId,
    /// Optimism related field, it will append the Optimism handle register to the EVM.
    #[cfg(feature = "optimism")]
    pub is_optimism: bool,
}

impl HandlerCfg {
    /// Creates new `HandlerCfg` instance.
    pub fn new(spec_id: SpecId) -> Self {
        Self {
            spec_id,
            #[cfg(feature = "optimism")]
            is_optimism: false,
        }
    }

    /// Creates new `HandlerCfg` instance with the optimism feature.
    #[cfg(feature = "optimism")]
    pub fn new_with_optimism(spec_id: SpecId, is_optimism: bool) -> Self {
        Self {
            spec_id,
            is_optimism,
        }
    }

    /// Returns true if the optimism feature is enabled and flag is set to true.
    pub fn is_optimism(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "optimism")] {
                self.is_optimism
            } else {
                false
            }
        }
    }
}

/// Configuration environment with the chain spec id.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CfgEnvWithHandlerCfg {
    /// Configuration environment.
    pub cfg_env: CfgEnv,
    /// Handler configuration fields.
    pub handler_cfg: HandlerCfg,
}

impl CfgEnvWithHandlerCfg {
    /// Returns new `CfgEnvWithHandlerCfg` instance.
    pub fn new(cfg_env: CfgEnv, spec_id: SpecId) -> Self {
        Self {
            cfg_env,
            handler_cfg: HandlerCfg {
                spec_id,
                #[cfg(feature = "optimism")]
                is_optimism: false,
            },
        }
    }

    /// Enables the optimism feature.
    #[cfg(feature = "optimism")]
    pub fn enable_optimism(&mut self) {
        self.handler_cfg.is_optimism = true;
    }
}

impl DerefMut for CfgEnvWithHandlerCfg {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cfg_env
    }
}

impl Deref for CfgEnvWithHandlerCfg {
    type Target = CfgEnv;

    fn deref(&self) -> &Self::Target {
        &self.cfg_env
    }
}

/// Evm environment with the chain spec id.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EnvWithHandlerCfg {
    /// Evm enironment.
    pub env: Box<Env>,
    /// Handler configuration fields.
    pub handler_cfg: HandlerCfg,
}

impl EnvWithHandlerCfg {
    /// Returns new `EnvWithHandlerCfg` instance.
    pub fn new(env: Box<Env>, spec_id: SpecId) -> Self {
        Self {
            env,
            handler_cfg: HandlerCfg {
                spec_id,
                #[cfg(feature = "optimism")]
                is_optimism: false,
            },
        }
    }

    /// Takes `CfgEnvWithHandlerCfg` and returns new `EnvWithHandlerCfg` instance.
    pub fn new_with_cfg_env(cfg: CfgEnvWithHandlerCfg, block: BlockEnv, tx: TxEnv) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(feature = "optimism")] {
                let mut new = Self::new(
                    Env::boxed(
                        cfg.cfg_env,
                        block,
                        tx,
                    ),
                    cfg.handler_cfg.spec_id,
                );
                if cfg.handler_cfg.is_optimism {
                    new.enable_optimism()
                }
                new
            } else {
            Self::new(
                Env::boxed(
                    cfg.cfg_env,
                    block,
                    tx,
                ),
                cfg.handler_cfg.spec_id,
            )
            }
        }
    }

    /// Enables the optimism handle register.
    #[cfg(feature = "optimism")]
    pub fn enable_optimism(&mut self) {
        self.handler_cfg.is_optimism = true;
    }
}

impl DerefMut for EnvWithHandlerCfg {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.env
    }
}

impl Deref for EnvWithHandlerCfg {
    type Target = Env;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}
