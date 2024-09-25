use super::eip2930::AccessListInterface;
use crate::CommonTxFields;
use primitives::TxKind;

pub trait Eip1559Tx: Eip1559CommonTxFields {
    fn kind(&self) -> TxKind;
}

/// This trait is base for Eip1559, EIp4844 and Eip7702 transactions.
pub trait Eip1559CommonTxFields: CommonTxFields {
    type AccessList: AccessListInterface;

    fn chain_id(&self) -> u64;

    fn max_fee_per_gas(&self) -> u128;

    fn max_priority_fee_per_gas(&self) -> u128;

    fn access_list(&self) -> &Self::AccessList;
}
