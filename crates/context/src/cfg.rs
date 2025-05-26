//! This module contains [`CfgEnv`] and implements [`Cfg`] trait for it.
pub use context_interface::Cfg;

use primitives::{eip170::MAX_CODE_SIZE, hardfork::SpecId};
use std::{vec, vec::Vec};

/// EVM configuration
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CfgEnv<SPEC = SpecId> {
    /// Chain ID of the EVM
    ///
    /// `chain_id` will be compared to the transaction's Chain ID.
    ///
    /// Chain ID is introduced EIP-155.
    pub chain_id: u64,
    /// Specification for EVM represent the hardfork
    pub spec: SPEC,
    /// If some it will effects EIP-170: Contract code size limit.
    ///
    /// Useful to increase this because of tests.
    ///
    /// By default it is `0x6000` (~25kb).
    pub limit_contract_code_size: Option<usize>,
    /// Skips the nonce validation against the account's nonce
    pub disable_nonce_check: bool,
    /// Blob target count. EIP-7840 Add blob schedule to EL config files.
    ///
    /// Note : Items must be sorted by `SpecId`.
    pub blob_target_and_max_count: Vec<(SpecId, u64, u64)>,
    /// A hard memory limit in bytes beyond which
    /// [OutOfGasError::Memory][context_interface::result::OutOfGasError::Memory] cannot be resized.
    ///
    /// In cases where the gas limit may be extraordinarily high, it is recommended to set this to
    /// a sane value to prevent memory allocation panics.
    ///
    /// Defaults to `2^32 - 1` bytes per EIP-1985.
    #[cfg(feature = "memory_limit")]
    pub memory_limit: u64,
    /// Skip balance checks if `true`
    ///
    /// Adds transaction cost to balance to ensure execution doesn't fail.
    ///
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_balance_check")]
    pub disable_balance_check: bool,
    /// There are use cases where it's allowed to provide a gas limit that's higher than a block's gas limit.
    ///
    /// To that end, you can disable the block gas limit validation.
    ///
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_block_gas_limit")]
    pub disable_block_gas_limit: bool,
    /// EIP-3607 rejects transactions from senders with deployed code
    ///
    /// In development, it can be desirable to simulate calls from contracts, which this setting allows.
    ///
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_eip3607")]
    pub disable_eip3607: bool,
    /// Disables base fee checks for EIP-1559 transactions
    ///
    /// This is useful for testing method calls with zero gas price.
    ///
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_no_base_fee")]
    pub disable_base_fee: bool,
}

impl CfgEnv {
    /// Creates new `CfgEnv` with default values.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<SPEC> CfgEnv<SPEC> {
    /// Create new `CfgEnv` with default values and specified spec.
    pub fn new_with_spec(spec: SPEC) -> Self {
        Self {
            chain_id: 1,
            limit_contract_code_size: None,
            spec,
            disable_nonce_check: false,
            blob_target_and_max_count: vec![(SpecId::CANCUN, 3, 6), (SpecId::PRAGUE, 6, 9)],
            #[cfg(feature = "memory_limit")]
            memory_limit: (1 << 32) - 1,
            #[cfg(feature = "optional_balance_check")]
            disable_balance_check: false,
            #[cfg(feature = "optional_block_gas_limit")]
            disable_block_gas_limit: false,
            #[cfg(feature = "optional_eip3607")]
            disable_eip3607: false,
            #[cfg(feature = "optional_no_base_fee")]
            disable_base_fee: false,
        }
    }

    /// Consumes `self` and returns a new `CfgEnv` with the specified chain ID.
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// Consumes `self` and returns a new `CfgEnv` with the specified spec.
    pub fn with_spec<OSPEC: Into<SpecId>>(self, spec: OSPEC) -> CfgEnv<OSPEC> {
        CfgEnv {
            chain_id: self.chain_id,
            limit_contract_code_size: self.limit_contract_code_size,
            spec,
            disable_nonce_check: self.disable_nonce_check,
            blob_target_and_max_count: self.blob_target_and_max_count,
            #[cfg(feature = "memory_limit")]
            memory_limit: self.memory_limit,
            #[cfg(feature = "optional_balance_check")]
            disable_balance_check: self.disable_balance_check,
            #[cfg(feature = "optional_block_gas_limit")]
            disable_block_gas_limit: self.disable_block_gas_limit,
            #[cfg(feature = "optional_eip3607")]
            disable_eip3607: self.disable_eip3607,
            #[cfg(feature = "optional_no_base_fee")]
            disable_base_fee: self.disable_base_fee,
        }
    }

    /// Sets the blob target and max count over hardforks.
    pub fn with_blob_max_and_target_count(mut self, blob_params: Vec<(SpecId, u64, u64)>) -> Self {
        self.set_blob_max_and_target_count(blob_params);
        self
    }

    /// Sets the blob target and max count over hardforks.
    pub fn set_blob_max_and_target_count(&mut self, mut blob_params: Vec<(SpecId, u64, u64)>) {
        blob_params.sort_by_key(|(id, _, _)| *id);
        self.blob_target_and_max_count = blob_params;
    }
}

impl<SPEC: Into<SpecId> + Copy> Cfg for CfgEnv<SPEC> {
    type Spec = SPEC;

    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    fn spec(&self) -> Self::Spec {
        self.spec
    }

    #[inline]
    fn blob_max_count(&self, spec_id: SpecId) -> u64 {
        self.blob_target_and_max_count
            .iter()
            .rev()
            .find_map(|(id, _, max)| {
                if spec_id as u8 >= *id as u8 {
                    return Some(*max);
                }
                None
            })
            .unwrap_or(6)
    }

    fn max_code_size(&self) -> usize {
        self.limit_contract_code_size.unwrap_or(MAX_CODE_SIZE)
    }

    fn is_eip3607_disabled(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "optional_eip3607")] {
                self.disable_eip3607
            } else {
                false
            }
        }
    }

    fn is_balance_check_disabled(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "optional_balance_check")] {
                self.disable_balance_check
            } else {
                false
            }
        }
    }

    /// Returns `true` if the block gas limit is disabled.
    fn is_block_gas_limit_disabled(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "optional_block_gas_limit")] {
                self.disable_block_gas_limit
            } else {
                false
            }
        }
    }

    fn is_nonce_check_disabled(&self) -> bool {
        self.disable_nonce_check
    }

    fn is_base_fee_check_disabled(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "optional_no_base_fee")] {
                self.disable_base_fee
            } else {
                false
            }
        }
    }
}

impl<SPEC: Default> Default for CfgEnv<SPEC> {
    fn default() -> Self {
        Self::new_with_spec(SPEC::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn blob_max_and_target_count() {
        let cfg: CfgEnv = Default::default();
        assert_eq!(cfg.blob_max_count(SpecId::BERLIN), (6));
        assert_eq!(cfg.blob_max_count(SpecId::CANCUN), (6));
        assert_eq!(cfg.blob_max_count(SpecId::PRAGUE), (9));
        assert_eq!(cfg.blob_max_count(SpecId::OSAKA), (9));
    }
}
