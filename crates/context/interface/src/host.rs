//! Host interface for external blockchain state access.

use crate::{
    cfg::GasParams,
    context::{SStoreResult, SelfDestructResult, StateLoad},
    journaled_state::{AccountInfoLoad, AccountLoad},
};
use auto_impl::auto_impl;
use primitives::{hardfork::SpecId, Address, Bytes, Log, StorageKey, StorageValue, B256, U256};
use state::Bytecode;

/// Error that can happen when loading account info.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadError {
    /// Cold load skipped.
    ColdLoadSkipped,
    /// Database error.
    DBError,
}

/// Result of a TIP-1060 storage gas token counter update.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GasTokenResult {
    /// SStore result of the counter slot write, used to charge the extra
    /// SSTORE-worth of gas. `None` when no write happened (a [`GasTokenOp::Consume`]
    /// against an empty counter).
    pub counter: Option<SStoreResult>,
    /// Whether a token was actually consumed (a [`GasTokenOp::Consume`] that hit
    /// a positive counter).
    pub consumed: bool,
}

/// Derives the storage slot of an account's TIP-1060 gas token counter inside
/// the storage gas token contract.
///
/// The counter is stored directly (no Solidity mapping hash): the slot is the
/// account address left-padded into a [`StorageKey`].
#[inline]
pub fn storage_gas_token_slot(account: Address) -> StorageKey {
    StorageKey::from_be_bytes(account.into_word().0)
}

/// TIP-1060 storage creation mode, encoded in bits 64..=65 of the packed state.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum StorageCreationMode {
    /// `0`: pay the full creation cost up front and settle against the token
    /// balance for a refund at the end of the transaction.
    #[default]
    RefundTokens,
    /// `1`: always pay the full creation cost; never consume tokens.
    PreserveTokens,
    /// `2`: consume tokens synchronously to offset the creation cost.
    DirectTokens,
}

impl StorageCreationMode {
    /// Decodes the 2-bit mode field (only the low two bits are considered).
    #[inline]
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => Self::RefundTokens,
            1 => Self::PreserveTokens,
            2 => Self::DirectTokens,
            _ => Self::Reserved,
        }
    }
}

/// Decoded TIP-1060 storage gas token packed state word.
///
/// Layout of the packed [`StorageValue`]:
/// - bits `0..=63`: `gas_token_balance` (`uint64`)
/// - bits `64..=65`: [`storage_creation_mode`](Self::storage_creation_mode)
/// - bits `66..=255`: reserved for future hardfork-gated extensions
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StorageGasTokenState {
    /// Number of storage gas tokens held by the account (bits `0..=63`).
    pub gas_token_balance: u64,
    /// Storage creation mode (bits `64..=65`).
    pub storage_creation_mode: StorageCreationMode,
}

impl From<StorageGasTokenState> for StorageValue {
    fn from(state: StorageGasTokenState) -> Self {
        StorageValue::from_limbs([
            state.gas_token_balance,
            state.storage_creation_mode as u64,
            0,
            0,
        ])
    }
}

impl From<StorageValue> for StorageGasTokenState {
    fn from(value: StorageValue) -> Self {
        // `StorageValue` (U256) limbs are little-endian: limb 0 holds bits 0..=63,
        // limb 1 holds bits 64..=127.
        let limbs = value.as_limbs();
        StorageGasTokenState {
            gas_token_balance: limbs[0],
            storage_creation_mode: StorageCreationMode::from_bits(limbs[1] as u8),
        }
    }
}
/// Host trait with all methods that are needed by the Interpreter.
///
/// This trait is implemented for all types that have `ContextTr` trait.
///
/// There are few groups of functions which are Block, Transaction, Config, Database and Journal functions.
#[auto_impl(&mut, Box)]
pub trait Host {
    /* Block */

    /// Block basefee, calls ContextTr::block().basefee()
    fn basefee(&self) -> U256;
    /// Block blob gasprice, calls `ContextTr::block().blob_gasprice()`
    fn blob_gasprice(&self) -> U256;
    /// Block gas limit, calls ContextTr::block().gas_limit()
    fn gas_limit(&self) -> U256;
    /// Block difficulty, calls ContextTr::block().difficulty()
    fn difficulty(&self) -> U256;
    /// Block prevrandao, calls ContextTr::block().prevrandao()
    fn prevrandao(&self) -> Option<U256>;
    /// Block number, calls ContextTr::block().number()
    fn block_number(&self) -> U256;
    /// Block timestamp, calls ContextTr::block().timestamp()
    fn timestamp(&self) -> U256;
    /// Block beneficiary, calls ContextTr::block().beneficiary()
    fn beneficiary(&self) -> Address;
    /// Block slot number, calls ContextTr::block().slot_num()
    fn slot_num(&self) -> U256;
    /// Chain id, calls ContextTr::cfg().chain_id()
    fn chain_id(&self) -> U256;

