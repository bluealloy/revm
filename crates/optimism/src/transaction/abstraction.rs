use super::deposit::{DepositTransaction, TxDeposit};
use crate::OpTransactionError;
use revm::{
    context::TxEnv,
    context_interface::{
        transaction::{CommonTxFields, Transaction, TransactionType},
        Journal, TransactionGetter,
    },
    primitives::Bytes,
    Context, Database,
};

pub trait OpTxTrait: Transaction {
    type DepositTx: DepositTransaction;

    fn deposit(&self) -> &Self::DepositTx;

    fn enveloped_tx(&self) -> Option<&Bytes>;
}

pub trait OpTxGetter: TransactionGetter {
    type OpTransaction: OpTxTrait;

    fn op_tx(&self) -> &Self::OpTransaction;
}

impl<BLOCK, TX: Transaction, CFG, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> OpTxGetter
    for Context<BLOCK, OpTransaction<TX>, CFG, DB, JOURNAL, CHAIN>
{
    type OpTransaction = OpTransaction<TX>;

    fn op_tx(&self) -> &Self::OpTransaction {
        &self.tx
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OpTransactionType {
    /// Base transaction type supported on Ethereum mainnet.
    Base(TransactionType),
    /// Optimism-specific deposit transaction type.
    Deposit,
}

impl From<OpTransactionType> for TransactionType {
    fn from(tx_type: OpTransactionType) -> Self {
        match tx_type {
            OpTransactionType::Base(tx_type) => tx_type,
            OpTransactionType::Deposit => TransactionType::Custom,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OpTransaction<T: Transaction> {
    Base {
        tx: T,
        /// An enveloped EIP-2718 typed transaction
        ///
        /// This is used to compute the L1 tx cost using the L1 block info, as
        /// opposed to requiring downstream apps to compute the cost
        /// externally.
        enveloped_tx: Option<Bytes>,
    },
    Deposit(TxDeposit),
}

impl Default for OpTransaction<TxEnv> {
    fn default() -> Self {
        Self::Base {
            tx: TxEnv::default(),
            enveloped_tx: None,
        }
    }
}

impl<T: Transaction> Transaction for OpTransaction<T> {
    // TODO
    type TransactionError = OpTransactionError;
    type TransactionType = OpTransactionType;

    type AccessList = T::AccessList;

    type Legacy = T::Legacy;

    type Eip2930 = T::Eip2930;

    type Eip1559 = T::Eip1559;

    type Eip4844 = T::Eip4844;

    type Eip7702 = T::Eip7702;

    fn tx_type(&self) -> Self::TransactionType {
        match self {
            Self::Base { tx, .. } => OpTransactionType::Base(tx.tx_type().into()),
            Self::Deposit(_) => OpTransactionType::Deposit,
        }
    }

    fn kind(&self) -> revm::primitives::TxKind {
        match self {
            Self::Base { tx, .. } => tx.kind(),
            Self::Deposit(deposit) => deposit.to,
        }
    }

    fn effective_gas_price(&self, base_fee: u128) -> u128 {
        match self {
            Self::Base { tx, .. } => tx.effective_gas_price(base_fee),
            Self::Deposit(_) => base_fee,
        }
    }

    fn max_fee(&self) -> u128 {
        match self {
            Self::Base { tx, .. } => tx.max_fee(),
            Self::Deposit(_) => 0,
        }
    }

    fn legacy(&self) -> &Self::Legacy {
        let Self::Base { tx, .. } = self else {
            panic!("Not a legacy transaction")
        };
        tx.legacy()
    }

    fn eip2930(&self) -> &Self::Eip2930 {
        let Self::Base { tx, .. } = self else {
            panic!("Not eip2930 transaction")
        };
        tx.eip2930()
    }

    fn eip1559(&self) -> &Self::Eip1559 {
        let Self::Base { tx, .. } = self else {
            panic!("Not a eip1559 transaction")
        };
        tx.eip1559()
    }

    fn eip4844(&self) -> &Self::Eip4844 {
        let Self::Base { tx, .. } = self else {
            panic!("Not a eip4844 transaction")
        };
        tx.eip4844()
    }

    fn eip7702(&self) -> &Self::Eip7702 {
        let Self::Base { tx, .. } = self else {
            panic!("Not a eip7702 transaction")
        };
        tx.eip7702()
    }
}

impl<T: Transaction> OpTxTrait for OpTransaction<T> {
    type DepositTx = TxDeposit;

    fn deposit(&self) -> &Self::DepositTx {
        match self {
            Self::Base { .. } => panic!("Not a deposit transaction"),
            Self::Deposit(deposit) => deposit,
        }
    }

    fn enveloped_tx(&self) -> Option<&Bytes> {
        match self {
            Self::Base { enveloped_tx, .. } => enveloped_tx.as_ref(),
            Self::Deposit(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm::primitives::{Address, B256, U256};

    #[test]
    fn test_deposit_transaction_type_conversion() {
        let deposit_tx = OpTransactionType::Deposit;
        let tx_type: TransactionType = deposit_tx.into();
        assert_eq!(tx_type, TransactionType::Custom);

        // Also test base transaction conversion
        let base_tx = OpTransactionType::Base(TransactionType::Legacy);
        let tx_type: TransactionType = base_tx.into();
        assert_eq!(tx_type, TransactionType::Legacy);
    }

    #[test]
    fn test_deposit_transaction_fields() {
        let deposit = TxDeposit {
            from: Address::ZERO,
            to: revm::primitives::TxKind::Call(Address::ZERO),
            value: U256::ZERO,
            gas_limit: 0,
            is_system_transaction: false,
            mint: Some(0u128),
            source_hash: B256::default(),
            input: Default::default(),
        };
        let op_tx: OpTransaction<TxEnv> = OpTransaction::Deposit(deposit);
        // Verify transaction type
        assert_eq!(op_tx.tx_type(), OpTransactionType::Deposit);
        // Verify common fields access
        assert_eq!(op_tx.gas_limit(), 0);
        assert_eq!(op_tx.kind(), revm::primitives::TxKind::Call(Address::ZERO));
        // Verify gas related calculations
        assert_eq!(op_tx.effective_gas_price(100), 100);
        assert_eq!(op_tx.max_fee(), 0);
    }
}
