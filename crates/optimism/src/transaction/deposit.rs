use revm::primitives::B256;

pub const DEPOSIT_TRANSACTION_TYPE: u8 = 0x7E;

pub trait DepositTransaction {
    fn source_hash(&self) -> B256;

    fn mint(&self) -> Option<u128>;

    fn is_system_transaction(&self) -> bool;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DepositTransactionParts {
    pub source_hash: B256,
    pub mint: Option<u128>,
    pub is_system_transaction: bool,
}

impl DepositTransactionParts {
    pub fn new(source_hash: B256, mint: Option<u128>, is_system_transaction: bool) -> Self {
        Self {
            source_hash,
            mint,
            is_system_transaction,
        }
    }
}
