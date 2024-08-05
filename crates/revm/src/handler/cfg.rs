use crate::primitives::{CfgEnv, Env, EnvWiring, EvmWiring};
use core::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use derive_where::derive_where;
use std::boxed::Box;

/// Configuration environment with the chain spec id.
#[derive(Debug, Eq, PartialEq)]
#[derive_where(Clone; EvmWiringT::Hardfork)]
pub struct CfgEnvWithEvmWiring<EvmWiringT: EvmWiring> {
    /// Configuration environment.
    pub cfg_env: CfgEnv,
    /// Handler configuration fields.
    pub spec_id: EvmWiringT::Hardfork,
}

impl<EvmWiringT: EvmWiring> CfgEnvWithEvmWiring<EvmWiringT> {
    /// Returns new instance of `CfgEnvWithHandlerCfg`.
    pub fn new(cfg_env: CfgEnv, spec_id: EvmWiringT::Hardfork) -> Self {
        Self { cfg_env, spec_id }
    }
}

impl<EvmWiringT: EvmWiring> DerefMut for CfgEnvWithEvmWiring<EvmWiringT> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cfg_env
    }
}

impl<EvmWiringT: EvmWiring> Deref for CfgEnvWithEvmWiring<EvmWiringT> {
    type Target = CfgEnv;

    fn deref(&self) -> &Self::Target {
        &self.cfg_env
    }
}

/// Evm environment with the chain spec id.
#[derive_where(Clone, Debug; EvmWiringT::Block, EvmWiringT::Hardfork, EvmWiringT::Transaction)]
pub struct EnvWithEvmWiring<EvmWiringT>
where
    EvmWiringT: EvmWiring,
{
    /// Evm enironment.
    pub env: Box<EnvWiring<EvmWiringT>>,
    /// Handler configuration fields.
    pub spec_id: EvmWiringT::Hardfork,
}

impl<EvmWiringT> EnvWithEvmWiring<EvmWiringT>
where
    EvmWiringT: EvmWiring<Transaction: Clone + Debug>,
{
    /// Returns new `EnvWithHandlerCfg` instance.
    pub fn new(env: Box<EnvWiring<EvmWiringT>>, spec_id: EvmWiringT::Hardfork) -> Self {
        Self { env, spec_id }
    }

    /// Takes `CfgEnvWithHandlerCfg` and returns new `EnvWithHandlerCfg` instance.
    pub fn new_with_cfg_env(
        cfg: CfgEnvWithEvmWiring<EvmWiringT>,
        block: EvmWiringT::Block,
        tx: EvmWiringT::Transaction,
    ) -> Self {
        Self::new(Env::boxed(cfg.cfg_env, block, tx), cfg.spec_id)
    }

    /// Returns the specification id.
    pub const fn spec_id(&self) -> EvmWiringT::Hardfork {
        self.spec_id
    }
}

impl<EvmWiringT> DerefMut for EnvWithEvmWiring<EvmWiringT>
where
    EvmWiringT: EvmWiring<Transaction: Clone + Debug>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.env
    }
}

impl<EvmWiringT> Deref for EnvWithEvmWiring<EvmWiringT>
where
    EvmWiringT: EvmWiring<Transaction: Clone + Debug>,
{
    type Target = EnvWiring<EvmWiringT>;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}
