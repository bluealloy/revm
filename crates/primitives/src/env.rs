use crate::{
    calc_blob_gasprice, AccessListItem, Account, Address, AuthorizationList, Block, Bytes,
    EvmWiring, InvalidHeader, InvalidTransaction, Spec, SpecId, Transaction, TransactionValidation,
    B256, MAX_BLOB_NUMBER_PER_BLOCK, MAX_CODE_SIZE, MAX_INITCODE_SIZE, U256,
    VERSIONED_HASH_VERSION_KZG,
};
use alloy_primitives::TxKind;
use core::cmp::{min, Ordering};
use core::fmt::Debug;
use core::hash::Hash;
use std::boxed::Box;
use std::vec::Vec;

/// Subtype
pub type EnvWiring<EvmWiringT> =
    Env<<EvmWiringT as EvmWiring>::Block, <EvmWiringT as EvmWiring>::Transaction>;

#[derive(Clone, Debug, Default)]
/// EVM environment configuration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Env<BlockT: Block, TxT: Transaction> {
    /// Configuration of the EVM itself.
    pub cfg: CfgEnv,
    /// Configuration of the block the transaction is in.
    pub block: BlockT,
    /// Configuration of the transaction that is being executed.
    pub tx: TxT,
}

impl<BlockT: Block, TxT: Transaction> Env<BlockT, TxT> {
    /// Create boxed [Env].
    #[inline]
    pub fn boxed(cfg: CfgEnv, block: BlockT, tx: TxT) -> Box<Self> {
        Box::new(Self { cfg, block, tx })
    }

    /// Calculates the effective gas price of the transaction.
    #[inline]
    pub fn effective_gas_price(&self) -> U256 {
        let gas_price = self.tx.gas_price();
        if let Some(priority_fee) = self.tx.max_priority_fee_per_gas() {
            min(*gas_price, self.block.basefee() + priority_fee)
        } else {
            *gas_price
        }
    }

    /// Calculates the [EIP-4844] `data_fee` of the transaction.
    ///
    /// Returns `None` if `Cancun` is not enabled. This is enforced in [`Env::validate_block_env`].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    #[inline]
    pub fn calc_data_fee(&self) -> Option<U256> {
        self.block.get_blob_gasprice().map(|blob_gas_price| {
            U256::from(*blob_gas_price).saturating_mul(U256::from(self.tx.get_total_blob_gas()))
        })
    }

    /// Calculates the maximum [EIP-4844] `data_fee` of the transaction.
    ///
    /// This is used for ensuring that the user has at least enough funds to pay the
    /// `max_fee_per_blob_gas * total_blob_gas`, on top of regular gas costs.
    ///
    /// See EIP-4844:
    /// <https://github.com/ethereum/EIPs/blob/master/EIPS/eip-4844.md#execution-layer-validation>
    pub fn calc_max_data_fee(&self) -> Option<U256> {
        self.tx.max_fee_per_blob_gas().map(|max_fee_per_blob_gas| {
            max_fee_per_blob_gas.saturating_mul(U256::from(self.tx.get_total_blob_gas()))
        })
    }

    /// Validate the block environment.
    #[inline]
    pub fn validate_block_env<SPEC: Spec>(&self) -> Result<(), InvalidHeader> {
        // `prevrandao` is required for the merge
        if SPEC::enabled(SpecId::MERGE) && self.block.prevrandao().is_none() {
            return Err(InvalidHeader::PrevrandaoNotSet);
        }
        // `excess_blob_gas` is required for Cancun
        if SPEC::enabled(SpecId::CANCUN) && self.block.blob_excess_gas_and_price().is_none() {
            return Err(InvalidHeader::ExcessBlobGasNotSet);
        }
        Ok(())
    }

