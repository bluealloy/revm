//! Configuration for the EVM. Containing [`SpecId`].
use auto_impl::auto_impl;
use core::fmt::Debug;
use core::hash::Hash;
use primitives::{hardfork::SpecId, Address, TxKind, U256};

/// Configuration for the EVM.
#[auto_impl(&, &mut, Box, Arc)]
pub trait Cfg {
    /// Specification id type, in requires to be convertible to `SpecId` so it can be used
    /// by default in mainnet.
    type Spec: Into<SpecId> + Clone;

    /// Returns the chain ID of the EVM that is compared with the transaction's chain ID.
    fn chain_id(&self) -> u64;

    /// Returns whether the transaction's chain ID check is enabled.
    fn tx_chain_id_check(&self) -> bool;

    /// Specification id
    fn spec(&self) -> Self::Spec;

    /// Returns the blob target and max count for the given spec id.
    /// If it is None, check for max count will be skipped.
    ///
    /// EIP-7840: Add blob schedule to execution client configuration files
    fn blob_max_count(&self) -> Option<u64>;

    /// Returns the maximum code size for the given spec id.
    fn max_code_size(&self) -> usize;

    /// Returns whether the EIP-3607 (account clearing) is disabled.
    fn is_eip3607_disabled(&self) -> bool;

    /// Returns whether the balance check is disabled.
    fn is_balance_check_disabled(&self) -> bool;

    /// Returns whether the block gas limit check is disabled.
    fn is_block_gas_limit_disabled(&self) -> bool;

    /// Returns whether the nonce check is disabled.
    fn is_nonce_check_disabled(&self) -> bool;

    /// Returns whether the base fee check is disabled.
    fn is_base_fee_check_disabled(&self) -> bool;
}

/// What bytecode analysis to perform
#[derive(Clone, Default, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnalysisKind {
    /// Do not perform bytecode analysis
    Raw,
    /// Perform bytecode analysis
    #[default]
    Analyse,
}

/// Transaction destination
pub type TransactTo = TxKind;

/// Create scheme
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CreateScheme {
    /// Legacy create scheme of `CREATE`
    Create,
    /// Create scheme of `CREATE2`
    Create2 {
        /// Salt
        salt: U256,
    },
    /// Custom scheme where we set up the original address
    Custom {
        /// Custom contract creation address.
        address: Address,
    },
}
