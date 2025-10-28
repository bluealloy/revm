//! This module contains [`CfgEnv`] and implements [`Cfg`] trait for it.
pub use context_interface::Cfg;

use primitives::{eip170, eip3860, eip7825, hardfork::SpecId};
/// EVM configuration
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CfgEnv<SPEC = SpecId> {
    /// Chain ID of the EVM. Used in CHAINID opcode and transaction's chain ID check.
    ///
    /// Chain ID is introduced EIP-155.
    pub chain_id: u64,

    /// Whether to check the transaction's chain ID.
    ///
    /// If set to `false`, the transaction's chain ID check will be skipped.
    pub tx_chain_id_check: bool,

    /// Specification for EVM represent the hardfork
    pub spec: SPEC,
    /// Contract code size limit override.
    ///
    /// If None, the limit will be determined by the SpecId (EIP-170 or EIP-7907) at runtime.
    /// If Some, this specific limit will be used regardless of SpecId.
    ///
    /// Useful to increase this because of tests.
    pub limit_contract_code_size: Option<usize>,
    /// Contract initcode size limit override.
    ///
    /// If None, the limit will check if `limit_contract_code_size` is set.
    /// If it is set, it will double it for a limit.
    /// If it is not set, the limit will be determined by the SpecId (EIP-170 or EIP-7907) at runtime.
    ///
    /// Useful to increase this because of tests.
    pub limit_contract_initcode_size: Option<usize>,
    /// Skips the nonce validation against the account's nonce
    pub disable_nonce_check: bool,
    /// Blob max count. EIP-7840 Add blob schedule to EL config files.
    ///
    /// If this config is not set, the check for max blobs will be skipped.
    pub max_blobs_per_tx: Option<u64>,
    /// Blob base fee update fraction. EIP-4844 Blob base fee update fraction.
    ///
    /// If this config is not set, the blob base fee update fraction will be set to the default value.
    /// See also [CfgEnv::blob_base_fee_update_fraction].
    ///
    /// Default values for Cancun is [`primitives::eip4844::BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN`]
    /// and for Prague is [`primitives::eip4844::BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE`].
    pub blob_base_fee_update_fraction: Option<u64>,
    /// Configures the gas limit cap for the transaction.
    ///
    /// If `None`, default value defined by spec will be used.
    ///
    /// Introduced in Osaka in [EIP-7825: Transaction Gas Limit Cap](https://eips.ethereum.org/EIPS/eip-7825)
    /// with initials cap of 30M.
    pub tx_gas_limit_cap: Option<u64>,
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
    /// EIP-3541 rejects the creation of contracts that starts with 0xEF
    ///
    /// This is useful for chains that do not implement EIP-3541.
    ///
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_eip3541")]
    pub disable_eip3541: bool,
    /// EIP-3607 rejects transactions from senders with deployed code
    ///
    /// In development, it can be desirable to simulate calls from contracts, which this setting allows.
    ///
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_eip3607")]
    pub disable_eip3607: bool,
    /// EIP-7623 increases calldata cost.
    ///
    /// This EIP can be considered irrelevant in the context of an EVM-compatible L2 rollup,
    /// if it does not make use of blobs.
    ///
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_eip7623")]
    pub disable_eip7623: bool,
    /// Disables base fee checks for EIP-1559 transactions
    ///
    /// This is useful for testing method calls with zero gas price.
    ///
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_no_base_fee")]
    pub disable_base_fee: bool,
    /// Disables "max fee must be less than or equal to max priority fee" check for EIP-1559 transactions.
    /// This is useful because some chains (e.g. Arbitrum) do not enforce this check.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_priority_fee_check")]
    pub disable_priority_fee_check: bool,
    /// Disables fee charging for transactions.
    /// This is useful when executing `eth_call` for example, on OP-chains where setting the base fee
    /// to 0 isn't sufficient.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_fee_charge")]
    pub disable_fee_charge: bool,
}