    /* Transaction */

    /// Transaction effective gas price, calls `ContextTr::tx().effective_gas_price(basefee as u128)`
    fn effective_gas_price(&self) -> U256;
    /// Transaction caller, calls `ContextTr::tx().caller()`
    fn caller(&self) -> Address;
    /// Transaction blob hash, calls `ContextTr::tx().blob_hash(number)`
    fn blob_hash(&self, number: usize) -> Option<U256>;

    /* Config */

    /// Max initcode size, calls `ContextTr::cfg().max_code_size().saturating_mul(2)`
    fn max_initcode_size(&self) -> usize;

    /// Gas params contains the dynamic gas constants for the EVM.
    fn gas_params(&self) -> &GasParams;

    /// Returns whether state gas (EIP-8037) is enabled.
    fn is_amsterdam_eip8037_enabled(&self) -> bool;

    /// Returns the TIP-1060 storage gas token account, calls
    /// `ContextTr::cfg().storage_gas_token_contract()`.
    ///
    /// When `Some`, SSTORE operations that create (0→x) or clear (x→0) storage
    /// mint/consume a per-account counter held in this account.
    fn storage_gas_token_contract(&self) -> Option<Address> {
        None
    }

    /* Database */

    /// Block hash, calls `ContextTr::journal_mut().db().block_hash(number)`
    fn block_hash(&mut self, number: u64) -> Option<B256>;

    /* Journal */

    /// Selfdestruct account, calls `ContextTr::journal_mut().selfdestruct(address, target)`
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SelfDestructResult>, LoadError>;

    /// Log, calls `ContextTr::journal_mut().log(log)`
    fn log(&mut self, log: Log);

    /// Sstore with optional fetch from database. Return none if the value is cold or if there is db error.
    fn sstore_skip_cold_load(
        &mut self,
        address: Address,
        key: StorageKey,
        value: StorageValue,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SStoreResult>, LoadError>;

    /// Sstore, calls `ContextTr::journal_mut().sstore(address, key, value)`
    fn sstore(
        &mut self,
        address: Address,
        key: StorageKey,
        value: StorageValue,
    ) -> Option<StateLoad<SStoreResult>> {
        self.sstore_skip_cold_load(address, key, value, false).ok()
    }

    /// Sload with optional fetch from database. Return none if the value is cold or if there is db error.
    fn sload_skip_cold_load(
        &mut self,
        address: Address,
        key: StorageKey,
        skip_cold_load: bool,
    ) -> Result<StateLoad<StorageValue>, LoadError>;

    /// Sload, calls `ContextTr::journal_mut().sload(address, key)`
    fn sload(&mut self, address: Address, key: StorageKey) -> Option<StateLoad<StorageValue>> {
        self.sload_skip_cold_load(address, key, false).ok()
    }

    /// Tstore, calls `ContextTr::journal_mut().tstore(address, key, value)`
    fn tstore(&mut self, address: Address, key: StorageKey, value: StorageValue);

    /// Tload, calls `ContextTr::journal_mut().tload(address, key)`
    fn tload(&mut self, address: Address, key: StorageKey) -> StorageValue;

    /// Main function to load account info.
    ///
    /// If load_code is true, it will load the code fetching it from the database if not done before.
    ///
    /// If skip_cold_load is true, it will not load the account if it is cold. This is needed to short circuit
    /// the load if there is not enough gas.
    ///
    /// Returns AccountInfo, is_cold and is_empty.
    fn load_account_info_skip_cold_load(
        &mut self,
        address: Address,
        load_code: bool,
        skip_cold_load: bool,
    ) -> Result<AccountInfoLoad<'_>, LoadError>;

    /// Balance, calls `ContextTr::journal_mut().load_account(address)`
    #[inline]
    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>> {
        self.load_account_info_skip_cold_load(address, false, false)
            .ok()
            .map(|load| load.into_state_load(|i| i.balance))
    }

    /// Load account delegated, calls `ContextTr::journal_mut().load_account_delegated(address)`
    #[inline]
    fn load_account_delegated(&mut self, address: Address) -> Option<StateLoad<AccountLoad>> {
        let account = self
            .load_account_info_skip_cold_load(address, true, false)
            .ok()?;

        let mut account_load = StateLoad::new(
            AccountLoad {
                is_delegate_account_cold: None,
                is_empty: account.is_empty,
            },
            account.is_cold,
        );

        // load delegate code if account is EIP-7702
        if let Some(address) = account.code.as_ref().and_then(Bytecode::eip7702_address) {
            let delegate_account = self
                .load_account_info_skip_cold_load(address, true, false)
                .ok()?;
            account_load.data.is_delegate_account_cold = Some(delegate_account.is_cold);
            account_load.data.is_empty = delegate_account.is_empty;
        }

        Some(account_load)
    }

    /// Load account code, calls [`Host::load_account_info_skip_cold_load`] with `load_code` set to false.
    #[inline]
    fn load_account_code(&mut self, address: Address) -> Option<StateLoad<Bytes>> {
        self.load_account_info_skip_cold_load(address, true, false)
            .ok()
            .map(|load| {
                load.into_state_load(|i| {
                    i.code
                        .as_ref()
                        .map(|b| b.original_bytes())
                        .unwrap_or_default()
                })
            })
    }

    /// Load account code hash, calls [`Host::load_account_info_skip_cold_load`] with `load_code` set to false.
    #[inline]
    fn load_account_code_hash(&mut self, address: Address) -> Option<StateLoad<B256>> {
        self.load_account_info_skip_cold_load(address, false, false)
            .ok()
            .map(|load| {
                load.into_state_load(|i| {
                    if i.is_empty() {
                        B256::ZERO
                    } else {
                        i.code_hash
                    }
                })
            })
    }
}

/// Dummy host that implements [`Host`] trait and  returns all default values.
#[derive(Default, Debug)]
pub struct DummyHost {
    gas_params: GasParams,
}

impl DummyHost {
    /// Create a new dummy host with the given spec.
    pub fn new(spec: SpecId) -> Self {
        Self {
            gas_params: GasParams::new_spec(spec),
        }
    }
}

impl Host for DummyHost {
    fn basefee(&self) -> U256 {
        U256::ZERO
    }