    /// Validate transaction data that is set inside ENV and return error if something is wrong.
    ///
    /// Return initial spend gas (Gas needed to execute transaction).
    #[inline]
    pub fn validate_tx<SPEC: Spec>(&self) -> Result<(), InvalidTransaction> {
        // Check if the transaction's chain id is correct
        if let Some(tx_chain_id) = self.tx.chain_id() {
            if tx_chain_id != self.cfg.chain_id {
                return Err(InvalidTransaction::InvalidChainId);
            }
        }

        // Check if gas_limit is more than block_gas_limit
        if !self.cfg.is_block_gas_limit_disabled()
            && U256::from(self.tx.gas_limit()) > *self.block.gas_limit()
        {
            return Err(InvalidTransaction::CallerGasLimitMoreThanBlock);
        }

        // Check that access list is empty for transactions before BERLIN
        if !SPEC::enabled(SpecId::BERLIN) && !self.tx.access_list().is_empty() {
            return Err(InvalidTransaction::AccessListNotSupported);
        }

        // BASEFEE tx check
        if SPEC::enabled(SpecId::LONDON) {
            if let Some(priority_fee) = self.tx.max_priority_fee_per_gas() {
                if priority_fee > self.tx.gas_price() {
                    // or gas_max_fee for eip1559
                    return Err(InvalidTransaction::PriorityFeeGreaterThanMaxFee);
                }
            }

            // check minimal cost against basefee
            if !self.cfg.is_base_fee_check_disabled()
                && self.effective_gas_price() < *self.block.basefee()
            {
                return Err(InvalidTransaction::GasPriceLessThanBasefee);
            }
        }

        // EIP-3860: Limit and meter initcode
        if SPEC::enabled(SpecId::SHANGHAI) && self.tx.kind().is_create() {
            let max_initcode_size = self
                .cfg
                .limit_contract_code_size
                .map(|limit| limit.saturating_mul(2))
                .unwrap_or(MAX_INITCODE_SIZE);
            if self.tx.data().len() > max_initcode_size {
                return Err(InvalidTransaction::CreateInitCodeSizeLimit);
            }
        }

        // - For before CANCUN, check that `blob_hashes` and `max_fee_per_blob_gas` are empty / not set
        if !SPEC::enabled(SpecId::CANCUN)
            && (self.tx.max_fee_per_blob_gas().is_some() || !self.tx.blob_hashes().is_empty())
        {
            return Err(InvalidTransaction::BlobVersionedHashesNotSupported);
        }

        // Presence of max_fee_per_blob_gas means that this is blob transaction.
        if let Some(max) = self.tx.max_fee_per_blob_gas() {
            // ensure that the user was willing to at least pay the current blob gasprice
            let price = self.block.get_blob_gasprice().expect("already checked");
            if U256::from(*price) > *max {
                return Err(InvalidTransaction::BlobGasPriceGreaterThanMax);
            }

            // there must be at least one blob
            if self.tx.blob_hashes().is_empty() {
                return Err(InvalidTransaction::EmptyBlobs);
            }

            // The field `to` deviates slightly from the semantics with the exception
            // that it MUST NOT be nil and therefore must always represent
            // a 20-byte address. This means that blob transactions cannot
            // have the form of a create transaction.
            if self.tx.kind().is_create() {
                return Err(InvalidTransaction::BlobCreateTransaction);
            }

            // all versioned blob hashes must start with VERSIONED_HASH_VERSION_KZG
            for blob in self.tx.blob_hashes() {
                if blob[0] != VERSIONED_HASH_VERSION_KZG {
                    return Err(InvalidTransaction::BlobVersionNotSupported);
                }
            }

            // ensure the total blob gas spent is at most equal to the limit
            // assert blob_gas_used <= MAX_BLOB_GAS_PER_BLOCK
            let num_blobs = self.tx.blob_hashes().len();
            if num_blobs > MAX_BLOB_NUMBER_PER_BLOCK as usize {
                return Err(InvalidTransaction::TooManyBlobs {
                    have: num_blobs,
                    max: MAX_BLOB_NUMBER_PER_BLOCK as usize,
                });
            }
        } else {
            // if max_fee_per_blob_gas is not set, then blob_hashes must be empty
            if !self.tx.blob_hashes().is_empty() {
                return Err(InvalidTransaction::BlobVersionedHashesNotSupported);
            }
        }

        // check if EIP-7702 transaction is enabled.
        if !SPEC::enabled(SpecId::PRAGUE) && self.tx.authorization_list().is_some() {
            return Err(InvalidTransaction::AuthorizationListNotSupported);
        }

        if let Some(auth_list) = &self.tx.authorization_list() {
            // The transaction is considered invalid if the length of authorization_list is zero.
            if auth_list.is_empty() {
                return Err(InvalidTransaction::EmptyAuthorizationList);
            }

            // Check validity of authorization_list
            auth_list.is_valid(self.cfg.chain_id)?;

            // Check if other fields are unset.
            if self.tx.max_fee_per_blob_gas().is_some() || !self.tx.blob_hashes().is_empty() {
                return Err(InvalidTransaction::AuthorizationListInvalidFields);
            }
        }

        Ok(())
    }

