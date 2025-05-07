use super::deposit::{DepositTransactionParts, DEPOSIT_TRANSACTION_TYPE};
use auto_impl::auto_impl;
use revm::{
    context::TxEnv,
    context_interface::transaction::Transaction,
    handler::SystemCallTx,
    primitives::{Address, Bytes, TxKind, B256, U256},
};
use std::vec;

#[auto_impl(&, &mut, Box, Arc)]
pub trait OpTxTr: Transaction {
    fn enveloped_tx(&self) -> Option<&Bytes>;

    /// Source hash of the deposit transaction
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

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpTransaction<T: Transaction> {
    pub base: T,
    /// An enveloped EIP-2718 typed transaction
    ///
    /// This is used to compute the L1 tx cost using the L1 block info, as
    /// opposed to requiring downstream apps to compute the cost
    /// externally.
    pub enveloped_tx: Option<Bytes>,
    pub deposit: DepositTransactionParts,
}

impl<T: Transaction> OpTransaction<T> {
    pub fn new(base: T) -> Self {
        Self {
            base,
            enveloped_tx: None,
            deposit: DepositTransactionParts::default(),
        }
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
    fn new_system_tx(data: Bytes, system_contract_address: Address) -> Self {
        OpTransaction::new(TX::new_system_tx(data, system_contract_address))
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
        self.base.tx_type()
    }

    fn caller(&self) -> Address {
        self.base.caller()
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
        self.base.effective_gas_price(base_fee)
    }

    fn authorization_list_len(&self) -> usize {
        self.base.authorization_list_len()
    }

    fn authorization_list(&self) -> impl Iterator<Item = Self::Authorization<'_>> {
        self.base.authorization_list()
    }

    // TODO(EOF)
    // fn initcodes(&self) -> &[Bytes] {
    //     self.base.initcodes()
    // }
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

#[cfg(test)]
mod tests {
    use crate::transaction::deposit::DEPOSIT_TRANSACTION_TYPE;

    use super::*;
    use revm::primitives::{Address, B256};

    #[test]
    fn test_deposit_transaction_fields() {
        let op_tx = OpTransaction {
            base: TxEnv {
                tx_type: DEPOSIT_TRANSACTION_TYPE,
                gas_limit: 10,
                gas_price: 100,
                gas_priority_fee: Some(5),
                ..Default::default()
            },
            enveloped_tx: None,
            deposit: DepositTransactionParts {
                is_system_transaction: false,
                mint: Some(0u128),
                source_hash: B256::default(),
            },
        };
        // Verify transaction type
        assert_eq!(op_tx.tx_type(), DEPOSIT_TRANSACTION_TYPE);
        // Verify common fields access
        assert_eq!(op_tx.gas_limit(), 10);
        assert_eq!(op_tx.kind(), revm::primitives::TxKind::Call(Address::ZERO));
        // Verify gas related calculations
        assert_eq!(op_tx.effective_gas_price(90), 95);
        assert_eq!(op_tx.max_fee_per_gas(), 100);
    }
}