impl CfgEnv {
    /// Creates new `CfgEnv` with default values.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<SPEC: Into<SpecId> + Copy> CfgEnv<SPEC> {
    /// Returns the blob base fee update fraction from [CfgEnv::blob_base_fee_update_fraction].
    ///
    /// If this field is not set, return the default value for the spec.
    ///
    /// Default values for Cancun is [`primitives::eip4844::BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN`]
    /// and for Prague is [`primitives::eip4844::BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE`].
    pub fn blob_base_fee_update_fraction(&mut self) -> u64 {
        self.blob_base_fee_update_fraction.unwrap_or_else(|| {
            let spec: SpecId = self.spec.into();
            if spec.is_enabled_in(SpecId::PRAGUE) {
                primitives::eip4844::BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE
            } else {
                primitives::eip4844::BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN
            }
        })
    }
}

impl<SPEC> CfgEnv<SPEC> {
    /// Create new `CfgEnv` with default values and specified spec.
    pub fn new_with_spec(spec: SPEC) -> Self {
        Self {
            chain_id: 1,
            tx_chain_id_check: true,
            limit_contract_code_size: None,
            limit_contract_initcode_size: None,
            spec,
            disable_nonce_check: false,
            max_blobs_per_tx: None,
            tx_gas_limit_cap: None,
            blob_base_fee_update_fraction: None,
            #[cfg(feature = "memory_limit")]
            memory_limit: (1 << 32) - 1,
            #[cfg(feature = "optional_balance_check")]
            disable_balance_check: false,
            #[cfg(feature = "optional_block_gas_limit")]
            disable_block_gas_limit: false,
            #[cfg(feature = "optional_eip3541")]
            disable_eip3541: false,
            #[cfg(feature = "optional_eip3607")]
            disable_eip3607: false,
            #[cfg(feature = "optional_eip7623")]
            disable_eip7623: false,
            #[cfg(feature = "optional_no_base_fee")]
            disable_base_fee: false,
            #[cfg(feature = "optional_priority_fee_check")]
            disable_priority_fee_check: false,
            #[cfg(feature = "optional_fee_charge")]
            disable_fee_charge: false,
        }
    }

    /// Consumes `self` and returns a new `CfgEnv` with the specified chain ID.
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// Enables the transaction's chain ID check.
    pub fn enable_tx_chain_id_check(mut self) -> Self {
        self.tx_chain_id_check = true;
        self
    }

    /// Disables the transaction's chain ID check.
    pub fn disable_tx_chain_id_check(mut self) -> Self {
        self.tx_chain_id_check = false;
        self
    }

    /// Consumes `self` and returns a new `CfgEnv` with the specified spec.
    pub fn with_spec<OSPEC: Into<SpecId>>(self, spec: OSPEC) -> CfgEnv<OSPEC> {
        CfgEnv {
            chain_id: self.chain_id,
            tx_chain_id_check: self.tx_chain_id_check,
            limit_contract_code_size: self.limit_contract_code_size,
            limit_contract_initcode_size: self.limit_contract_initcode_size,
            spec,
            disable_nonce_check: self.disable_nonce_check,
            tx_gas_limit_cap: self.tx_gas_limit_cap,
            max_blobs_per_tx: self.max_blobs_per_tx,
            blob_base_fee_update_fraction: self.blob_base_fee_update_fraction,
            #[cfg(feature = "memory_limit")]
            memory_limit: self.memory_limit,
            #[cfg(feature = "optional_balance_check")]
            disable_balance_check: self.disable_balance_check,
            #[cfg(feature = "optional_block_gas_limit")]
            disable_block_gas_limit: self.disable_block_gas_limit,
            #[cfg(feature = "optional_eip3541")]
            disable_eip3541: self.disable_eip3541,
            #[cfg(feature = "optional_eip3607")]
            disable_eip3607: self.disable_eip3607,
            #[cfg(feature = "optional_eip7623")]
            disable_eip7623: self.disable_eip7623,
            #[cfg(feature = "optional_no_base_fee")]
            disable_base_fee: self.disable_base_fee,
            #[cfg(feature = "optional_priority_fee_check")]
            disable_priority_fee_check: self.disable_priority_fee_check,
            #[cfg(feature = "optional_fee_charge")]
            disable_fee_charge: self.disable_fee_charge,
        }
    }