    /// Validate transaction against state.
    ///
    /// # Panics
    ///
    /// If account code is not loaded.
    #[inline]
    pub fn validate_tx_against_state<SPEC: Spec>(
        &self,
        account: &mut Account,
    ) -> Result<(), InvalidTransaction> {
        // EIP-3607: Reject transactions from senders with deployed code
        // This EIP is introduced after london but there was no collision in past
        // so we can leave it enabled always
        if !self.cfg.is_eip3607_disabled() {
            let bytecode = &account.info.code.as_ref().unwrap();
            // allow EOAs whose code is a valid delegation designation,
            // i.e. 0xef0100 || address, to continue to originate transactions.
            if !bytecode.is_empty() && !bytecode.is_eip7702() {
                return Err(InvalidTransaction::RejectCallerWithCode);
            }
        }

        // Check that the transaction's nonce is correct
        if !self.cfg.is_nonce_check_disabled() {
            let tx = self.tx.nonce();
            let state = account.info.nonce;
            match tx.cmp(&state) {
                Ordering::Greater => {
                    return Err(InvalidTransaction::NonceTooHigh { tx, state });
                }
                Ordering::Less => {
                    return Err(InvalidTransaction::NonceTooLow { tx, state });
                }
                _ => {}
            }
        }

        let mut balance_check = U256::from(self.tx.gas_limit())
            .checked_mul(*self.tx.gas_price())
            .and_then(|gas_cost| gas_cost.checked_add(*self.tx.value()))
            .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;

        if SPEC::enabled(SpecId::CANCUN) {
            // if the tx is not a blob tx, this will be None, so we add zero
            let data_fee = self.calc_max_data_fee().unwrap_or_default();
            balance_check = balance_check
                .checked_add(U256::from(data_fee))
                .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;
        }

        // Check if account has enough balance for gas_limit*gas_price and value transfer.
        // Transfer will be done inside `*_inner` functions.
        if balance_check > account.info.balance {
            if self.cfg.is_balance_check_disabled() {
                // Add transaction cost to balance to ensure execution doesn't fail.
                account.info.balance = balance_check;
            } else {
                return Err(InvalidTransaction::LackOfFundForMaxFee {
                    fee: Box::new(balance_check),
                    balance: Box::new(account.info.balance),
                });
            }
        }

        Ok(())
    }
}

impl<BlockT: Block + Default, TxT: Transaction + Default> Env<BlockT, TxT> {
    /// Resets environment to default values.
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// EVM configuration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CfgEnv {
    /// Chain ID of the EVM, it will be compared to the transaction's Chain ID.
    /// Chain ID is introduced EIP-155
    pub chain_id: u64,
    /// KZG Settings for point evaluation precompile. By default, this is loaded from the ethereum mainnet trusted setup.
    #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub kzg_settings: crate::kzg::EnvKzgSettings,
    /// Bytecode that is created with CREATE/CREATE2 is by default analysed and jumptable is created.
    /// This is very beneficial for testing and speeds up execution of that bytecode if called multiple times.
    ///
    /// Default: Analyse
    pub perf_analyse_created_bytecodes: AnalysisKind,
    /// If some it will effects EIP-170: Contract code size limit. Useful to increase this because of tests.
    /// By default it is 0x6000 (~25kb).
    pub limit_contract_code_size: Option<usize>,
    /// Skips the nonce validation against the account's nonce:
    /// [`crate::InvalidTransaction::NonceTooHigh`] and
    /// [`crate::InvalidTransaction::NonceTooLow`]
    pub disable_nonce_check: bool,
    /// A hard memory limit in bytes beyond which [crate::result::OutOfGasError::Memory] cannot be resized.
    ///
    /// In cases where the gas limit may be extraordinarily high, it is recommended to set this to
    /// a sane value to prevent memory allocation panics. Defaults to `2^32 - 1` bytes per
    /// EIP-1985.
    #[cfg(feature = "memory_limit")]
    pub memory_limit: u64,
    /// Skip balance checks if true. Adds transaction cost to balance to ensure execution doesn't fail.
    #[cfg(feature = "optional_balance_check")]
    pub disable_balance_check: bool,
    /// There are use cases where it's allowed to provide a gas limit that's higher than a block's gas limit. To that
    /// end, you can disable the block gas limit validation.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_block_gas_limit")]
    pub disable_block_gas_limit: bool,
    /// EIP-3607 rejects transactions from senders with deployed code. In development, it can be desirable to simulate
    /// calls from contracts, which this setting allows.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_eip3607")]
    pub disable_eip3607: bool,
    /// Disables all gas refunds. This is useful when using chains that have gas refunds disabled e.g. Avalanche.
    /// Reasoning behind removing gas refunds can be found in EIP-3298.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_gas_refund")]
    pub disable_gas_refund: bool,
    /// Disables base fee checks for EIP-1559 transactions.
    /// This is useful for testing method calls with zero gas price.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_no_base_fee")]
    pub disable_base_fee: bool,
    /// Disables the payout of the reward to the beneficiary.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_beneficiary_reward")]
    pub disable_beneficiary_reward: bool,
}

