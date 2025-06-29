//! Optimism transaction abstraction containing the `[OpTxTr]` trait and corresponding `[OpTransaction]` type.
use super::deposit::{DepositTransactionParts, DEPOSIT_TRANSACTION_TYPE};
use auto_impl::auto_impl;
use revm::{
    context::{
        tx::{TxEnvBuildError, TxEnvBuilder},
        TxEnv,
    },
    context_interface::transaction::Transaction,
    handler::SystemCallTx,
    primitives::{Address, Bytes, TxKind, B256, U256},
};
use std::vec;

/// Optimism Transaction trait.
#[auto_impl(&, &mut, Box, Arc)]
pub trait OpTxTr: Transaction {
    /// Enveloped transaction bytes.
    fn enveloped_tx(&self) -> Option<&Bytes>;

    /// Source hash of the deposit transaction.
    fn source_hash(&self) -> Option<B256>;

    /// Mint of the deposit transaction
    fn mint(&self) -> Option<u128>;

    /// Whether the transaction is a system transaction
    fn is_system_transaction(&self) -> bool;

    /// Returns `true` if transaction is of type [`DEPOSIT_TRANSACTION_TYPE`].
    fn is_deposit(&self) -> bool {
        self.tx_type() == DEPOSIT_TRANSACTION_TYPE
    }
}

/// Optimism transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpTransaction<T: Transaction> {
    /// Base transaction fields.
    pub(crate) base: T,
    /// An enveloped EIP-2718 typed transaction
    ///
    /// This is used to compute the L1 tx cost using the L1 block info, as
    /// opposed to requiring downstream apps to compute the cost
    /// externally.
    pub(crate) enveloped_tx: Option<Bytes>,
    /// Deposit transaction parts.
    pub(crate) deposit: DepositTransactionParts,
}

impl<T: Transaction> AsRef<T> for OpTransaction<T> {
    fn as_ref(&self) -> &T {
        &self.base
    }
}

impl<T: Transaction> OpTransaction<T> {
    /// Create a new Optimism transaction.
    pub fn new(base: T) -> Self {
        Self {
            base,
            enveloped_tx: None,
            deposit: DepositTransactionParts::default(),
        }
    }
}

impl OpTransaction<TxEnv> {
    /// Create a new Optimism transaction.
    pub fn builder() -> OpTransactionBuilder {
        OpTransactionBuilder::new()
    }
}

impl Default for OpTransaction<TxEnv> {
    fn default() -> Self {
        Self {
            base: TxEnv::default(),
            enveloped_tx: Some(vec![0x00].into()),
            deposit: DepositTransactionParts::default(),
        }
    }
}

impl<TX: Transaction + SystemCallTx> SystemCallTx for OpTransaction<TX> {
    fn new_system_tx_with_caller(
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Self {
        OpTransaction::new(TX::new_system_tx_with_caller(
            caller,
            system_contract_address,
            data,
        ))
    }
}

impl<T: Transaction> Transaction for OpTransaction<T> {
    type AccessListItem<'a>
        = T::AccessListItem<'a>
    where
        T: 'a;
    type Authorization<'a>
        = T::Authorization<'a>
    where
        T: 'a;

    fn tx_type(&self) -> u8 {
        // If this is a deposit transaction (has source_hash set), return deposit type
        if self.deposit.source_hash != B256::ZERO {
            DEPOSIT_TRANSACTION_TYPE
        } else {
            self.base.tx_type()
        }
    }

    fn caller(&self) -> Address {
        self.base.caller()
    }

    fn caller_u256(&self) -> U256 {
        self.base.caller_u256()
    }

    fn gas_limit(&self) -> u64 {
        self.base.gas_limit()
    }

    fn value(&self) -> U256 {
        self.base.value()
    }

    fn input(&self) -> &Bytes {
        self.base.input()
    }

    fn nonce(&self) -> u64 {
        self.base.nonce()
    }

    fn kind(&self) -> TxKind {
        self.base.kind()
    }

    fn chain_id(&self) -> Option<u64> {
        self.base.chain_id()
    }

    fn access_list(&self) -> Option<impl Iterator<Item = Self::AccessListItem<'_>>> {
        self.base.access_list()
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.base.max_priority_fee_per_gas()
    }

    fn max_fee_per_gas(&self) -> u128 {
        self.base.max_fee_per_gas()
    }

    fn gas_price(&self) -> u128 {
        self.base.gas_price()
    }

    fn blob_versioned_hashes(&self) -> &[B256] {
        self.base.blob_versioned_hashes()
    }

    fn max_fee_per_blob_gas(&self) -> u128 {
        self.base.max_fee_per_blob_gas()
    }

    fn effective_gas_price(&self, base_fee: u128) -> u128 {
        // Deposit transactions use gas_price directly
        if self.tx_type() == DEPOSIT_TRANSACTION_TYPE {
            return self.gas_price();
        }
        self.base.effective_gas_price(base_fee)
    }

    fn authorization_list_len(&self) -> usize {
        self.base.authorization_list_len()
    }