    /// Sets the blob target
    pub fn with_max_blobs_per_tx(mut self, max_blobs_per_tx: u64) -> Self {
        self.set_max_blobs_per_tx(max_blobs_per_tx);
        self
    }

    /// Sets the blob target
    pub fn set_max_blobs_per_tx(&mut self, max_blobs_per_tx: u64) {
        self.max_blobs_per_tx = Some(max_blobs_per_tx);
    }

    /// Clears the blob target and max count over hardforks.
    pub fn clear_max_blobs_per_tx(&mut self) {
        self.max_blobs_per_tx = None;
    }

    /// Sets the disable priority fee check flag.
    #[cfg(feature = "optional_priority_fee_check")]
    pub fn with_disable_priority_fee_check(mut self, disable: bool) -> Self {
        self.disable_priority_fee_check = disable;
        self
    }

    /// Sets the disable fee charge flag.
    #[cfg(feature = "optional_fee_charge")]
    pub fn with_disable_fee_charge(mut self, disable: bool) -> Self {
        self.disable_fee_charge = disable;
        self
    }

    /// Sets the disable eip7623 flag.
    #[cfg(feature = "optional_eip7623")]
    pub fn with_disable_eip7623(mut self, disable: bool) -> Self {
        self.disable_eip7623 = disable;
        self
    }
}

impl<SPEC: Into<SpecId> + Copy> Cfg for CfgEnv<SPEC> {
    type Spec = SPEC;

    #[inline]
    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    #[inline]
    fn spec(&self) -> Self::Spec {
        self.spec
    }

    #[inline]
    fn tx_chain_id_check(&self) -> bool {
        self.tx_chain_id_check
    }

    #[inline]
    fn tx_gas_limit_cap(&self) -> u64 {
        self.tx_gas_limit_cap
            .unwrap_or(if self.spec.into().is_enabled_in(SpecId::OSAKA) {
                eip7825::TX_GAS_LIMIT_CAP
            } else {
                u64::MAX
            })
    }

    #[inline]
    fn max_blobs_per_tx(&self) -> Option<u64> {
        self.max_blobs_per_tx
    }

    fn max_code_size(&self) -> usize {
        self.limit_contract_code_size
            .unwrap_or(eip170::MAX_CODE_SIZE)
    }

    fn max_initcode_size(&self) -> usize {
        self.limit_contract_initcode_size
            .or_else(|| {
                self.limit_contract_code_size
                    .map(|size| size.saturating_mul(2))
            })
            .unwrap_or(eip3860::MAX_INITCODE_SIZE)
    }

    fn is_eip3541_disabled(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "optional_eip3541")] {
                self.disable_eip3541
            } else {
                false
            }
        }
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

    fn is_eip7623_disabled(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "optional_eip7623")] {
                self.disable_eip7623
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

    fn is_priority_fee_check_disabled(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "optional_priority_fee_check")] {
                self.disable_priority_fee_check
            } else {
                false
            }
        }
    }

    fn is_fee_charge_disabled(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "optional_fee_charge")] {
                self.disable_fee_charge
            } else {
                false
            }
        }
    }

    fn memory_limit(&self) -> u64 {
        cfg_if::cfg_if! {
            if #[cfg(feature = "memory_limit")] {
                self.memory_limit
            } else {
                u64::MAX
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
        assert_eq!(cfg.max_blobs_per_tx(), None);
    }
}
