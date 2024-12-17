use auto_impl::auto_impl;
use core::fmt::Debug;
use core::hash::Hash;
use primitives::{TxKind, U256};
use specification::hardfork::SpecId;

#[auto_impl(&, &mut, Box, Arc)]
pub trait Cfg {
    type Spec: Into<SpecId>;

    fn chain_id(&self) -> u64;

    // TODO Make SpecId a associated type but for faster development we use impl Into.
    fn spec(&self) -> Self::Spec;

    fn max_code_size(&self) -> usize;

    fn is_eip3607_disabled(&self) -> bool;

    fn is_balance_check_disabled(&self) -> bool;

    fn is_gas_refund_disabled(&self) -> bool;

    fn is_block_gas_limit_disabled(&self) -> bool;

    fn is_nonce_check_disabled(&self) -> bool;

    fn is_base_fee_check_disabled(&self) -> bool;
}

/// What bytecode analysis to perform.
#[derive(Clone, Default, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnalysisKind {
    /// Do not perform bytecode analysis.
    Raw,
    /// Perform bytecode analysis.
    #[default]
    Analyse,
}

/// Transaction destination
pub type TransactTo = TxKind;

/// Create scheme.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CreateScheme {
    /// Legacy create scheme of `CREATE`.
    Create,
    /// Create scheme of `CREATE2`.
    Create2 {
        /// Salt.
        salt: U256,
    },
}

#[auto_impl(&, &mut, Box, Arc)]
pub trait CfgGetter {
    type Cfg: Cfg;

    fn cfg(&self) -> &Self::Cfg;
}
