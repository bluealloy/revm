use crate::CommonTxFields;
use primitives::TxKind;

/// Legacy transaction trait before introduction of EIP-2929
pub trait LegacyTx: CommonTxFields {
    /// Transaction kind.
    fn kind(&self) -> TxKind;

    /// Chain Id is optional for legacy transactions
    /// As it was introduced in EIP-155.
    fn chain_id(&self) -> Option<u64>;

    /// Gas price for the transaction.
    fn gas_price(&self) -> u128;
}
