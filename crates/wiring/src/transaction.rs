use crate::{AccessListItem, Address, AuthorizationList, Bytes, TxKind, B256, GAS_PER_BLOB, U256};

/// Trait for retrieving transaction information required for execution.
pub trait Transaction {
    /// Caller aka Author aka transaction signer.
    fn caller(&self) -> &Address;
    /// The maximum amount of gas the transaction can use.
    fn gas_limit(&self) -> u64;
    /// The gas price the sender is willing to pay.
    fn gas_price(&self) -> &U256;
    /// Returns what kind of transaction this is.
    fn kind(&self) -> TxKind;
    /// The value sent to the receiver of `TxKind::Call`.
    fn value(&self) -> &U256;
    /// Returns the input data of the transaction.
    fn data(&self) -> &Bytes;
    /// The nonce of the transaction.
    fn nonce(&self) -> u64;
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
    fn access_list(&self) -> &[AccessListItem];
    /// The maximum priority fee per gas the sender is willing to pay.
    ///
    /// Incorporated as part of the London upgrade via [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    fn max_priority_fee_per_gas(&self) -> Option<&U256>;
    /// The list of blob versioned hashes. Per EIP there should be at least
    /// one blob present if [`Self::max_fee_per_blob_gas`] is `Some`.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn blob_hashes(&self) -> &[B256];
    /// The maximum fee per blob gas the sender is willing to pay.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn max_fee_per_blob_gas(&self) -> Option<&U256>;
    /// List of authorizations, that contains the signature that authorizes this
    /// caller to place the code to signer account.
    ///
    /// Set EOA account code for one transaction
    ///
    /// [EIP-Set EOA account code for one transaction](https://eips.ethereum.org/EIPS/eip-7702)
    fn authorization_list(&self) -> Option<&AuthorizationList>;

    /// See [EIP-4844], [`crate::Env::calc_data_fee`], and [`crate::Env::calc_max_data_fee`].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn get_total_blob_gas(&self) -> u64 {
        GAS_PER_BLOB * self.blob_hashes().len() as u64
    }
}
