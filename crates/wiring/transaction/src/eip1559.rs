use crate::CommonTxFields;

pub trait Eip1559Tx: CommonTxFields {
    fn effective_gas() -> u64;
}
