use auto_impl::auto_impl;
use core::fmt::Debug;
use core::hash::Hash;
use primitives::{hardfork::SpecId, Address, TxKind, U256};

#[auto_impl(&, &mut, Box, Arc)]
pub trait Cfg {
    type Spec: Into<SpecId> + Clone;

    fn chain_id(&self) -> u64;

    /// Returns the gas limit cap for the transaction.
    ///
    /// Cap is introduced in [`EIP-7825: Transaction Gas Limit Cap`](https://eips.ethereum.org/EIPS/eip-7825)
    /// with initial cap of 30M gas.
    ///
    /// Value before EIP-7825 is `u64::MAX`.
    fn tx_gas_limit_cap(&self) -> u64;

    /// Specification id
    fn spec(&self) -> Self::Spec;

    /// Returns the maximum number of blobs allowed per transaction.
    /// If it is None, check for max count will be skipped.
    fn max_blobs_per_tx(&self) -> Option<u64>;

    fn max_code_size(&self) -> usize;

    fn is_eip3607_disabled(&self) -> bool;

    fn is_balance_check_disabled(&self) -> bool;

    fn is_block_gas_limit_disabled(&self) -> bool;

    fn is_nonce_check_disabled(&self) -> bool;

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
    Custom { address: Address },
}
