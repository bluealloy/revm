use revm::primitives::{
    AccessListItem, Address, AuthorizationList, Bytes, Transaction, TransactionValidation, TxKind,
    B256, U256,
};

use super::{OptimismInvalidTransaction, OptimismTransaction};

/// The Optimism transaction environment.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxEnv {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub base: revm::primitives::TxEnv,

    /// The source hash is used to make sure that deposit transactions do
    /// not have identical hashes.
    ///
    /// L1 originated deposit transaction source hashes are computed using
    /// the hash of the l1 block hash and the l1 log index.
    /// L1 attributes deposit source hashes are computed with the l1 block
    /// hash and the sequence number = l2 block number - l2 epoch start
    /// block number.
    ///
    /// These two deposit transaction sources specify a domain in the outer
    /// hash so there are no collisions.
    pub source_hash: Option<B256>,
    /// The amount to increase the balance of the `from` account as part of
    /// a deposit transaction. This is unconditional and is applied to the
    /// `from` account even if the deposit transaction fails since
    /// the deposit is pre-paid on L1.
    pub mint: Option<u128>,
    /// Whether or not the transaction is a system transaction.
    pub is_system_transaction: Option<bool>,
    /// An enveloped EIP-2718 typed transaction. This is used
    /// to compute the L1 tx cost using the L1 block info, as
    /// opposed to requiring downstream apps to compute the cost
    /// externally.
    pub enveloped_tx: Option<Bytes>,
}

impl Transaction for TxEnv {
    fn caller(&self) -> &Address {
        self.base.caller()
    }

    fn gas_limit(&self) -> u64 {
        self.base.gas_limit()
    }

    fn gas_price(&self) -> &U256 {
        self.base.gas_price()
    }

    fn kind(&self) -> TxKind {
        self.base.kind()
    }

    fn value(&self) -> &U256 {
        self.base.value()
    }

    fn data(&self) -> &Bytes {
        self.base.data()
    }

    fn nonce(&self) -> u64 {
        self.base.nonce()
    }

    fn chain_id(&self) -> Option<u64> {
        self.base.chain_id()
    }

    fn access_list(&self) -> &[AccessListItem] {
        self.base.access_list()
    }

    fn max_priority_fee_per_gas(&self) -> Option<&U256> {
        self.base.max_priority_fee_per_gas()
    }

    fn blob_hashes(&self) -> &[B256] {
        self.base.blob_hashes()
    }

    fn max_fee_per_blob_gas(&self) -> Option<&U256> {
        self.base.max_fee_per_blob_gas()
    }

    fn authorization_list(&self) -> Option<&AuthorizationList> {
        self.base.authorization_list()
    }
}

impl OptimismTransaction for TxEnv {
    fn source_hash(&self) -> Option<&B256> {
        self.source_hash.as_ref()
    }

    fn mint(&self) -> Option<&u128> {
        self.mint.as_ref()
    }

    fn is_system_transaction(&self) -> Option<bool> {
        self.is_system_transaction
    }

    fn enveloped_tx(&self) -> Option<Bytes> {
        self.enveloped_tx.clone()
    }
}

impl TransactionValidation for TxEnv {
    type ValidationError = OptimismInvalidTransaction;
}
