use crate::{Address, Bytes, HashMap, TransactTo, B256, GAS_PER_BLOB, U256};

/// Trait for retrieving transaction information required for execution.
pub trait Transaction {
    /// Caller aka Author aka transaction signer.
    fn caller(&self) -> &Address;
    /// The gas limit of the transaction.
    fn gas_limit(&self) -> u64;
    /// The gas price of the transaction.
    fn gas_price(&self) -> &U256;
    /// The destination of the transaction.
    fn transact_to(&self) -> &TransactTo;
    /// The value sent to `transact_to`.
    fn value(&self) -> &U256;
    /// The data of the transaction.
    fn data(&self) -> &Bytes;
    /// The nonce of the transaction.
    ///
    /// Caution: If set to `None`, then nonce validation against the account's nonce is skipped: [InvalidTransaction::NonceTooHigh] and [InvalidTransaction::NonceTooLow]
    fn nonce(&self) -> Option<u64>;
    /// The chain ID of the transaction. If set to `None`, no checks are performed.
    ///
    /// Incorporated as part of the Spurious Dragon upgrade via [EIP-155].
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    fn chain_id(&self) -> Option<u64>;
    /// A list of addresses and storage keys that the transaction plans to access.
    ///
    /// Added in [EIP-2930].
    ///
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    fn access_list(&self) -> &[(Address, Vec<U256>)];
    /// The priority fee per gas.
    ///
    /// Incorporated as part of the London upgrade via [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    fn gas_priority_fee(&self) -> Option<&U256>;
    /// The list of blob versioned hashes. Per EIP there should be at least
    /// one blob present if [`Self::max_fee_per_blob_gas`] is `Some`.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn blob_hashes(&self) -> &[B256];
    /// The max fee per blob gas.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn max_fee_per_blob_gas(&self) -> Option<&U256>;
    /// EOF Initcodes for EOF CREATE transaction
    ///
    /// Incorporated as part of the Prague upgrade via [EOF]
    ///
    /// [EOF]: https://eips.ethereum.org/EIPS/eip-4844
    fn eof_initcodes(&self) -> &[Bytes];
    /// Internal Temporary field that stores the hashes of the EOF initcodes.
    ///
    /// Those are always cleared after the transaction is executed.
    /// And calculated/overwritten every time transaction starts.
    /// They are calculated from the [`Self::eof_initcodes`] field.
    fn eof_initcodes_hashed(&self) -> &HashMap<B256, Bytes>;
}

/// See [EIP-4844], [`Env::calc_data_fee`], and [`Env::calc_max_data_fee`].
///
/// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
#[inline]
pub fn get_total_blob_gas(transaction: &impl Transaction) -> u64 {
    GAS_PER_BLOB * transaction.blob_hashes().len() as u64
}
