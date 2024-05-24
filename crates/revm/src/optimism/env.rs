use crate::{
    primitives::{
        db::Database, Address, BlobExcessGasAndPrice, Block, BlockEnv, Bytes, HashMap, TransactTo,
        Transaction, TxEnv, B256, U256,
    },
    L1BlockInfo,
};

use super::OptimismSpecId;

/// The Optimism block environment.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OptimismBlock {
    pub base: BlockEnv,
    /// L1 block info used to compute the L1-cost fee.
    ///
    /// Needs to be provided for Optimism non-deposit transactions.
    l1_block_info: Option<L1BlockInfo>,
}

impl OptimismBlock {
    /// Constructs a new instance.
    pub fn new(base: BlockEnv, l1_block_info: Option<L1BlockInfo>) -> Self {
        Self {
            base,
            l1_block_info,
        }
    }

    /// Create a new Optimism block environment.
    pub fn with_l1_block_info<DB: Database>(
        base: BlockEnv,
        db: &mut DB,
        spec_id: OptimismSpecId,
    ) -> Result<Self, DB::Error> {
        let l1_block_info = L1BlockInfo::try_fetch(db, spec_id)?;

        Ok(Self {
            base,
            l1_block_info: Some(l1_block_info),
        })
    }

    /// Retrieves the L1 block info.
    pub fn l1_block_info(&self) -> Option<&L1BlockInfo> {
        self.l1_block_info.as_ref()
    }
}

impl Block for OptimismBlock {
    fn number(&self) -> &U256 {
        self.base.number()
    }

    fn coinbase(&self) -> &Address {
        self.base.coinbase()
    }

    fn timestamp(&self) -> &U256 {
        self.base.timestamp()
    }

    fn gas_limit(&self) -> &U256 {
        self.base.gas_limit()
    }

    fn basefee(&self) -> &U256 {
        self.base.basefee()
    }

    fn difficulty(&self) -> &U256 {
        self.base.difficulty()
    }

    fn prevrandao(&self) -> Option<&B256> {
        self.base.prevrandao()
    }

    fn blob_excess_gas_and_price(&self) -> Option<&BlobExcessGasAndPrice> {
        self.base.blob_excess_gas_and_price()
    }
}

/// The Optimism transaction environment.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OptimismTransaction {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub base: TxEnv,

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
    /// This field is optional to allow the [TxEnv] to be constructed
    /// for non-optimism chains when the `optimism` feature is enabled,
    /// but the [CfgEnv] `optimism` field is set to false.
    pub enveloped_tx: Option<Bytes>,
}

impl Transaction for OptimismTransaction {
    fn caller(&self) -> &Address {
        self.base.caller()
    }

    fn gas_limit(&self) -> u64 {
        self.base.gas_limit()
    }

    fn gas_price(&self) -> &U256 {
        self.base.gas_price()
    }

    fn transact_to(&self) -> &TransactTo {
        self.base.transact_to()
    }

    fn value(&self) -> &U256 {
        self.base.value()
    }

    fn data(&self) -> &Bytes {
        self.base.data()
    }

    fn nonce(&self) -> Option<u64> {
        self.base.nonce()
    }

    fn chain_id(&self) -> Option<u64> {
        self.base.chain_id()
    }

    fn access_list(&self) -> &[(Address, Vec<U256>)] {
        self.base.access_list()
    }

    fn gas_priority_fee(&self) -> Option<&U256> {
        self.base.gas_priority_fee()
    }

    fn blob_hashes(&self) -> &[B256] {
        self.base.blob_hashes()
    }

    fn max_fee_per_blob_gas(&self) -> Option<&U256> {
        self.base.max_fee_per_blob_gas()
    }

    fn eof_initcodes(&self) -> &[Bytes] {
        self.base.eof_initcodes()
    }

    fn eof_initcodes_hashed(&self) -> &HashMap<B256, Bytes> {
        self.base.eof_initcodes_hashed()
    }
}
