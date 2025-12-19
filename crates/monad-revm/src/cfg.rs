//! Monad-specific EVM configuration.
//!
//! This module provides [`MonadCfgEnv`], a wrapper around [`CfgEnv<MonadSpecId>`] that
//! implements the [`Cfg`] trait with Monad-specific defaults.

use crate::MonadSpecId;
use core::ops::{Deref, DerefMut};
use revm::context::{Cfg, CfgEnv};

/// Monad maximum contract code size.
///
/// Monad uses a larger code size limit than Ethereum's EIP-170 (24KB).
/// Set to 128KB (0x20000) to allow larger contracts.
pub const MONAD_MAX_CODE_SIZE: usize = 0x20000; // 128KB

/// Monad maximum initcode size.
///
/// Following EIP-3860 pattern (2x code size), this is 256KB.
pub const MONAD_MAX_INITCODE_SIZE: usize = MONAD_MAX_CODE_SIZE * 2; // 256KB

/// Monad-specific EVM configuration.
///
/// This is a newtype wrapper around [`CfgEnv<MonadSpecId>`] that implements
/// the [`Cfg`] trait with Monad-specific defaults for:
/// - `max_code_size()`: Returns [`MONAD_MAX_CODE_SIZE`] (128KB) instead of EIP-170's 24KB
/// - `max_initcode_size()`: Returns [`MONAD_MAX_INITCODE_SIZE`] (256KB) instead of EIP-3860's 48KB
///
/// All other configuration options are delegated to the inner [`CfgEnv`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MonadCfgEnv(pub CfgEnv<MonadSpecId>);

impl MonadCfgEnv {
    /// Creates a new `MonadCfgEnv` with default Monad spec.
    pub fn new() -> Self {
        Self(CfgEnv::new_with_spec(MonadSpecId::default()))
    }

    /// Creates a new `MonadCfgEnv` with the specified spec.
    pub fn new_with_spec(spec: MonadSpecId) -> Self {
        Self(CfgEnv::new_with_spec(spec))
    }

    /// Returns a reference to the inner `CfgEnv`.
    pub const fn inner(&self) -> &CfgEnv<MonadSpecId> {
        &self.0
    }

    /// Returns a mutable reference to the inner `CfgEnv`.
    pub fn inner_mut(&mut self) -> &mut CfgEnv<MonadSpecId> {
        &mut self.0
    }

    /// Consumes self and returns the inner `CfgEnv`.
    pub fn into_inner(self) -> CfgEnv<MonadSpecId> {
        self.0
    }

    /// Sets the chain ID.
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.0.chain_id = chain_id;
        self
    }
}

impl Default for MonadCfgEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl From<CfgEnv<MonadSpecId>> for MonadCfgEnv {
    fn from(cfg: CfgEnv<MonadSpecId>) -> Self {
        Self(cfg)
    }
}

impl From<MonadCfgEnv> for CfgEnv<MonadSpecId> {
    fn from(cfg: MonadCfgEnv) -> Self {
        cfg.0
    }
}

impl Deref for MonadCfgEnv {
    type Target = CfgEnv<MonadSpecId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MonadCfgEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Cfg for MonadCfgEnv {
    type Spec = MonadSpecId;

    #[inline]
    fn chain_id(&self) -> u64 {
        self.0.chain_id
    }

    #[inline]
    fn spec(&self) -> Self::Spec {
        self.0.spec
    }

    #[inline]
    fn tx_chain_id_check(&self) -> bool {
        self.0.tx_chain_id_check
    }

    #[inline]
    fn tx_gas_limit_cap(&self) -> u64 {
        // Delegate to inner - Monad doesn't change this
        <CfgEnv<MonadSpecId> as Cfg>::tx_gas_limit_cap(&self.0)
    }

    #[inline]
    fn max_blobs_per_tx(&self) -> Option<u64> {
        self.0.max_blobs_per_tx
    }

    /// Returns Monad's max code size.
    ///
    /// Uses [`MONAD_MAX_CODE_SIZE`] as default instead of EIP-170's 24KB.
    /// Can still be overridden via `limit_contract_code_size`.
    fn max_code_size(&self) -> usize {
        self.0
            .limit_contract_code_size
            .unwrap_or(MONAD_MAX_CODE_SIZE)
    }

    /// Returns Monad's max initcode size.
    ///
    /// Uses [`MONAD_MAX_INITCODE_SIZE`] as default instead of EIP-3860's 48KB.
    /// Can still be overridden via `limit_contract_initcode_size`.
    fn max_initcode_size(&self) -> usize {
        self.0
            .limit_contract_initcode_size
            .or_else(|| {
                self.0
                    .limit_contract_code_size
                    .map(|size| size.saturating_mul(2))
            })
            .unwrap_or(MONAD_MAX_INITCODE_SIZE)
    }

    fn is_eip3541_disabled(&self) -> bool {
        <CfgEnv<MonadSpecId> as Cfg>::is_eip3541_disabled(&self.0)
    }

    fn is_eip3607_disabled(&self) -> bool {
        <CfgEnv<MonadSpecId> as Cfg>::is_eip3607_disabled(&self.0)
    }

    fn is_eip7623_disabled(&self) -> bool {
        <CfgEnv<MonadSpecId> as Cfg>::is_eip7623_disabled(&self.0)
    }

    fn is_balance_check_disabled(&self) -> bool {
        <CfgEnv<MonadSpecId> as Cfg>::is_balance_check_disabled(&self.0)
    }

    fn is_block_gas_limit_disabled(&self) -> bool {
        <CfgEnv<MonadSpecId> as Cfg>::is_block_gas_limit_disabled(&self.0)
    }

    fn is_nonce_check_disabled(&self) -> bool {
        self.0.disable_nonce_check
    }

    fn is_base_fee_check_disabled(&self) -> bool {
        <CfgEnv<MonadSpecId> as Cfg>::is_base_fee_check_disabled(&self.0)
    }

    fn is_priority_fee_check_disabled(&self) -> bool {
        <CfgEnv<MonadSpecId> as Cfg>::is_priority_fee_check_disabled(&self.0)
    }

    fn is_fee_charge_disabled(&self) -> bool {
        <CfgEnv<MonadSpecId> as Cfg>::is_fee_charge_disabled(&self.0)
    }

    fn memory_limit(&self) -> u64 {
        <CfgEnv<MonadSpecId> as Cfg>::memory_limit(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monad_defaults() {
        let cfg = MonadCfgEnv::new();

        // Verify Monad-specific defaults
        assert_eq!(cfg.max_code_size(), MONAD_MAX_CODE_SIZE);
        assert_eq!(cfg.max_initcode_size(), MONAD_MAX_INITCODE_SIZE);

        // Verify we can still override
        let mut cfg = MonadCfgEnv::new();
        cfg.0.limit_contract_code_size = Some(100_000);
        assert_eq!(cfg.max_code_size(), 100_000);
        assert_eq!(cfg.max_initcode_size(), 200_000);
    }

    #[test]
    fn test_from_cfg_env() {
        let cfg_env = CfgEnv::new_with_spec(MonadSpecId::default());
        let monad_cfg: MonadCfgEnv = cfg_env.into();

        // Should now use Monad defaults
        assert_eq!(monad_cfg.max_code_size(), MONAD_MAX_CODE_SIZE);
    }
}
