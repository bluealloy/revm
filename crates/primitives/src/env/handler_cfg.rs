use super::{BlockEnv, CfgEnv, Env, SpecId, TxEnv};
use core::ops::{Deref, DerefMut};
use std::boxed::Box;

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
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "optimism-default-handler",
                not(feature = "negate-optimism-default-handler")))] {
                    let is_optimism = true;
            } else if #[cfg(feature = "optimism")] {
                let is_optimism = false;
            }
        }
        Self {
            spec_id,
            #[cfg(feature = "optimism")]
            is_optimism,
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
    /// Returns new instance of `CfgEnvWithHandlerCfg` with the handler configuration.
    pub fn new(cfg_env: CfgEnv, handler_cfg: HandlerCfg) -> Self {
        Self {
            cfg_env,
            handler_cfg,
        }
    }

    /// Returns new `CfgEnvWithHandlerCfg` instance with the chain spec id.
    ///
    /// is_optimism will be set to default value depending on `optimism-default-handler` feature.
    pub fn new_with_spec_id(cfg_env: CfgEnv, spec_id: SpecId) -> Self {
        Self::new(cfg_env, HandlerCfg::new(spec_id))
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
    pub fn new(env: Box<Env>, handler_cfg: HandlerCfg) -> Self {
        Self { env, handler_cfg }
    }

    /// Returns new `EnvWithHandlerCfg` instance with the chain spec id.
    ///
    /// is_optimism will be set to default value depending on `optimism-default-handler` feature.
    pub fn new_with_spec_id(env: Box<Env>, spec_id: SpecId) -> Self {
        Self::new(env, HandlerCfg::new(spec_id))
    }

    /// Takes `CfgEnvWithHandlerCfg` and returns new `EnvWithHandlerCfg` instance.
    pub fn new_with_cfg_env(cfg: CfgEnvWithHandlerCfg, block: BlockEnv, tx: TxEnv) -> Self {
        Self::new(Env::boxed(cfg.cfg_env, block, tx), cfg.handler_cfg)
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
