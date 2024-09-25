use crate::TransactionType;
use primitives::{Address, Bytes, U256};

/// Trait that contains all common field that are shared by all transactions.
/// This trait is base for Legacy, EIp2930 and Eip1559 transactions.
pub trait CommonTxFields {
    /// Transaction type;
    fn transaction_type(&self) -> TransactionType;
    /// Caller aka Author aka transaction signer.
    fn caller(&self) -> &Address;
    /// The maximum amount of gas the transaction can use.
    fn gas_limit(&self) -> u64;
    /// The value sent to the receiver of `TxKind::Call`.
    fn value(&self) -> &U256;
    /// Returns the input data of the transaction.
    fn input(&self) -> &Bytes;
    /// The nonce of the transaction.
    fn nonce(&self) -> u64;
    /// The chain ID of the transaction. If set to `None`, no checks are performed.
    ///
    /// Incorporated as part of the Spurious Dragon upgrade via [EIP-155].
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    fn chain_id(&self) -> Option<u64>;
}
