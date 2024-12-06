/// Transaction types of all Ethereum transaction.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactionType {
    /// Legacy transaction type.
    Legacy,
    /// EIP-2930 Access List transaction type.
    Eip2930,
    /// EIP-1559 Fee market change transaction type.
    Eip1559,
    /// EIP-4844 Blob transaction type.
    Eip4844,
    /// EIP-7702 Set EOA account code transaction type.
    Eip7702,
    /// Custom type means that transaction trait was extend and have custom types.
    Custom,
}
