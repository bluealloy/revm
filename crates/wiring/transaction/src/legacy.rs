use crate::CommonTxFields;
use primitives::TxKind;

pub trait LegacyTx: CommonTxFields {
    fn kind(&self) -> TxKind;
}