impl CfgEnv {
    /// Returns max code size from [`Self::limit_contract_code_size`] if set
    /// or default [`MAX_CODE_SIZE`] value.
    pub fn max_code_size(&self) -> usize {
        self.limit_contract_code_size.unwrap_or(MAX_CODE_SIZE)
    }

    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }

    #[cfg(feature = "optional_eip3607")]
    pub fn is_eip3607_disabled(&self) -> bool {
        self.disable_eip3607
    }

    #[cfg(not(feature = "optional_eip3607"))]
    pub fn is_eip3607_disabled(&self) -> bool {
        false
    }

    #[cfg(feature = "optional_balance_check")]
    pub fn is_balance_check_disabled(&self) -> bool {
        self.disable_balance_check
    }

    #[cfg(not(feature = "optional_balance_check"))]
    pub fn is_balance_check_disabled(&self) -> bool {
        false
    }

    #[cfg(feature = "optional_gas_refund")]
    pub fn is_gas_refund_disabled(&self) -> bool {
        self.disable_gas_refund
    }

    #[cfg(not(feature = "optional_gas_refund"))]
    pub fn is_gas_refund_disabled(&self) -> bool {
        false
    }

    #[cfg(feature = "optional_no_base_fee")]
    pub fn is_base_fee_check_disabled(&self) -> bool {
        self.disable_base_fee
    }

    #[cfg(not(feature = "optional_no_base_fee"))]
    pub fn is_base_fee_check_disabled(&self) -> bool {
        false
    }

    #[cfg(feature = "optional_block_gas_limit")]
    pub fn is_block_gas_limit_disabled(&self) -> bool {
        self.disable_block_gas_limit
    }

    #[cfg(not(feature = "optional_block_gas_limit"))]
    pub fn is_block_gas_limit_disabled(&self) -> bool {
        false
    }

    #[cfg(feature = "optional_beneficiary_reward")]
    pub fn is_beneficiary_reward_disabled(&self) -> bool {
        self.disable_beneficiary_reward
    }

    #[cfg(not(feature = "optional_beneficiary_reward"))]
    pub fn is_beneficiary_reward_disabled(&self) -> bool {
        false
    }

    pub const fn is_nonce_check_disabled(&self) -> bool {
        self.disable_nonce_check
    }
}

impl Default for CfgEnv {
    fn default() -> Self {
        Self {
            chain_id: 1,
            perf_analyse_created_bytecodes: AnalysisKind::default(),
            limit_contract_code_size: None,
            disable_nonce_check: false,
            #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
            kzg_settings: crate::kzg::EnvKzgSettings::Default,
            #[cfg(feature = "memory_limit")]
            memory_limit: (1 << 32) - 1,
            #[cfg(feature = "optional_balance_check")]
            disable_balance_check: false,
            #[cfg(feature = "optional_block_gas_limit")]
            disable_block_gas_limit: false,
            #[cfg(feature = "optional_eip3607")]
            disable_eip3607: false,
            #[cfg(feature = "optional_gas_refund")]
            disable_gas_refund: false,
            #[cfg(feature = "optional_no_base_fee")]
            disable_base_fee: false,
            #[cfg(feature = "optional_beneficiary_reward")]
            disable_beneficiary_reward: false,
        }
    }
}

