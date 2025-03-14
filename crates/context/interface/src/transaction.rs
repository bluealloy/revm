mod alloy_types;
pub mod eip2930;
pub mod eip7702;
pub mod transaction_type;

pub use alloy_types::{
    AccessList, AccessListItem, Authorization, RecoveredAuthority, RecoveredAuthorization,
    SignedAuthorization,
};
pub use eip2930::AccessListItemTr;
pub use eip7702::AuthorizationTr;
pub use transaction_type::TransactionType;

use auto_impl::auto_impl;
use core::cmp::min;
use core::fmt::Debug;
use primitives::{eip4844::GAS_PER_BLOB, Address, Bytes, TxKind, B256, U256};

/// Transaction validity error types.
pub trait TransactionError: Debug + core::error::Error {}

/// Main Transaction trait that abstracts and specifies all transaction currently supported by Ethereum
///
/// Access to any associated type is gaited behind [`tx_type`][Transaction::tx_type] function.
///
/// It can be extended to support new transaction types and only transaction types can be
/// deprecated by not returning tx_type.
#[auto_impl(&, Box, Arc, Rc)]
pub trait Transaction {
    type AccessListItem: AccessListItemTr;
    type Authorization: AuthorizationTr;

    /// Returns the transaction type.
    ///
    /// Depending on this field other functions should be called.
    fn tx_type(&self) -> u8;

    /// Caller aka Author aka transaction signer.
    ///
    /// Note : Common field for all transactions.
    fn caller(&self) -> Address;

    /// The maximum amount of gas the transaction can use.
    ///
    /// Note : Common field for all transactions.
    fn gas_limit(&self) -> u64;

    /// The value sent to the receiver of [`TxKind::Call`][primitives::TxKind::Call].
    ///
    /// Note : Common field for all transactions.
    fn value(&self) -> U256;

    /// Returns the input data of the transaction.
    ///
    /// Note : Common field for all transactions.
    fn input(&self) -> &Bytes;

    /// The nonce of the transaction.
    ///
    /// Note : Common field for all transactions.
    fn nonce(&self) -> u64;

    /// Transaction kind. It can be Call or Create.
    ///
    /// Kind is applicable for: Legacy, EIP-2930, EIP-1559
    /// And is Call for EIP-4844 and EIP-7702 transactions.
    fn kind(&self) -> TxKind;

    /// Chain Id is optional for legacy transactions.
    ///
    /// As it was introduced in EIP-155.
    fn chain_id(&self) -> Option<u64>;

    /// Gas price for the transaction.
    /// It is only applicable for Legacy and EIP-2930 transactions.
    /// For Eip1559 it is max_fee_per_gas.
    fn gas_price(&self) -> u128;

    /// Access list for the transaction.
    ///
    /// Introduced in EIP-2930.
    fn access_list(&self) -> Option<impl Iterator<Item = &Self::AccessListItem>>;

    /// Returns vector of fixed size hash(32 bytes)
    ///
    /// Note : EIP-4844 transaction field.
    fn blob_versioned_hashes(&self) -> &[B256];

    /// Max fee per data gas
    ///
    /// Note : EIP-4844 transaction field.
    fn max_fee_per_blob_gas(&self) -> u128;

    /// Total gas for all blobs. Max number of blocks is already checked
    /// so we dont need to check for overflow.
    fn total_blob_gas(&self) -> u64 {
        GAS_PER_BLOB * self.blob_versioned_hashes().len() as u64
    }

    /// Calculates the maximum [EIP-4844] `data_fee` of the transaction.
    ///
    /// This is used for ensuring that the user has at least enough funds to pay the
    /// `max_fee_per_blob_gas * total_blob_gas`, on top of regular gas costs.
    ///
    /// See EIP-4844:
    /// <https://github.com/ethereum/EIPs/blob/master/EIPS/eip-4844.md#execution-layer-validation>
    fn calc_max_data_fee(&self) -> U256 {
        let blob_gas = U256::from(self.total_blob_gas());
        let max_blob_fee = U256::from(self.max_fee_per_blob_gas());
        max_blob_fee.saturating_mul(blob_gas)
    }

    /// Returns length of the authorization list.
    ///
    /// # Note
    ///
    /// Transaction is considered invalid if list is empty.
    fn authorization_list_len(&self) -> usize;

    /// List of authorizations, that contains the signature that authorizes this
    /// caller to place the code to signer account.
    ///
    /// Set EOA account code for one transaction
    ///
    /// [EIP-Set EOA account code for one transaction](https://eips.ethereum.org/EIPS/eip-7702)
    fn authorization_list(&self) -> impl Iterator<Item = &Self::Authorization>;

    /// Returns maximum fee that can be paid for the transaction.
    fn max_fee_per_gas(&self) -> u128 {
        self.gas_price()
    }

    /// Maximum priority fee per gas.
    fn max_priority_fee_per_gas(&self) -> Option<u128>;

    /// Returns effective gas price is gas price field for Legacy and Eip2930 transaction.
    ///
    /// While for transactions after Eip1559 it is minimum of max_fee and `base + max_priority_fee`.
    fn effective_gas_price(&self, base_fee: u128) -> u128 {
        let max_fee = self.gas_price();
        let Some(max_priority_fee) = self.max_priority_fee_per_gas() else {
            return max_fee;
        };
        min(max_fee, base_fee.saturating_add(max_priority_fee))
    }
}

#[auto_impl(&, &mut, Box, Arc)]
pub trait TransactionGetter {
    type Transaction: Transaction;

    fn tx(&self) -> &Self::Transaction;
}
