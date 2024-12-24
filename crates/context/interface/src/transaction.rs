mod access_list;
mod common;
pub mod eip1559;
pub mod eip2930;
pub mod eip4844;
pub mod eip7702;
pub mod legacy;
pub mod transaction_type;

pub use access_list::AccessListTrait;
pub use common::CommonTxFields;
pub use eip1559::{Eip1559CommonTxFields, Eip1559Tx};
pub use eip2930::Eip2930Tx;
pub use eip4844::Eip4844Tx;
pub use eip7702::Eip7702Tx;
pub use legacy::LegacyTx;
pub use transaction_type::TransactionType;

use auto_impl::auto_impl;
use core::cmp::min;
use core::fmt::Debug;
use primitives::{TxKind, U256};
use std::boxed::Box;

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
    /// An error that occurs when validating a transaction
    type TransactionError: TransactionError;
    /// Transaction type
    type TransactionType: Into<TransactionType>;
    /// Access list type
    type AccessList: AccessListTrait;

    type Legacy: LegacyTx;
    type Eip2930: Eip2930Tx<AccessList = Self::AccessList>;
    type Eip1559: Eip1559Tx<AccessList = Self::AccessList>;
    type Eip4844: Eip4844Tx<AccessList = Self::AccessList>;
    type Eip7702: Eip7702Tx<AccessList = Self::AccessList>;

    /// Returns the transaction type.
    ///
    /// Depending on this field other functions should be called.
    ///
    /// If transaction is Legacy, then [`legacy()`][Transaction::legacy] should be called.
    fn tx_type(&self) -> Self::TransactionType;

    /// Returns the legacy transaction.
    fn legacy(&self) -> &Self::Legacy {
        unimplemented!("legacy tx not supported")
    }

    /// Returns EIP-2930 transaction.
    fn eip2930(&self) -> &Self::Eip2930 {
        unimplemented!("Eip2930 tx not supported")
    }

    /// Returns EIP-1559 transaction.
    fn eip1559(&self) -> &Self::Eip1559 {
        unimplemented!("Eip1559 tx not supported")
    }

    /// Returns EIP-4844 transaction.
    fn eip4844(&self) -> &Self::Eip4844 {
        unimplemented!("Eip4844 tx not supported")
    }

    /// Returns EIP-7702 transaction.
    fn eip7702(&self) -> &Self::Eip7702 {
        unimplemented!("Eip7702 tx not supported")
    }

    /// Returns common fields for all transactions.
    fn common_fields(&self) -> &dyn CommonTxFields {
        match self.tx_type().into() {
            TransactionType::Legacy => self.legacy(),
            TransactionType::Eip2930 => self.eip2930(),
            TransactionType::Eip1559 => self.eip1559(),
            TransactionType::Eip4844 => self.eip4844(),
            TransactionType::Eip7702 => self.eip7702(),
            TransactionType::Custom => unimplemented!("Custom tx not supported"),
        }
    }

    /// Returns maximum fee that can be paid for the transaction.
    fn max_fee(&self) -> u128 {
        match self.tx_type().into() {
            TransactionType::Legacy => self.legacy().gas_price(),
            TransactionType::Eip2930 => self.eip2930().gas_price(),
            TransactionType::Eip1559 => self.eip1559().max_fee_per_gas(),
            TransactionType::Eip4844 => self.eip4844().max_fee_per_gas(),
            TransactionType::Eip7702 => self.eip7702().max_fee_per_gas(),
            TransactionType::Custom => unimplemented!("Custom tx not supported"),
        }
    }

    /// Returns effective gas price is gas price field for Legacy and Eip2930 transaction.
    ///
    /// While for transactions after Eip1559 it is minimum of max_fee and `base + max_priority_fee`.
    fn effective_gas_price(&self, base_fee: U256) -> U256 {
        let tx_type = self.tx_type().into();
        let (max_fee, max_priority_fee) = match tx_type {
            TransactionType::Legacy => return U256::from(self.legacy().gas_price()),
            TransactionType::Eip2930 => return U256::from(self.eip2930().gas_price()),
            TransactionType::Eip1559 => (
                self.eip1559().max_fee_per_gas(),
                self.eip1559().max_priority_fee_per_gas(),
            ),
            TransactionType::Eip4844 => (
                self.eip4844().max_fee_per_gas(),
                self.eip4844().max_priority_fee_per_gas(),
            ),
            TransactionType::Eip7702 => (
                self.eip7702().max_fee_per_gas(),
                self.eip7702().max_priority_fee_per_gas(),
            ),
            TransactionType::Custom => unimplemented!("Custom tx not supported"),
        };

        min(U256::from(max_fee), base_fee + U256::from(max_priority_fee))
    }

    /// Returns transaction kind.
    fn kind(&self) -> TxKind {
        let tx_type = self.tx_type().into();
        match tx_type {
            TransactionType::Legacy => self.legacy().kind(),
            TransactionType::Eip2930 => self.eip2930().kind(),
            TransactionType::Eip1559 => self.eip1559().kind(),
            TransactionType::Eip4844 => TxKind::Call(self.eip4844().destination()),
            TransactionType::Eip7702 => TxKind::Call(self.eip7702().destination()),
            TransactionType::Custom => unimplemented!("Custom tx not supported"),
        }
    }

    /// Returns access list.
    fn access_list(&self) -> Option<&Self::AccessList> {
        let tx_type = self.tx_type().into();
        match tx_type {
            TransactionType::Legacy => None,
            TransactionType::Eip2930 => Some(self.eip2930().access_list()),
            TransactionType::Eip1559 => Some(self.eip1559().access_list()),
            TransactionType::Eip4844 => Some(self.eip4844().access_list()),
            TransactionType::Eip7702 => Some(self.eip7702().access_list()),
            TransactionType::Custom => unimplemented!("Custom tx not supported"),
        }
    }
}

#[auto_impl(&, &mut, Box, Arc)]
pub trait TransactionGetter {
    type Transaction: Transaction;

    fn tx(&self) -> &Self::Transaction;
}

pub trait TransactionSetter: TransactionGetter {
    fn set_tx(&mut self, tx: <Self as TransactionGetter>::Transaction);
}

impl<T: TransactionSetter> TransactionSetter for &mut T {
    fn set_tx(&mut self, block: <Self as TransactionGetter>::Transaction) {
        (**self).set_tx(block)
    }
}

impl<T: TransactionSetter> TransactionSetter for Box<T> {
    fn set_tx(&mut self, block: <Self as TransactionGetter>::Transaction) {
        (**self).set_tx(block)
    }
}