    fn authorization_list(&self) -> impl Iterator<Item = Self::Authorization<'_>> {
        self.base.authorization_list()
    }
}

impl<T: Transaction> OpTxTr for OpTransaction<T> {
    fn enveloped_tx(&self) -> Option<&Bytes> {
        self.enveloped_tx.as_ref()
    }

    fn source_hash(&self) -> Option<B256> {
        if self.tx_type() != DEPOSIT_TRANSACTION_TYPE {
            return None;
        }
        Some(self.deposit.source_hash)
    }

    fn mint(&self) -> Option<u128> {
        self.deposit.mint
    }

    fn is_system_transaction(&self) -> bool {
        self.deposit.is_system_transaction
    }
}

/// Builder for constructing [`OpTransaction`] instances
#[derive(Default, Debug)]
pub struct OpTransactionBuilder {
    base: TxEnvBuilder,
    enveloped_tx: Option<Bytes>,
    deposit: DepositTransactionParts,
}

impl OpTransactionBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            base: TxEnvBuilder::new(),
            enveloped_tx: None,
            deposit: DepositTransactionParts::default(),
        }
    }

    /// Set the base transaction builder based for TxEnvBuilder.
    pub fn base(mut self, base: TxEnvBuilder) -> Self {
        self.base = base;
        self
    }

    /// Set the enveloped transaction bytes.
    pub fn enveloped_tx(mut self, enveloped_tx: Option<Bytes>) -> Self {
        self.enveloped_tx = enveloped_tx;
        self
    }

    /// Set the source hash of the deposit transaction.
    pub fn source_hash(mut self, source_hash: B256) -> Self {
        self.deposit.source_hash = source_hash;
        self
    }

    /// Set the mint of the deposit transaction.
    pub fn mint(mut self, mint: u128) -> Self {
        self.deposit.mint = Some(mint);
        self
    }

    /// Set the deposit transaction to be a system transaction.
    pub fn is_system_transaction(mut self) -> Self {
        self.deposit.is_system_transaction = true;
        self
    }

    /// Set the deposit transaction to not be a system transaction.
    pub fn not_system_transaction(mut self) -> Self {
        self.deposit.is_system_transaction = false;
        self
    }

    /// Set the deposit transaction to be a deposit transaction.
    pub fn is_deposit_tx(mut self) -> Self {
        self.base = self.base.tx_type(Some(DEPOSIT_TRANSACTION_TYPE));
        self
    }

    /// Build the [`OpTransaction`] with default values for missing fields.
    ///
    /// This is useful for testing and debugging where it is not necessary to
    /// have full [`OpTransaction`] instance.
    pub fn build_fill(mut self) -> OpTransaction<TxEnv> {
        let base = self.base.build_fill();
        if base.tx_type() != DEPOSIT_TRANSACTION_TYPE {
            self.enveloped_tx = Some(vec![0x00].into());
        }

        OpTransaction {
            base,
            enveloped_tx: self.enveloped_tx,
            deposit: self.deposit,
        }
    }

    /// Build the [`OpTransaction`] instance, return error if the transaction is not valid.
    ///
    pub fn build(self) -> Result<OpTransaction<TxEnv>, OpBuildError> {
        let base = self.base.build()?;
        // Check if this will be a deposit transaction based on source_hash
        let is_deposit = self.deposit.source_hash != B256::ZERO;
        if !is_deposit && self.enveloped_tx.is_none() {
            return Err(OpBuildError::MissingEnvelopedTxBytes);
        }

        Ok(OpTransaction {
            base,
            enveloped_tx: self.enveloped_tx,
            deposit: self.deposit,
        })
    }
}

/// Error type for building [`TxEnv`]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OpBuildError {
    /// Base transaction build error
    Base(TxEnvBuildError),
    /// Missing enveloped transaction bytes
    MissingEnvelopedTxBytes,
}

impl From<TxEnvBuildError> for OpBuildError {
    fn from(error: TxEnvBuildError) -> Self {
        OpBuildError::Base(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm::{
        context_interface::Transaction,
        primitives::{Address, B256},
    };

    #[test]
    fn test_deposit_transaction_fields() {
        let base_tx = TxEnv::builder()
            .gas_limit(10)
            .gas_price(100)
            .gas_priority_fee(Some(5))
            .build()
            .unwrap();

        let op_tx = OpTransaction::builder()
            .base(base_tx.modify())
            .enveloped_tx(None)
            .not_system_transaction()
            .mint(0u128)
            .source_hash(B256::from([1u8; 32]))
            .build()
            .unwrap();
        // Verify transaction type (deposit transactions should have tx_type based on OpSpecId)
        // The tx_type is derived from the transaction structure, not set manually
        // Verify common fields access
        assert_eq!(op_tx.gas_limit(), 10);
        assert_eq!(op_tx.kind(), revm::primitives::TxKind::Call(Address::ZERO));
        // Verify gas related calculations - deposit transactions use gas_price for effective gas price
        assert_eq!(op_tx.effective_gas_price(90), 100);
        assert_eq!(op_tx.max_fee_per_gas(), 100);
    }
}
