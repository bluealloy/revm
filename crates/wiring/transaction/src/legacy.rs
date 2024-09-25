use crate::CommonTxFields;
use primitives::TxKind;

pub trait LegacyTx: CommonTxFields {
    /// Legacy transaction kind
    fn kind(&self) -> TxKind;

    /// Chain Id is optional for legacy transactions
    fn chain_id(&self) -> Option<u64>;

    /// Gas price for the transaction
    fn gas_price(&self) -> u128;
}
