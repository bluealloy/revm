use crate::{
    eip1559::Eip1559CommonTxFields, CommonTxFields, Eip1559Tx, Eip2930Tx, Eip4844Tx, Eip7702Tx,
    LegacyTx, TransactionType,
};
use core::cmp::min;
use core::fmt::Debug;
use primitives::U256;

/// Transaction validity error type.
pub trait TransactionError: Debug + core::error::Error {}

/// Main Transaction trait that abstracts and specifies all transaction currently supported by Ethereum.
///
/// Access to any associated type is gaited behind `tx_type` function.
///
/// It can be extended to support new transaction types and only transaction types can be
/// deprecated by not returning tx_type.
pub trait Transaction {
    /// An error that occurs when validating a transaction.
    type TransactionError: TransactionError;

    type Legacy: LegacyTx;
    type Eip1559: Eip1559Tx;
    type Eip2930: Eip2930Tx;
    type Eip4844: Eip4844Tx;
    type Eip7702: Eip7702Tx;

    /// Transaction type. Depending on this field other functions should be called.
    /// If transaction is Legacy, then `legacy()` should be called.
    fn tx_type(&self) -> impl Into<TransactionType>;

    fn legacy(&self) -> &Self::Legacy {
        unimplemented!("legacy tx not supported")
    }

    fn eip2930(&self) -> &Self::Eip2930 {
        unimplemented!("Eip2930 tx not supported")
    }

    fn eip1559(&self) -> &Self::Eip1559 {
        unimplemented!("Eip1559 tx not supported")
    }

    fn eip4844(&self) -> &Self::Eip4844 {
        unimplemented!("Eip4844 tx not supported")
    }

    fn eip7702(&self) -> &Self::Eip7702 {
        unimplemented!("Eip7702 tx not supported")
    }

    fn common_fields(&self) -> &dyn CommonTxFields {
        match self.tx_type().into() {
            TransactionType::Legacy => self.legacy(),
            TransactionType::Eip2930 => self.eip2930(),
            TransactionType::Eip1559 => self.eip1559(),
            TransactionType::Eip4844 => self.eip4844(),
            TransactionType::Eip7702 => self.eip7702(),
        }
    }

    /// Effective gas price is gas price field for Legacy and Eip2930 transaction
    /// While for transactions after Eip1559 it is minimum of max_fee and base+max_priority_fee.
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
        };

        min(U256::from(max_fee), base_fee + U256::from(max_priority_fee))
    }
}