/// The block environment.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockEnv {
    /// The number of ancestor blocks of this block (block height).
    pub number: U256,
    /// Coinbase or miner or address that created and signed the block.
    ///
    /// This is the receiver address of all the gas spent in the block.
    pub coinbase: Address,

    /// The timestamp of the block in seconds since the UNIX epoch.
    pub timestamp: U256,
    /// The gas limit of the block.
    pub gas_limit: U256,
    /// The base fee per gas, added in the London upgrade with [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    pub basefee: U256,
    /// The difficulty of the block.
    ///
    /// Unused after the Paris (AKA the merge) upgrade, and replaced by `prevrandao`.
    pub difficulty: U256,
    /// The output of the randomness beacon provided by the beacon chain.
    ///
    /// Replaces `difficulty` after the Paris (AKA the merge) upgrade with [EIP-4399].
    ///
    /// NOTE: `prevrandao` can be found in a block in place of `mix_hash`.
    ///
    /// [EIP-4399]: https://eips.ethereum.org/EIPS/eip-4399
    pub prevrandao: Option<B256>,
    /// Excess blob gas and blob gasprice.
    /// See also [`crate::calc_excess_blob_gas`]
    /// and [`calc_blob_gasprice`].
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    pub blob_excess_gas_and_price: Option<BlobExcessGasAndPrice>,
}

impl BlockEnv {
    /// Takes `blob_excess_gas` saves it inside env
    /// and calculates `blob_fee` with [`BlobExcessGasAndPrice`].
    pub fn set_blob_excess_gas_and_price(&mut self, excess_blob_gas: u64) {
        self.blob_excess_gas_and_price = Some(BlobExcessGasAndPrice::new(excess_blob_gas));
    }
}

impl Block for BlockEnv {
    #[inline]
    fn number(&self) -> &U256 {
        &self.number
    }

    #[inline]
    fn coinbase(&self) -> &Address {
        &self.coinbase
    }

    #[inline]
    fn timestamp(&self) -> &U256 {
        &self.timestamp
    }

    #[inline]
    fn gas_limit(&self) -> &U256 {
        &self.gas_limit
    }

    #[inline]
    fn basefee(&self) -> &U256 {
        &self.basefee
    }

    #[inline]
    fn difficulty(&self) -> &U256 {
        &self.difficulty
    }

    #[inline]
    fn prevrandao(&self) -> Option<&B256> {
        self.prevrandao.as_ref()
    }

    #[inline]
    fn blob_excess_gas_and_price(&self) -> Option<&BlobExcessGasAndPrice> {
        self.blob_excess_gas_and_price.as_ref()
    }
}

impl Default for BlockEnv {
    fn default() -> Self {
        Self {
            number: U256::ZERO,
            coinbase: Address::ZERO,
            timestamp: U256::from(1),
            gas_limit: U256::MAX,
            basefee: U256::ZERO,
            difficulty: U256::ZERO,
            prevrandao: Some(B256::ZERO),
            blob_excess_gas_and_price: Some(BlobExcessGasAndPrice::new(0)),
        }
    }
}

/// The transaction environment.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxEnv {
    /// Caller aka Author aka transaction signer.
    pub caller: Address,
    /// The gas limit of the transaction.
    pub gas_limit: u64,
    /// The gas price of the transaction.
    pub gas_price: U256,
    /// The destination of the transaction.
    pub transact_to: TxKind,
    /// The value sent to `transact_to`.
    pub value: U256,
    /// The data of the transaction.
    pub data: Bytes,

    /// The nonce of the transaction.
    pub nonce: u64,

    /// The chain ID of the transaction. If set to `None`, no checks are performed.
    ///
    /// Incorporated as part of the Spurious Dragon upgrade via [EIP-155].
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    pub chain_id: Option<u64>,

    /// A list of addresses and storage keys that the transaction plans to access.
    ///
    /// Added in [EIP-2930].
    ///
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    pub access_list: Vec<AccessListItem>,

    /// The priority fee per gas.
    ///
    /// Incorporated as part of the London upgrade via [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    pub gas_priority_fee: Option<U256>,

    /// The list of blob versioned hashes. Per EIP there should be at least
    /// one blob present if [`Self::max_fee_per_blob_gas`] is `Some`.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    pub blob_hashes: Vec<B256>,

    /// The max fee per blob gas.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    pub max_fee_per_blob_gas: Option<U256>,

    /// List of authorizations, that contains the signature that authorizes this
    /// caller to place the code to signer account.
    ///
    /// Set EOA account code for one transaction
    ///
    /// [EIP-Set EOA account code for one transaction](https://eips.ethereum.org/EIPS/eip-7702)
    pub authorization_list: Option<AuthorizationList>,
}

