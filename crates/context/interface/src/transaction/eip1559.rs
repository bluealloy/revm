use super::{AccessListTrait, CommonTxFields};
use auto_impl::auto_impl;
use primitives::TxKind;

#[auto_impl(&, Box, Arc, Rc)]
pub trait Eip1559Tx: Eip1559CommonTxFields {
    fn kind(&self) -> TxKind;
}

/// This trait is base for Eip1559, EIp4844 and Eip7702 transactions.
#[auto_impl(&, Box, Arc, Rc)]
pub trait Eip1559CommonTxFields: CommonTxFields + AccessListTrait {
    /// Access list type.
    type AccessList: AccessListTrait;

    /// Chain id became mandatory in all transaction after EIP-2930.
    fn chain_id(&self) -> u64;

    /// Maximum fee per gas.
    fn max_fee_per_gas(&self) -> u128;

    /// Maximum priority fee per gas.
    fn max_priority_fee_per_gas(&self) -> u128;
}