    fn blob_gasprice(&self) -> U256 {
        U256::ZERO
    }

    fn gas_limit(&self) -> U256 {
        U256::ZERO
    }

    fn gas_params(&self) -> &GasParams {
        &self.gas_params
    }

    fn is_amsterdam_eip8037_enabled(&self) -> bool {
        false
    }

    fn difficulty(&self) -> U256 {
        U256::ZERO
    }

    fn prevrandao(&self) -> Option<U256> {
        None
    }

    fn block_number(&self) -> U256 {
        U256::ZERO
    }

    fn timestamp(&self) -> U256 {
        U256::ZERO
    }

    fn beneficiary(&self) -> Address {
        Address::ZERO
    }

    fn slot_num(&self) -> U256 {
        U256::ZERO
    }

    fn chain_id(&self) -> U256 {
        U256::ZERO
    }

    fn effective_gas_price(&self) -> U256 {
        U256::ZERO
    }

    fn caller(&self) -> Address {
        Address::ZERO
    }

    fn blob_hash(&self, _number: usize) -> Option<U256> {
        None
    }

    fn max_initcode_size(&self) -> usize {
        0
    }

    fn block_hash(&mut self, _number: u64) -> Option<B256> {
        None
    }

    fn selfdestruct(
        &mut self,
        _address: Address,
        _target: Address,
        _skip_cold_load: bool,
    ) -> Result<StateLoad<SelfDestructResult>, LoadError> {
        Ok(Default::default())
    }

    fn log(&mut self, _log: Log) {}

    fn tstore(&mut self, _address: Address, _key: StorageKey, _value: StorageValue) {}

    fn tload(&mut self, _address: Address, _key: StorageKey) -> StorageValue {
        StorageValue::ZERO
    }

    fn load_account_info_skip_cold_load(
        &mut self,
        _address: Address,
        _load_code: bool,
        _skip_cold_load: bool,
    ) -> Result<AccountInfoLoad<'_>, LoadError> {
        Ok(Default::default())
    }

    fn sstore_skip_cold_load(
        &mut self,
        _address: Address,
        _key: StorageKey,
        _value: StorageValue,
        _skip_cold_load: bool,
    ) -> Result<StateLoad<SStoreResult>, LoadError> {
        Ok(Default::default())
    }

    fn sload_skip_cold_load(
        &mut self,
        _address: Address,
        _key: StorageKey,
        _skip_cold_load: bool,
    ) -> Result<StateLoad<StorageValue>, LoadError> {
        Ok(Default::default())
    }
}