impl Transaction for TxEnv {
    #[inline]
    fn caller(&self) -> &Address {
        &self.caller
    }

    #[inline]
    fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    #[inline]
    fn gas_price(&self) -> &U256 {
        &self.gas_price
    }

    #[inline]
    fn kind(&self) -> TxKind {
        self.transact_to
    }

    #[inline]
    fn value(&self) -> &U256 {
        &self.value
    }

    #[inline]
    fn data(&self) -> &Bytes {
        &self.data
    }

    #[inline]
    fn nonce(&self) -> u64 {
        self.nonce
    }

    #[inline]
    fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    #[inline]
    fn access_list(&self) -> &[AccessListItem] {
        &self.access_list
    }

    #[inline]
    fn max_priority_fee_per_gas(&self) -> Option<&U256> {
        self.gas_priority_fee.as_ref()
    }

    #[inline]
    fn blob_hashes(&self) -> &[B256] {
        &self.blob_hashes
    }

    #[inline]
    fn max_fee_per_blob_gas(&self) -> Option<&U256> {
        self.max_fee_per_blob_gas.as_ref()
    }

    #[inline]
    fn authorization_list(&self) -> Option<&AuthorizationList> {
        self.authorization_list.as_ref()
    }
}

impl TransactionValidation for TxEnv {
    type ValidationError = InvalidTransaction;
}

pub enum TxType {
    Legacy,
    Eip1559,
    BlobTx,
    EofCreate,
}

impl Default for TxEnv {
    fn default() -> Self {
        Self {
            caller: Address::ZERO,
            gas_limit: u64::MAX,
            gas_price: U256::ZERO,
            gas_priority_fee: None,
            transact_to: TxKind::Call(Address::ZERO), // will do nothing
            value: U256::ZERO,
            data: Bytes::new(),
            chain_id: None,
            nonce: 0,
            access_list: Vec::new(),
            blob_hashes: Vec::new(),
            max_fee_per_blob_gas: None,
            authorization_list: None,
        }
    }
}

/// Structure holding block blob excess gas and it calculates blob fee.
///
/// Incorporated as part of the Cancun upgrade via [EIP-4844].
///
/// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlobExcessGasAndPrice {
    /// The excess blob gas of the block.
    pub excess_blob_gas: u64,
    /// The calculated blob gas price based on the `excess_blob_gas`, See [calc_blob_gasprice]
    pub blob_gasprice: u128,
}

impl BlobExcessGasAndPrice {
    /// Creates a new instance by calculating the blob gas price with [`calc_blob_gasprice`].
    pub fn new(excess_blob_gas: u64) -> Self {
        let blob_gasprice = calc_blob_gasprice(excess_blob_gas);
        Self {
            excess_blob_gas,
            blob_gasprice,
        }
    }
}

/// Transaction destination
pub type TransactTo = TxKind;

/// Create scheme.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CreateScheme {
    /// Legacy create scheme of `CREATE`.
    Create,
    /// Create scheme of `CREATE2`.
    Create2 {
        /// Salt.
        salt: U256,
    },
}

/// What bytecode analysis to perform.
#[derive(Clone, Default, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnalysisKind {
    /// Do not perform bytecode analysis.
    Raw,
    /// Perform bytecode analysis.
    #[default]
    Analyse,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tx_chain_id() {
        let mut env = Env::<BlockEnv, TxEnv>::default();
        env.tx.chain_id = Some(1);
        env.cfg.chain_id = 2;
        assert_eq!(
            env.validate_tx::<crate::LatestSpec>(),
            Err(InvalidTransaction::InvalidChainId)
        );
    }

    #[test]
    fn test_validate_tx_access_list() {
        let mut env = Env::<BlockEnv, TxEnv>::default();
        env.tx.access_list = vec![AccessListItem {
            address: Address::ZERO,
            storage_keys: vec![],
        }];
        assert_eq!(
            env.validate_tx::<crate::FrontierSpec>(),
            Err(InvalidTransaction::AccessListNotSupported)
        );
    }
}
