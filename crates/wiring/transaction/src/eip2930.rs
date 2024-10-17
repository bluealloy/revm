use crate::{AccessListTrait, CommonTxFields};
use primitives::TxKind;

/// EIP-2930: Optional access lists
pub trait Eip2930Tx: CommonTxFields {
    type AccessList: AccessListTrait;

    /// The chain ID of the chain the transaction is intended for.
    fn chain_id(&self) -> u64;

    /// The gas price of the transaction.
    fn gas_price(&self) -> u128;

    /// The kind of transaction.
    fn kind(&self) -> TxKind;

    /// The access list of the transaction.
    fn access_list(&self) -> &Self::AccessList;
}
