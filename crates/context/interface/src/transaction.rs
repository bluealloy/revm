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

use crate::result::InvalidTransaction;
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
    type AccessListItem<'a>: AccessListItemTr
    where
        Self: 'a;
    type Authorization<'a>: AuthorizationTr
    where
        Self: 'a;

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
    fn access_list(&self) -> Option<impl Iterator<Item = Self::AccessListItem<'_>>>;

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
    fn authorization_list(&self) -> impl Iterator<Item = Self::Authorization<'_>>;

    // TODO(EOF)
    // /// List of initcodes found in Initcode transaction. Initcodes can only be accessed
    // /// by TXCREATE opcode to create a new EOF contract.
    // ///
    // /// Each transaction can contain up to [`primitives::eof::MAX_INITCODE_COUNT`] initcodes,
    // /// with each initcode not exceeding [`primitives::MAX_INITCODE_SIZE`] bytes in size.
    // ///
    // /// EIP link: <https://eips.ethereum.org/EIPS/eip-7873>
    // fn initcodes(&self) -> &[Bytes];

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
        if self.tx_type() == TransactionType::Legacy as u8
            || self.tx_type() == TransactionType::Eip2930 as u8
        {
            return self.gas_price();
        }

        // for EIP-1559 tx and onwards gas_price represents maximum price.
        let max_price = self.gas_price();
        let Some(max_priority_fee) = self.max_priority_fee_per_gas() else {
            return max_price;
        };
        min(max_price, base_fee.saturating_add(max_priority_fee))
    }

    /// Returns the maximum balance that can be spent by the transaction.
    ///
    /// Return U256 or error if all values overflow U256 number.
    fn max_balance_spending(&self) -> Result<U256, InvalidTransaction> {
        // gas_limit * max_fee + value + additional_gas_cost
        let mut max_balance_spending = U256::from(self.gas_limit())
            .checked_mul(U256::from(self.max_fee_per_gas()))
            .and_then(|gas_cost| gas_cost.checked_add(self.value()))
            .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;

        // add blob fee
        if self.tx_type() == TransactionType::Eip4844 {
            let data_fee = self.calc_max_data_fee();
            max_balance_spending = max_balance_spending
                .checked_add(data_fee)
                .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;
        }
        Ok(max_balance_spending)
    }

    /// Returns the effective balance that is going to be spent that depends on base_fee
    ///
    /// This is always strictly less than [`Self::max_balance_spending`].
    ///
    /// Return U256 or error if all values overflow U256 number.
    fn effective_balance_spending(
        &self,
        base_fee: u128,
        blob_price: u128,
    ) -> Result<U256, InvalidTransaction> {
        // gas_limit * max_fee + value + additional_gas_cost
        let mut effective_balance_spending = U256::from(self.gas_limit())
            .checked_mul(U256::from(self.effective_gas_price(base_fee)))
            .and_then(|gas_cost| gas_cost.checked_add(self.value()))
            .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;

        // add blob fee
        if self.tx_type() == TransactionType::Eip4844 {
            let blob_gas = self.total_blob_gas() as u128;
            effective_balance_spending = effective_balance_spending
                .checked_add(U256::from(blob_price).saturating_mul(U256::from(blob_gas)))
                .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;
        }

        Ok(effective_balance_spending)
    }
}

#[auto_impl(&, &mut, Box, Arc)]
pub trait TransactionGetter {
    type Transaction: Transaction;

    fn tx(&self) -> &Self::Transaction;
}
