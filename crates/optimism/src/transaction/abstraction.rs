use super::deposit::{DepositTransaction, TxDeposit};
use crate::OpTransactionError;
use revm::{
    primitives::Bytes,
    transaction::{CommonTxFields, Transaction, TransactionType},
    wiring::default::TxEnv,
};

pub trait OpTxTrait: Transaction {
    type DepositTx: DepositTransaction;

    fn deposit(&self) -> &Self::DepositTx;

    fn enveloped_tx(&self) -> Option<&Bytes>;
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
        /// An enveloped EIP-2718 typed transaction. This is used
        /// to compute the L1 tx cost using the L1 block info, as
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

    fn common_fields(&self) -> &dyn CommonTxFields {
        match self {
            Self::Base { tx, .. } => tx.common_fields(),
            Self::Deposit(deposit) => deposit,
        }
    }

    fn kind(&self) -> revm::primitives::TxKind {
        match self {
            Self::Base { tx, .. } => tx.kind(),
            Self::Deposit(deposit) => deposit.to,
        }
    }

    fn effective_gas_price(&self, base_fee: revm::primitives::U256) -> revm::primitives::U256 {
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
