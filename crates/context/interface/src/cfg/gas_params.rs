//! Gas table for dynamic gas constants.

use crate::{
    cfg::gas::{self, get_tokens_in_calldata, InitialAndFloorGas},
    context::SStoreResult,
    transaction::AccessListItemTr as _,
    Transaction, TransactionType,
};
use core::hash::{Hash, Hasher};
use primitives::{
    eip2780, eip7702, eip8037, eip8038,
    hardfork::SpecId::{self},
    OnceLock, U256,
};
use std::sync::Arc;

/// Gas table for dynamic gas constants.
#[derive(Clone)]
pub struct GasParams {
    /// Table of gas costs for operations
    table: Arc<[u64; 256]>,
}

impl PartialEq<GasParams> for GasParams {
    fn eq(&self, other: &GasParams) -> bool {
        self.table == other.table
    }
}

impl Hash for GasParams {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.table.hash(hasher);
    }
}

impl core::fmt::Debug for GasParams {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "GasParams {{ table: {:?} }}", self.table)
    }
}

/// Returns number of words what would fit to provided number of bytes,
/// i.e. it rounds up the number bytes to number of words.
#[inline]
pub const fn num_words(len: usize) -> usize {
    len.div_ceil(32)
}

impl Eq for GasParams {}
#[cfg(feature = "serde")]
mod serde {
    use super::{Arc, GasParams};
    use std::vec::Vec;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct GasParamsSerde {
        table: Vec<u64>,
    }

    #[cfg(feature = "serde")]
    impl serde::Serialize for GasParams {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            GasParamsSerde {
                table: self.table.to_vec(),
            }
            .serialize(serializer)
        }
    }

    impl<'de> serde::Deserialize<'de> for GasParams {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let table = GasParamsSerde::deserialize(deserializer)?;
            if table.table.len() != 256 {
                return Err(serde::de::Error::custom("Invalid gas params length"));
            }
            Ok(Self::new(Arc::new(table.table.try_into().unwrap())))
        }
    }
}

impl Default for GasParams {
    #[inline]
    fn default() -> Self {
        Self::new_spec(SpecId::default())
    }
}

impl GasParams {
    /// Creates a new `GasParams` with the given table.
    #[inline]
    pub const fn new(table: Arc<[u64; 256]>) -> Self {
        Self { table }
    }

    /// Overrides the gas cost for the given gas id.
    ///
    /// It will clone underlying table and override the values.
    ///
    /// Use to override default gas cost
    ///
    /// ```rust
    /// use revm_context_interface::cfg::gas_params::{GasParams, GasId};
    /// use primitives::hardfork::SpecId;
    ///
    /// let mut gas_table = GasParams::new_spec(SpecId::default());
    /// gas_table.override_gas([(GasId::memory_linear_cost(), 2), (GasId::memory_quadratic_reduction(), 512)].into_iter());
    /// assert_eq!(gas_table.get(GasId::memory_linear_cost()), 2);
    /// assert_eq!(gas_table.get(GasId::memory_quadratic_reduction()), 512);
    /// ```
    pub fn override_gas(&mut self, values: impl IntoIterator<Item = (GasId, u64)>) {
        let mut table = *self.table.clone();
        for (id, value) in values.into_iter() {
            table[id.as_usize()] = value;
        }
        *self = Self::new(Arc::new(table));
    }

    /// Returns the table.
    #[inline]
    pub fn table(&self) -> &[u64; 256] {
        &self.table
    }

    /// Creates a new `GasParams` for the given spec.
    #[inline(never)]
    pub fn new_spec(spec: SpecId) -> Self {
        use SpecId::*;
        let gas_params = match spec {
            FRONTIER => {
                static TABLE: OnceLock<GasParams> = OnceLock::new();
                TABLE.get_or_init(|| Self::new_spec_inner(spec))
            }
            // Transaction creation cost was added in homestead fork.
            HOMESTEAD => {
                static TABLE: OnceLock<GasParams> = OnceLock::new();
                TABLE.get_or_init(|| Self::new_spec_inner(spec))
            }
            // New account cost for selfdestruct was added in tangerine fork.
            TANGERINE => {
                static TABLE: OnceLock<GasParams> = OnceLock::new();
                TABLE.get_or_init(|| Self::new_spec_inner(spec))
            }
            // EXP cost was increased in spurious dragon fork.
            SPURIOUS_DRAGON | BYZANTIUM | PETERSBURG => {
                static TABLE: OnceLock<GasParams> = OnceLock::new();
                TABLE.get_or_init(|| Self::new_spec_inner(spec))
            }
            // SSTORE gas calculation changed in istanbul fork.
            ISTANBUL => {
                static TABLE: OnceLock<GasParams> = OnceLock::new();
                TABLE.get_or_init(|| Self::new_spec_inner(spec))
            }
            // Warm/cold state access
            BERLIN => {
                static TABLE: OnceLock<GasParams> = OnceLock::new();
                TABLE.get_or_init(|| Self::new_spec_inner(spec))
            }
            // Refund reduction in london fork.
            LONDON | MERGE => {
                static TABLE: OnceLock<GasParams> = OnceLock::new();
                TABLE.get_or_init(|| Self::new_spec_inner(spec))
            }
            // Transaction initcode cost was introduced in shanghai fork.
            SHANGHAI | CANCUN => {
                static TABLE: OnceLock<GasParams> = OnceLock::new();
                TABLE.get_or_init(|| Self::new_spec_inner(spec))
            }
            // EIP-7702 was introduced in prague fork.
            PRAGUE | OSAKA => {
                static TABLE: OnceLock<GasParams> = OnceLock::new();
                TABLE.get_or_init(|| Self::new_spec_inner(spec))
            }
            // New fork.
            SpecId::AMSTERDAM => {
                static TABLE: OnceLock<GasParams> = OnceLock::new();
                TABLE.get_or_init(|| Self::new_spec_inner(spec))
            }
        };
        gas_params.clone()
    }

    /// Creates a new `GasParams` for the given spec.
    #[inline]
    fn new_spec_inner(spec: SpecId) -> Self {
        let mut table = [0; 256];

        table[GasId::exp_byte_gas().as_usize()] = 10;
        table[GasId::logdata().as_usize()] = gas::LOGDATA;
        table[GasId::logtopic().as_usize()] = gas::LOGTOPIC;
        table[GasId::copy_per_word().as_usize()] = gas::COPY;
        table[GasId::extcodecopy_per_word().as_usize()] = gas::COPY;
        table[GasId::mcopy_per_word().as_usize()] = gas::COPY;
        table[GasId::keccak256_per_word().as_usize()] = gas::KECCAK256WORD;
        table[GasId::memory_linear_cost().as_usize()] = gas::MEMORY;
        table[GasId::memory_quadratic_reduction().as_usize()] = 512;
        table[GasId::initcode_per_word().as_usize()] = gas::INITCODE_WORD_COST;
        table[GasId::create().as_usize()] = gas::CREATE;
        table[GasId::call_stipend_reduction().as_usize()] = 64;
        table[GasId::max_refund_quotient().as_usize()] = 2;
        table[GasId::transfer_value_cost().as_usize()] = gas::CALLVALUE;
        table[GasId::cold_account_additional_cost().as_usize()] = 0;
        table[GasId::new_account_cost().as_usize()] = gas::NEWACCOUNT;
        table[GasId::warm_storage_read_cost().as_usize()] = 0;
        // Frontiers had fixed 5k cost.
        table[GasId::sstore_static().as_usize()] = gas::SSTORE_RESET;
        // SSTORE SET
        table[GasId::sstore_set_without_load_cost().as_usize()] =
            gas::SSTORE_SET - gas::SSTORE_RESET;
        // SSTORE RESET Is covered in SSTORE_STATIC.
        table[GasId::sstore_reset_without_cold_load_cost().as_usize()] = 0;
        // SSTORE SET REFUND (same as sstore_set_without_load_cost but used only in sstore_refund)
        table[GasId::sstore_set_refund().as_usize()] =
            table[GasId::sstore_set_without_load_cost().as_usize()];
        // SSTORE RESET REFUND (same as sstore_reset_without_cold_load_cost but used only in sstore_refund)
        table[GasId::sstore_reset_refund().as_usize()] =
            table[GasId::sstore_reset_without_cold_load_cost().as_usize()];
        // SSTORE CLEARING SLOT REFUND
        table[GasId::sstore_clearing_slot_refund().as_usize()] = 15000;
        table[GasId::selfdestruct_refund().as_usize()] = 24000;
        table[GasId::call_stipend().as_usize()] = gas::CALL_STIPEND;
        table[GasId::cold_storage_additional_cost().as_usize()] = 0;
        table[GasId::cold_storage_cost().as_usize()] = 0;
        table[GasId::new_account_cost_for_selfdestruct().as_usize()] = 0;
        table[GasId::code_deposit_cost().as_usize()] = gas::CODEDEPOSIT;
        table[GasId::tx_token_non_zero_byte_multiplier().as_usize()] =
            gas::NON_ZERO_BYTE_MULTIPLIER;
        table[GasId::tx_token_cost().as_usize()] = gas::STANDARD_TOKEN_COST;
        table[GasId::tx_base_stipend().as_usize()] = 21000;

        if spec.is_enabled_in(SpecId::HOMESTEAD) {
            table[GasId::tx_create_cost().as_usize()] = gas::CREATE;
        }

        if spec.is_enabled_in(SpecId::TANGERINE) {
            table[GasId::new_account_cost_for_selfdestruct().as_usize()] = gas::NEWACCOUNT;
        }

        if spec.is_enabled_in(SpecId::SPURIOUS_DRAGON) {
            table[GasId::exp_byte_gas().as_usize()] = 50;
        }

        if spec.is_enabled_in(SpecId::ISTANBUL) {
            table[GasId::sstore_static().as_usize()] = gas::ISTANBUL_SLOAD_GAS;
            table[GasId::sstore_set_without_load_cost().as_usize()] =
                gas::SSTORE_SET - gas::ISTANBUL_SLOAD_GAS;
            table[GasId::sstore_reset_without_cold_load_cost().as_usize()] =
                gas::SSTORE_RESET - gas::ISTANBUL_SLOAD_GAS;
            table[GasId::sstore_set_refund().as_usize()] =
                table[GasId::sstore_set_without_load_cost().as_usize()];
            table[GasId::sstore_reset_refund().as_usize()] =
                table[GasId::sstore_reset_without_cold_load_cost().as_usize()];
            table[GasId::tx_token_non_zero_byte_multiplier().as_usize()] =
                gas::NON_ZERO_BYTE_MULTIPLIER_ISTANBUL;
        }

        if spec.is_enabled_in(SpecId::BERLIN) {
            table[GasId::sstore_static().as_usize()] = gas::WARM_STORAGE_READ_COST;
            table[GasId::cold_account_additional_cost().as_usize()] =
                gas::COLD_ACCOUNT_ACCESS_COST_ADDITIONAL;
            table[GasId::cold_storage_additional_cost().as_usize()] =
                gas::COLD_SLOAD_COST - gas::WARM_STORAGE_READ_COST;
            table[GasId::cold_storage_cost().as_usize()] = gas::COLD_SLOAD_COST;
            table[GasId::warm_storage_read_cost().as_usize()] = gas::WARM_STORAGE_READ_COST;

            table[GasId::sstore_reset_without_cold_load_cost().as_usize()] =
                gas::WARM_SSTORE_RESET - gas::WARM_STORAGE_READ_COST;
            table[GasId::sstore_set_without_load_cost().as_usize()] =
                gas::SSTORE_SET - gas::WARM_STORAGE_READ_COST;
            table[GasId::sstore_set_refund().as_usize()] =
                table[GasId::sstore_set_without_load_cost().as_usize()];
            table[GasId::sstore_reset_refund().as_usize()] =
                table[GasId::sstore_reset_without_cold_load_cost().as_usize()];

            table[GasId::tx_access_list_address_cost().as_usize()] = gas::ACCESS_LIST_ADDRESS;
            table[GasId::tx_access_list_storage_key_cost().as_usize()] =
                gas::ACCESS_LIST_STORAGE_KEY;
        }

        if spec.is_enabled_in(SpecId::LONDON) {
            // EIP-3529: Reduction in refunds

            // Replace SSTORE_CLEARS_SCHEDULE (as defined in EIP-2200) with
            // SSTORE_RESET_GAS + ACCESS_LIST_STORAGE_KEY_COST (4,800 gas as of EIP-2929 + EIP-2930)
            table[GasId::sstore_clearing_slot_refund().as_usize()] =
                gas::WARM_SSTORE_RESET + gas::ACCESS_LIST_STORAGE_KEY;

            table[GasId::selfdestruct_refund().as_usize()] = 0;
            table[GasId::max_refund_quotient().as_usize()] = 5;
        }

        if spec.is_enabled_in(SpecId::SHANGHAI) {
            table[GasId::tx_initcode_cost().as_usize()] = gas::INITCODE_WORD_COST;
        }

        if spec.is_enabled_in(SpecId::PRAGUE) {
            table[GasId::tx_eip7702_regular_gas().as_usize()] = eip7702::PER_EMPTY_ACCOUNT_COST;

            // EIP-7702 authorization refund for existing accounts
            table[GasId::tx_eip7702_regular_refund().as_usize()] =
                eip7702::PER_EMPTY_ACCOUNT_COST - eip7702::PER_AUTH_BASE_COST;

            table[GasId::tx_floor_cost_per_token().as_usize()] = gas::TOTAL_COST_FLOOR_PER_TOKEN;
            table[GasId::tx_floor_cost_base_gas().as_usize()] = 21000;
            // EIP-7623 floor tokens reuse `tokens_in_calldata`, i.e. zero bytes count as
            // one token each.
            table[GasId::tx_floor_token_zero_byte_multiplier().as_usize()] = 1;
        }

        // EIP-8037: State creation gas cost increase.
        // State-gas entries store final gas values, with Glamsterdam CPSB applied
        // once when building the gas table.
        if spec.is_enabled_in(SpecId::AMSTERDAM) {
            // Regular gas changes
            table[GasId::create().as_usize()] = 9000;
            table[GasId::tx_create_cost().as_usize()] = 9000;
            table[GasId::code_deposit_cost().as_usize()] = 0;
            table[GasId::new_account_cost().as_usize()] = 0;
            table[GasId::new_account_cost_for_selfdestruct().as_usize()] = 0;
            // GAS_STORAGE_SET regular = GAS_STORAGE_UPDATE - GAS_COLD_SLOAD = 5000 - 2100 = 2900
            // sstore_set_without_load_cost = 2900 - WARM_STORAGE_READ_COST(100) = 2800
            table[GasId::sstore_set_without_load_cost().as_usize()] = 2800;

            // State gas values with Glamsterdam CPSB baked in.
            table[GasId::sstore_set_state_gas().as_usize()] =
                eip8037::SSTORE_SET_BYTES * eip8037::CPSB_GLAMSTERDAM;
            table[GasId::new_account_state_gas().as_usize()] =
                eip8037::NEW_ACCOUNT_BYTES * eip8037::CPSB_GLAMSTERDAM;
            table[GasId::code_deposit_state_gas().as_usize()] =
                eip8037::CODE_DEPOSIT_PER_BYTE * eip8037::CPSB_GLAMSTERDAM;
            table[GasId::create_state_gas().as_usize()] =
                eip8037::NEW_ACCOUNT_BYTES * eip8037::CPSB_GLAMSTERDAM;
            table[GasId::tx_eip7702_state_gas_bytecode().as_usize()] =
                eip8037::AUTH_BASE_BYTES * eip8037::CPSB_GLAMSTERDAM;

            // SSTORE refund for 0→X→0 restoration: regular gas only.
            // The state-gas portion is restored directly
            // to the reservoir via `GasParams::sstore_state_gas_refill`.
            table[GasId::sstore_set_refund().as_usize()] = 2800;

            // EIP-2780: the floor base drops from 21,000 to TX_BASE (12,000).
            table[GasId::tx_floor_cost_base_gas().as_usize()] = eip2780::TX_BASE_COST;

            // EIP-7976: Increase calldata floor cost from 10/40 to 64/64 gas per byte
            // (zero/nonzero). The per-token constant bumps from 10 to 16, and
            // `floor_tokens_in_calldata` switches from `zero + nonzero * 4` to
            // `(zero + nonzero) * 4`, i.e. every byte now costs 16 * 4 = 64 gas in the floor.
            table[GasId::tx_floor_cost_per_token().as_usize()] = 16;
            table[GasId::tx_floor_token_zero_byte_multiplier().as_usize()] =
                table[GasId::tx_token_non_zero_byte_multiplier().as_usize()];

            // EIP-7981: Charge access list data at 64 gas per byte, matching
            // calldata floor pricing. Per-item costs bake in the data charge:
            //   address: 2400 + 20 * 64 = 3680
            //   key:     1900 + 32 * 64 = 3948
            // And every access-list byte contributes 4 floor tokens (16 * 4 = 64 gas).
            table[GasId::tx_access_list_address_cost().as_usize()] =
                gas::ACCESS_LIST_ADDRESS + 20 * 64;
            table[GasId::tx_access_list_storage_key_cost().as_usize()] =
                gas::ACCESS_LIST_STORAGE_KEY + 32 * 64;
            table[GasId::tx_access_list_floor_byte_multiplier().as_usize()] = 4;

            // EIP-8038: State-access gas cost update (ethereum/EIPs#11802;
            // preliminary draft values). Constants live in `primitives::eip8038`.
            //   WARM_ACCESS                    100 ->    100  (unchanged)
            //   COLD_ACCOUNT_ACCESS          2,600 ->  3,000
            //   ACCOUNT_WRITE                6,700 ->  8,000
            //   COLD_STORAGE_ACCESS          2,100 ->  3,000
            //   STORAGE_WRITE                2,800 -> 10,000
            //   STORAGE_CLEAR_REFUND         4,800 -> 12,480
            //   CREATE_ACCESS                7,000 -> 11,000  (ACCOUNT_WRITE + COLD_STORAGE_ACCESS)
            //   ACCESS_LIST_ADDRESS_COST     2,400 ->  3,000  (COLD_ACCOUNT_ACCESS)
            //   ACCESS_LIST_STORAGE_KEY_COST 1,900 ->  3,000  (COLD_STORAGE_ACCESS)
            //
            // Account access table values.
            table[GasId::warm_storage_read_cost().as_usize()] = eip8038::WARM_ACCESS;
            table[GasId::cold_account_additional_cost().as_usize()] =
                eip8038::COLD_ACCOUNT_ACCESS_ADDITIONAL;
            table[GasId::cold_storage_additional_cost().as_usize()] =
                eip8038::COLD_STORAGE_ACCESS_ADDITIONAL;
            // EIP-8038 folds the warm base into the cold cost: a cold SSTORE pays
            // COLD_STORAGE_ACCESS (3000) total, not warm(100)+cold. Since
            // `sstore_static` (warm, 100) is always charged in `sstore_dynamic_gas`,
            // the cold add-on here is the premium above warm (2900), unlike pre-8038
            // forks which add the full `COLD_SLOAD_COST` on top of the warm base.
            table[GasId::cold_storage_cost().as_usize()] = eip8038::COLD_STORAGE_ACCESS_ADDITIONAL;
            // CALL_VALUE = ACCOUNT_WRITE + CALL_STIPEND.
            // CALL_VALUE = ACCOUNT_WRITE + CALL_STIPEND. A value-bearing CALL already
            // pays the ACCOUNT_WRITE surcharge via `transfer_value_cost`, so creating
            // the target charges no extra regular gas — only the NEW_ACCOUNT state gas
            // (hence `new_account_cost` is zero). SELFDESTRUCT has no such bundled
            // charge, so it still pays a separate ACCOUNT_WRITE when sending balance to
            // an empty account (execution-specs `selfdestruct`).
            table[GasId::transfer_value_cost().as_usize()] = eip8038::CALL_VALUE;
            table[GasId::new_account_cost().as_usize()] = 0;
            table[GasId::new_account_cost_for_selfdestruct().as_usize()] = eip8038::ACCOUNT_WRITE;

            // SSTORE table values.
            //   warm-base       = WARM_ACCESS         (sstore_static)
            //   write surcharge = STORAGE_WRITE       (sstore_set / sstore_reset dynamic)
            //   refunds         = STORAGE_WRITE / STORAGE_CLEAR_REFUND
            table[GasId::sstore_static().as_usize()] = eip8038::WARM_ACCESS;
            table[GasId::sstore_set_without_load_cost().as_usize()] = eip8038::STORAGE_WRITE;
            table[GasId::sstore_reset_without_cold_load_cost().as_usize()] = eip8038::STORAGE_WRITE;
            table[GasId::sstore_set_refund().as_usize()] = eip8038::STORAGE_WRITE;
            table[GasId::sstore_reset_refund().as_usize()] = eip8038::STORAGE_WRITE;
            table[GasId::sstore_clearing_slot_refund().as_usize()] = eip8038::STORAGE_CLEAR_REFUND;

            // CREATE / CREATE2 regular-gas access cost.
            //   `create` slot is the regular-gas portion charged at the
            //   CREATE/CREATE2 opcodes and for create-kind txns.
            table[GasId::create().as_usize()] = eip8038::CREATE_ACCESS;
            table[GasId::tx_create_cost().as_usize()] = eip8038::CREATE_ACCESS;

            // Access-list per-item costs: EIP-8038 base (COLD_*_ACCESS, 3,000 each),
            // keeping the EIP-7981 64 gas/byte data charge on top.
            table[GasId::tx_access_list_address_cost().as_usize()] =
                eip8038::ACCESS_LIST_ADDRESS_COST + 20 * 64;
            table[GasId::tx_access_list_storage_key_cost().as_usize()] =
                eip8038::ACCESS_LIST_STORAGE_KEY_COST + 32 * 64;

            // EIP-7702 under EIP-2780: the intrinsic per-auth charge is the
            // state-independent REGULAR_PER_AUTH_BASE_COST (7,816) only. The
            // state-dependent remainder — ACCOUNT_WRITE plus the new-account
            // (`new_account_state_gas`) and delegation-bytes
            // (`tx_eip7702_state_gas_bytecode`) state gas — is charged at the
            // runtime gas phase, per authority that incurs it, so the
            // pre-Amsterdam per-auth refund never applies.
            table[GasId::tx_eip7702_regular_gas().as_usize()] =
                eip8038::EIP7702_PER_AUTH_BASE_REGULAR;
            table[GasId::tx_eip7702_regular_refund().as_usize()] = 0;

            // EIP-2780: Intrinsic gas decomposition. The new path uses
            // `eip2780::TX_BASE_COST` directly for the sender base and these
            // entries for the additional `to`- and `value`-based charges.
            // ACCOUNT_WRITE / CREATE_ACCESS source from `eip8038` so a single
            // change to the placeholder TBD values propagates everywhere.
            table[GasId::tx_transfer_log_cost().as_usize()] = eip2780::TRANSFER_LOG_COST;
            table[GasId::tx_account_write_cost().as_usize()] = eip8038::ACCOUNT_WRITE;
            table[GasId::tx_create_access_cost().as_usize()] = eip8038::CREATE_ACCESS;
        }

        Self::new(Arc::new(table))
    }

    /// Gets the gas cost for the given gas id.
    #[inline]
    pub fn get(&self, id: GasId) -> u64 {
        self.table[id.as_usize()]
    }

    /// `EXP` opcode cost calculation.
    #[inline]
    pub fn exp_cost(&self, power: U256) -> u64 {
        if power.is_zero() {
            return 0;
        }
        // EIP-160: EXP cost increase
        self.get(GasId::exp_byte_gas())
            .saturating_mul(log2floor(power) / 8 + 1)
    }

    /// Selfdestruct refund.
    #[inline]
    pub fn selfdestruct_refund(&self) -> i64 {
        self.get(GasId::selfdestruct_refund()) as i64
    }

    /// Selfdestruct cold cost is calculated differently from other cold costs.
    /// and it contains both cold and warm costs.
    #[inline]
    pub fn selfdestruct_cold_cost(&self) -> u64 {
        self.cold_account_additional_cost() + self.warm_storage_read_cost()
    }

    /// Selfdestruct cost.
    #[inline]
    pub fn selfdestruct_cost(&self, should_charge_topup: bool, is_cold: bool) -> u64 {
        let mut gas = 0;

        // EIP-150: Gas cost changes for IO-heavy operations
        if should_charge_topup {
            gas += self.new_account_cost_for_selfdestruct();
        }

        if is_cold {
            // Note: SELFDESTRUCT does not charge a WARM_STORAGE_READ_COST in case the recipient is already warm,
            // which differs from how the other call-variants work. The reasoning behind this is to keep
            // the changes small, a SELFDESTRUCT already costs 5K and is a no-op if invoked more than once.
            //
            // For GasParams both values are zero before BERLIN fork.
            gas += self.selfdestruct_cold_cost();
        }
        gas
    }

    /// EXTCODECOPY gas cost
    #[inline]
    pub fn extcodecopy(&self, len: usize) -> u64 {
        self.get(GasId::extcodecopy_per_word())
            .saturating_mul(num_words(len) as u64)
    }

    /// MCOPY gas cost
    #[inline]
    pub fn mcopy_cost(&self, len: usize) -> u64 {
        self.get(GasId::mcopy_per_word())
            .saturating_mul(num_words(len) as u64)
    }

    /// Static gas cost for SSTORE opcode
    #[inline]
    pub fn sstore_static_gas(&self) -> u64 {
        self.get(GasId::sstore_static())
    }

    /// SSTORE set cost
    #[inline]
    pub fn sstore_set_without_load_cost(&self) -> u64 {
        self.get(GasId::sstore_set_without_load_cost())
    }

    /// SSTORE reset cost
    #[inline]
    pub fn sstore_reset_without_cold_load_cost(&self) -> u64 {
        self.get(GasId::sstore_reset_without_cold_load_cost())
    }

    /// SSTORE clearing slot refund
    #[inline]
    pub fn sstore_clearing_slot_refund(&self) -> u64 {
        self.get(GasId::sstore_clearing_slot_refund())
    }

    /// SSTORE set refund. Used in sstore_refund for SSTORE_SET_GAS - SLOAD_GAS.
    #[inline]
    pub fn sstore_set_refund(&self) -> u64 {
        self.get(GasId::sstore_set_refund())
    }

    /// SSTORE reset refund. Used in sstore_refund for SSTORE_RESET_GAS - SLOAD_GAS.
    #[inline]
    pub fn sstore_reset_refund(&self) -> u64 {
        self.get(GasId::sstore_reset_refund())
    }

    /// Maximum gas refund quotient.
    ///
    /// The final transaction refund is capped to `gas_used / max_refund_quotient`.
    #[inline]
    pub fn max_refund_quotient(&self) -> u64 {
        self.get(GasId::max_refund_quotient())
    }

    /// Dynamic gas cost for SSTORE opcode.
    ///
    /// Dynamic gas cost is gas that needs input from SSTORE operation to be calculated.
    #[inline]
    pub fn sstore_dynamic_gas(&self, is_istanbul: bool, vals: &SStoreResult, is_cold: bool) -> u64 {
        // frontier logic gets charged for every SSTORE operation if original value is zero.
        // this behaviour is fixed in istanbul fork.
        if !is_istanbul {
            if vals.is_present_zero() && !vals.is_new_zero() {
                return self.sstore_set_without_load_cost();
            } else {
                return self.sstore_reset_without_cold_load_cost();
            }
        }

        let mut gas = 0;

        // this will be zero before berlin fork.
        if is_cold {
            gas += self.cold_storage_cost();
        }

        // if new values changed present value and present value is unchanged from original.
        if vals.new_values_changes_present() && vals.is_original_eq_present() {
            gas += if vals.is_original_zero() {
                // set cost for creating storage slot (Zero slot means it is not existing).
                // and previous condition says present is same as original.
                self.sstore_set_without_load_cost()
            } else {
                // if new value is not zero, this means we are setting some value to it.
                self.sstore_reset_without_cold_load_cost()
            };
        }
        gas
    }

    /// SSTORE refund calculation.
    #[inline]
    pub fn sstore_refund(&self, is_istanbul: bool, vals: &SStoreResult) -> i64 {
        // EIP-3529: Reduction in refunds
        let sstore_clearing_slot_refund = self.sstore_clearing_slot_refund() as i64;

        if !is_istanbul {
            // // before istanbul fork, refund was always awarded without checking original state.
            if !vals.is_present_zero() && vals.is_new_zero() {
                return sstore_clearing_slot_refund;
            }
            return 0;
        }

        // If current value equals new value (this is a no-op)
        if vals.is_new_eq_present() {
            return 0;
        }

        // refund for the clearing of storage slot.
        // As new is not equal to present, new values zero means that original and present values are not zero
        if vals.is_original_eq_present() && vals.is_new_zero() {
            return sstore_clearing_slot_refund;
        }

        let mut refund = 0;
        // If original value is not 0
        if !vals.is_original_zero() {
            // If current value is 0 (also means that new value is not 0),
            if vals.is_present_zero() {
                // remove SSTORE_CLEARS_SCHEDULE gas from refund counter.
                refund -= sstore_clearing_slot_refund;
            // If new value is 0 (also means that current value is not 0),
            } else if vals.is_new_zero() {
                // add SSTORE_CLEARS_SCHEDULE gas to refund counter.
                refund += sstore_clearing_slot_refund;
            }
        }

        // If original value equals new value (this storage slot is reset)
        if vals.is_original_eq_new() {
            // If original value is 0
            if vals.is_original_zero() {
                // add SSTORE_SET_GAS - SLOAD_GAS to refund counter.
                refund += self.sstore_set_refund() as i64;
            // Otherwise
            } else {
                // add SSTORE_RESET_GAS - SLOAD_GAS gas to refund counter.
                refund += self.sstore_reset_refund() as i64;
            }
        }
        refund
    }

    /// `LOG` opcode cost calculation.
    #[inline]
    pub fn log_cost(&self, n: u8, len: u64) -> u64 {
        self.get(GasId::logdata())
            .saturating_mul(len)
            .saturating_add(self.get(GasId::logtopic()) * n as u64)
    }

    /// KECCAK256 gas cost per word
    #[inline]
    pub fn keccak256_cost(&self, len: usize) -> u64 {
        self.get(GasId::keccak256_per_word())
            .saturating_mul(num_words(len) as u64)
    }

    /// Memory gas cost
    #[inline]
    pub fn memory_cost(&self, len: usize) -> u64 {
        let len = len as u64;
        self.get(GasId::memory_linear_cost())
            .saturating_mul(len)
            .saturating_add(
                (len.saturating_mul(len))
                    .saturating_div(self.get(GasId::memory_quadratic_reduction())),
            )
    }

    /// Initcode word cost
    #[inline]
    pub fn initcode_cost(&self, len: usize) -> u64 {
        self.get(GasId::initcode_per_word())
            .saturating_mul(num_words(len) as u64)
    }

    /// Create gas cost
    #[inline]
    pub fn create_cost(&self) -> u64 {
        self.get(GasId::create())
    }

    /// Create2 gas cost.
    #[inline]
    pub fn create2_cost(&self, len: usize) -> u64 {
        self.get(GasId::create()).saturating_add(
            self.get(GasId::keccak256_per_word())
                .saturating_mul(num_words(len) as u64),
        )
    }

    /// Call stipend.
    #[inline]
    pub fn call_stipend(&self) -> u64 {
        self.get(GasId::call_stipend())
    }

    /// Call stipend reduction. Call stipend is reduced by 1/64 of the gas limit.
    #[inline]
    pub fn call_stipend_reduction(&self, gas_limit: u64) -> u64 {
        gas_limit - gas_limit / self.get(GasId::call_stipend_reduction())
    }

    /// Transfer value cost
    #[inline]
    pub fn transfer_value_cost(&self) -> u64 {
        self.get(GasId::transfer_value_cost())
    }

    /// Additional cold cost. Additional cold cost is added to the gas cost if the account is cold loaded.
    #[inline]
    pub fn cold_account_additional_cost(&self) -> u64 {
        self.get(GasId::cold_account_additional_cost())
    }

    /// Cold storage additional cost.
    #[inline]
    pub fn cold_storage_additional_cost(&self) -> u64 {
        self.get(GasId::cold_storage_additional_cost())
    }

    /// Cold storage cost.
    #[inline]
    pub fn cold_storage_cost(&self) -> u64 {
        self.get(GasId::cold_storage_cost())
    }

    /// New account cost. New account cost is added to the gas cost if the account is empty.
    #[inline]
    pub fn new_account_cost(&self, is_spurious_dragon: bool, transfers_value: bool) -> u64 {
        // EIP-161: State trie clearing (invariant-preserving alternative)
        // Pre-Spurious Dragon: always charge for new account
        // Post-Spurious Dragon: only charge if value is transferred
        if !is_spurious_dragon || transfers_value {
            return self.get(GasId::new_account_cost());
        }
        0
    }

    /// New account cost for selfdestruct.
    #[inline]
    pub fn new_account_cost_for_selfdestruct(&self) -> u64 {
        self.get(GasId::new_account_cost_for_selfdestruct())
    }

    /// Warm storage read cost. Warm storage read cost is added to the gas cost if the account is warm loaded.
    #[inline]
    pub fn warm_storage_read_cost(&self) -> u64 {
        self.get(GasId::warm_storage_read_cost())
    }

    /// Copy cost
    #[inline]
    pub fn copy_cost(&self, len: usize) -> u64 {
        self.copy_per_word_cost(num_words(len))
    }

    /// Copy per word cost
    #[inline]
    pub fn copy_per_word_cost(&self, word_num: usize) -> u64 {
        self.get(GasId::copy_per_word())
            .saturating_mul(word_num as u64)
    }

    /// Code deposit cost, calculated per byte as len * code_deposit_cost.
    #[inline]
    pub fn code_deposit_cost(&self, len: usize) -> u64 {
        self.get(GasId::code_deposit_cost())
            .saturating_mul(len as u64)
    }

    /// State gas for SSTORE: charges for new slot creation (zero → non-zero).
    #[inline]
    pub fn sstore_state_gas(&self, vals: &SStoreResult) -> u64 {
        if vals.new_values_changes_present()
            && vals.is_original_eq_present()
            && vals.is_original_zero()
        {
            self.get(GasId::sstore_set_state_gas())
        } else {
            0
        }
    }

    /// State gas to refill the reservoir on 0→x→0 storage restoration (EIP-8037).
    ///
    /// When a storage slot is restored to its original zero value within the
    /// same transaction, the state gas originally charged for the 0→x
    /// transition is returned directly to the reservoir (not via the capped
    /// refund counter). Returns 0 in any other case.
    ///
    #[inline]
    pub fn sstore_state_gas_refill(&self, vals: &SStoreResult) -> u64 {
        if !vals.is_new_eq_present() && vals.is_original_eq_new() && vals.is_original_zero() {
            self.get(GasId::sstore_set_state_gas())
        } else {
            0
        }
    }

    /// State gas for new account creation.
    #[inline]
    pub fn new_account_state_gas(&self) -> u64 {
        self.get(GasId::new_account_state_gas())
    }

    /// State gas for code deposit of `len` bytes.
    #[inline]
    pub fn code_deposit_state_gas(&self, len: usize) -> u64 {
        self.get(GasId::code_deposit_state_gas())
            .saturating_mul(len as u64)
    }

    /// State gas for contract metadata creation.
    #[inline]
    pub fn create_state_gas(&self) -> u64 {
        self.get(GasId::create_state_gas())
    }

    /// Used in [GasParams::initial_tx_gas] to calculate the eip7702 per-auth cost.
    ///
    /// Pre-Amsterdam this is the pessimistic bundled `PER_EMPTY_ACCOUNT_COST`
    /// (25,000). Under EIP-2780 (Amsterdam) it is the state-independent
    /// `REGULAR_PER_AUTH_BASE_COST` (7,816) only; the state-dependent remainder
    /// (`ACCOUNT_WRITE` plus the new-account / delegation-bytes state gas) is
    /// charged at the runtime gas phase, per authority that incurs it.
    #[inline]
    pub fn tx_eip7702_per_empty_account_cost(&self) -> u64 {
        self.get(GasId::tx_eip7702_regular_gas())
    }

    /// EIP-7702 per-auth refund for an already-existing authority.
    ///
    /// Pre-Amsterdam this is `PER_EMPTY_ACCOUNT_COST - PER_AUTH_BASE_COST` (12500).
    /// Under EIP-2780 it is zero — the state-dependent per-auth charges are
    /// applied at the runtime gas phase instead of refunded.
    #[inline]
    pub fn tx_eip7702_auth_refund_regular(&self) -> u64 {
        self.get(GasId::tx_eip7702_regular_refund())
    }

    /// EIP-8037: state gas for one 23-byte EIP-7702 delegation indicator
    /// (`STATE_BYTES_PER_AUTH_BASE × CPSB`). Zero before AMSTERDAM.
    #[inline]
    pub fn tx_eip7702_state_gas_bytecode(&self) -> u64 {
        self.get(GasId::tx_eip7702_state_gas_bytecode())
    }

    /// Used in [GasParams::initial_tx_gas] to calculate the token non zero byte multiplier.
    #[inline]
    pub fn tx_token_non_zero_byte_multiplier(&self) -> u64 {
        self.get(GasId::tx_token_non_zero_byte_multiplier())
    }

    /// Used in [GasParams::initial_tx_gas] to calculate the token cost for input data.
    #[inline]
    pub fn tx_token_cost(&self) -> u64 {
        self.get(GasId::tx_token_cost())
    }

    /// Used in [GasParams::initial_tx_gas] to calculate the floor gas per token.
    pub fn tx_floor_cost_per_token(&self) -> u64 {
        self.get(GasId::tx_floor_cost_per_token())
    }

    /// Multiplier for a zero byte in the floor tokens calculation.
    ///
    /// Under EIP-7623 this is `1` (zero bytes count as one token), so the floor
    /// reuses `tokens_in_calldata`. Under [EIP-7976](https://eips.ethereum.org/EIPS/eip-7976)
    /// it is raised to [`tx_token_non_zero_byte_multiplier`](Self::tx_token_non_zero_byte_multiplier)
    /// so every calldata byte contributes the same amount (`floor_tokens_in_calldata =
    /// (zero + nonzero) * 4`).
    pub fn tx_floor_token_zero_byte_multiplier(&self) -> u64 {
        self.get(GasId::tx_floor_token_zero_byte_multiplier())
    }

    /// Floor gas cost for a transaction with the given calldata.
    ///
    /// Introduced by EIP-7623 and further updated by EIP-7976. Computes
    /// `tx_floor_cost_per_token * floor_tokens_in_calldata + tx_floor_cost_base_gas`,
    /// where
    /// `floor_tokens_in_calldata = zero * tx_floor_token_zero_byte_multiplier + nonzero * tx_token_non_zero_byte_multiplier`.
    /// When the two multipliers match (EIP-7976), every byte contributes the
    /// same amount, so the zero/nonzero split is skipped and `input.len()` is
    /// used directly; otherwise (EIP-7623 path, zero multiplier = 1) the result
    /// matches `get_tokens_in_calldata(input, nonzero)`.
    #[inline]
    pub fn tx_floor_cost(&self, input: &[u8]) -> u64 {
        let zero_multiplier = self.tx_floor_token_zero_byte_multiplier();
        let non_zero_multiplier = self.tx_token_non_zero_byte_multiplier();
        let floor_tokens = if zero_multiplier == non_zero_multiplier {
            input.len() as u64 * non_zero_multiplier
        } else {
            get_tokens_in_calldata(input, non_zero_multiplier)
        };
        self.tx_floor_cost_with_tokens(floor_tokens)
    }

    /// Calculate the floor gas cost for a transaction with the given number of tokens.
    #[inline]
    pub fn tx_floor_cost_with_tokens(&self, tokens: u64) -> u64 {
        self.tx_floor_cost_per_token() * tokens + self.tx_floor_cost_base_gas()
    }

    /// Used in [GasParams::initial_tx_gas] to calculate the floor gas base gas.
    pub fn tx_floor_cost_base_gas(&self) -> u64 {
        self.get(GasId::tx_floor_cost_base_gas())
    }

    /// Used in [GasParams::initial_tx_gas] to calculate the access list address cost.
    pub fn tx_access_list_address_cost(&self) -> u64 {
        self.get(GasId::tx_access_list_address_cost())
    }

    /// Used in [GasParams::initial_tx_gas] to calculate the access list storage key cost.
    pub fn tx_access_list_storage_key_cost(&self) -> u64 {
        self.get(GasId::tx_access_list_storage_key_cost())
    }

    /// Calculate the total gas cost for an access list.
    ///
    /// This is a helper method that calculates the combined cost of:
    /// - `accounts` addresses in the access list
    /// - `storages` storage keys in the access list
    ///
    /// # Examples
    ///
    /// ```
    /// use revm_context_interface::cfg::gas_params::GasParams;
    /// use primitives::hardfork::SpecId;
    ///
    /// let gas_params = GasParams::new_spec(SpecId::BERLIN);
    /// // Calculate cost for 2 addresses and 5 storage keys
    /// let cost = gas_params.tx_access_list_cost(2, 5);
    /// assert_eq!(cost, 2 * 2400 + 5 * 1900); // 2 * ACCESS_LIST_ADDRESS + 5 * ACCESS_LIST_STORAGE_KEY
    /// ```
    #[inline]
    pub fn tx_access_list_cost(&self, accounts: u64, storages: u64) -> u64 {
        accounts
            .saturating_mul(self.tx_access_list_address_cost())
            .saturating_add(storages.saturating_mul(self.tx_access_list_storage_key_cost()))
    }

    /// Floor tokens contributed per access-list byte ([EIP-7981]).
    ///
    /// Zero before AMSTERDAM. From AMSTERDAM onward this is `4`, so each
    /// access-list byte contributes the same 64 gas to the floor as a calldata
    /// byte under EIP-7976.
    ///
    /// [EIP-7981]: https://eips.ethereum.org/EIPS/eip-7981
    #[inline]
    pub fn tx_access_list_floor_byte_multiplier(&self) -> u64 {
        self.get(GasId::tx_access_list_floor_byte_multiplier())
    }

    /// Floor tokens contributed by an access list with the given address and
    /// storage-key counts (EIP-7981). Each address is 20 bytes, each storage
    /// key is 32 bytes; tokens per byte come from
    /// [`tx_access_list_floor_byte_multiplier`](Self::tx_access_list_floor_byte_multiplier).
    #[inline]
    pub fn tx_floor_tokens_in_access_list(&self, accounts: u64, storages: u64) -> u64 {
        let bytes = accounts
            .saturating_mul(20)
            .saturating_add(storages.saturating_mul(32));
        bytes.saturating_mul(self.tx_access_list_floor_byte_multiplier())
    }

    /// Used in [GasParams::initial_tx_gas] to calculate the base transaction stipend.
    pub fn tx_base_stipend(&self) -> u64 {
        self.get(GasId::tx_base_stipend())
    }

    /// EIP-2780: regular gas cost of the EIP-7708 transfer log emitted on
    /// every nonzero-value transfer to a different account. Zero before AMSTERDAM.
    #[inline]
    pub fn tx_transfer_log_cost(&self) -> u64 {
        self.get(GasId::tx_transfer_log_cost())
    }

    /// EIP-2780/EIP-8038: regular gas cost of an account-leaf write, added
    /// when `tx.value > 0` and the recipient differs from the sender.
    /// Zero before AMSTERDAM.
    #[inline]
    pub fn tx_account_write_cost(&self) -> u64 {
        self.get(GasId::tx_account_write_cost())
    }

    /// EIP-2780/EIP-8038: regular gas cost of a top-level CREATE access,
    /// in addition to [`Self::tx_base_stipend`] and the EIP-8037 state gas.
    /// Zero before AMSTERDAM.
    #[inline]
    pub fn tx_create_access_cost(&self) -> u64 {
        self.get(GasId::tx_create_access_cost())
    }

    /// Used in [GasParams::initial_tx_gas] to calculate the create cost.
    ///
    /// Similar to the [`Self::create_cost`] method but it got activated in different fork,
    #[inline]
    pub fn tx_create_cost(&self) -> u64 {
        self.get(GasId::tx_create_cost())
    }

    /// Used in [GasParams::initial_tx_gas] to calculate the initcode cost per word of len.
    #[inline]
    pub fn tx_initcode_cost(&self, len: usize) -> u64 {
        self.get(GasId::tx_initcode_cost())
            .saturating_mul(num_words(len) as u64)
    }

    /// Initial gas that is deducted for transaction to be included.
    /// Initial gas contains initial stipend gas, gas for access list and input data.
    ///
    /// Under EIP-8037, state gas is tracked separately in `initial_state_gas`,
    /// while regular intrinsic gas accumulates in `initial_regular_gas`. The state
    /// gas components are:
    /// - EIP-7702 auth list state gas (per-auth account creation + metadata costs)
    /// - For CREATE transactions: `create_state_gas` (account creation + contract metadata)
    ///
    /// When `eip2780` is `Some`, the legacy `21,000`-style base + create-cost
    /// stipend is replaced with the EIP-2780 decomposition
    /// (`TX_BASE_COST + to-based + value-based`). Calldata, access list, and
    /// authorization-list costs are unchanged.
    ///
    /// Note: `code_deposit_state_gas` is not included since deployed code size is unknown at validation time.
    ///
    /// # Returns
    ///
    /// - Intrinsic gas (including state gas for CREATE)
    /// - Number of tokens in calldata
    #[allow(clippy::too_many_arguments)]
    pub fn initial_tx_gas(
        &self,
        input: &[u8],
        is_create: bool,
        access_list_accounts: u64,
        access_list_storages: u64,
        authorization_list_num: u64,
        eip2780: Option<Eip2780TxInfo>,
    ) -> InitialAndFloorGas {
        // Initdate stipend
        let tokens_in_calldata =
            get_tokens_in_calldata(input, self.tx_token_non_zero_byte_multiplier());

        // EIP-7702: Compute auth list costs. See
        // [`tx_eip7702_per_empty_account_cost`](Self::tx_eip7702_per_empty_account_cost)
        // for the per-auth intrinsic charge per fork.
        let auth_regular_cost = authorization_list_num * self.tx_eip7702_per_empty_account_cost();

        let base_and_to_and_value_gas = match &eip2780 {
            None => {
                let mut base = self.tx_base_stipend();
                if is_create {
                    // EIP-2: Homestead Hard-fork Changes
                    base += self.tx_create_cost();
                }
                base
            }
            Some(info) => self.eip2780_base_to_value_gas(is_create, info),
        };

        let mut initial_regular_gas = tokens_in_calldata * self.tx_token_cost()
            // before berlin tx_access_list_address_cost will be zero
            + access_list_accounts * self.tx_access_list_address_cost()
            // before berlin tx_access_list_storage_key_cost will be zero
            + access_list_storages * self.tx_access_list_storage_key_cost()
            + base_and_to_and_value_gas
            // EIP-7702: Only the regular portion of auth list cost
            + auth_regular_cost;

        if is_create {
            // EIP-3860: Limit and meter initcode
            initial_regular_gas += self.tx_initcode_cost(input.len());
        }

        // Calculate gas floor. Introduced by EIP-7623, updated by EIP-7976, and
        // extended by EIP-7981 to include access-list data alongside calldata.
        //
        // Under EIP-2780 the floor is anchored on the decomposed regular-gas
        // intrinsic base (`TX_BASE + to-based + value-based`, the same sum used
        // for `base_and_to_and_value_gas` above) rather than the flat
        // `tx_floor_cost_base_gas`, so it never undercuts the transaction's own
        // intrinsic base.
        let access_list_floor_tokens =
            self.tx_floor_tokens_in_access_list(access_list_accounts, access_list_storages);
        let mut floor_gas =
            self.tx_floor_cost(input) + access_list_floor_tokens * self.tx_floor_cost_per_token();
        if eip2780.is_some() {
            floor_gas = floor_gas - self.tx_floor_cost_base_gas() + base_and_to_and_value_gas;
        }

        // `initial_state_gas` stays zero at the intrinsic phase: state-dependent
        // charges are applied at the EIP-2780 runtime gas phase
        // (`apply_eip2780_runtime_gas`), which adds them to `initial_state_gas`.
        InitialAndFloorGas::default()
            .with_initial_regular_gas(initial_regular_gas)
            .with_floor_gas(floor_gas)
    }

    /// EIP-2780: sum of the sender base, `tx.to`-based, and `tx.value`-based
    /// regular-gas charges. Excludes calldata, access list, authorizations,
    /// and initcode/state-gas pieces which are added by the caller.
    ///
    /// Per execution-specs, a self-transfer (`tx.to == sender`) pays neither
    /// the `to`- nor `value`-based charge — only the base. Precompile
    /// recipients are charged the same as any other account (the precompile
    /// carve-out from the draft is not implemented).
    fn eip2780_base_to_value_gas(&self, is_create: bool, info: &Eip2780TxInfo) -> u64 {
        let mut gas = eip2780::TX_BASE_COST;

        if is_create {
            // tx.to charge: contract-creation access cost.
            gas += self.tx_create_access_cost();
            if !info.value.is_zero() {
                gas += self.tx_transfer_log_cost();
            }
        } else if !info.is_self_transfer {
            // tx.to charge: cold account access of the recipient.
            gas += eip8038::COLD_ACCOUNT_ACCESS;
            if !info.value.is_zero() {
                gas += self.tx_transfer_log_cost() + eip2780::TX_VALUE_COST;
            }
        }

        gas
    }

    /// Calculates the initial transaction gas directly from a [`Transaction`],
    /// deriving the access list counts from the transaction itself.
    ///
    /// See [`GasParams::initial_tx_gas`] for details on the returned gas.
    pub fn initial_tx_gas_for_tx(
        &self,
        tx: impl Transaction,
        eip2780: Option<Eip2780TxInfo>,
    ) -> InitialAndFloorGas {
        let mut accounts = 0;
        let mut storages = 0;
        // Legacy is the only tx type that does not have an access list.
        if tx.tx_type() != TransactionType::Legacy {
            (accounts, storages) = tx
                .access_list()
                .map(|al| {
                    al.fold((0, 0), |(num_accounts, num_storage_slots), item| {
                        (
                            num_accounts + 1,
                            num_storage_slots + item.storage_slots().count() as u64,
                        )
                    })
                })
                .unwrap_or_default();
        }

        self.initial_tx_gas(
            tx.input(),
            tx.kind().is_create(),
            accounts,
            storages,
            tx.authorization_list_len() as u64,
            eip2780,
        )
    }
}

/// EIP-2780 inputs to [`GasParams::initial_tx_gas`].
///
/// Carries the transferred value and whether the transaction is a
/// self-transfer (`tx.to == sender`). The decomposed intrinsic model branches
/// on `is_create` (already passed to `initial_tx_gas`), whether `tx.value` is
/// zero, and the self-transfer carve-out; see
/// `GasParams::eip2780_base_to_value_gas`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eip2780TxInfo {
    /// Transferred value.
    pub value: U256,
    /// Whether `tx.to == sender` (a `Call` to the sender's own address).
    pub is_self_transfer: bool,
}

#[inline]
pub(crate) const fn log2floor(value: U256) -> u64 {
    255u64.saturating_sub(value.leading_zeros() as u64)
}

/// Gas identifier that maps onto index in gas table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GasId(u8);

impl GasId {
    /// Creates a new `GasId` with the given id.
    #[inline]
    pub const fn new(id: u8) -> Self {
        Self(id)
    }

    /// Returns the id of the gas.
    #[inline]
    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    /// Returns the id of the gas as a usize.
    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    /// Returns the name of the gas identifier as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use revm_context_interface::cfg::gas_params::GasId;
    ///
    /// assert_eq!(GasId::exp_byte_gas().name(), "exp_byte_gas");
    /// assert_eq!(GasId::memory_linear_cost().name(), "memory_linear_cost");
    /// assert_eq!(GasId::sstore_static().name(), "sstore_static");
    /// ```
    pub const fn name(&self) -> &'static str {
        match self.0 {
            x if x == Self::exp_byte_gas().as_u8() => "exp_byte_gas",
            x if x == Self::extcodecopy_per_word().as_u8() => "extcodecopy_per_word",
            x if x == Self::copy_per_word().as_u8() => "copy_per_word",
            x if x == Self::logdata().as_u8() => "logdata",
            x if x == Self::logtopic().as_u8() => "logtopic",
            x if x == Self::mcopy_per_word().as_u8() => "mcopy_per_word",
            x if x == Self::keccak256_per_word().as_u8() => "keccak256_per_word",
            x if x == Self::memory_linear_cost().as_u8() => "memory_linear_cost",
            x if x == Self::memory_quadratic_reduction().as_u8() => "memory_quadratic_reduction",
            x if x == Self::initcode_per_word().as_u8() => "initcode_per_word",
            x if x == Self::create().as_u8() => "create",
            x if x == Self::call_stipend_reduction().as_u8() => "call_stipend_reduction",
            x if x == Self::max_refund_quotient().as_u8() => "max_refund_quotient",
            x if x == Self::transfer_value_cost().as_u8() => "transfer_value_cost",
            x if x == Self::cold_account_additional_cost().as_u8() => {
                "cold_account_additional_cost"
            }
            x if x == Self::new_account_cost().as_u8() => "new_account_cost",
            x if x == Self::warm_storage_read_cost().as_u8() => "warm_storage_read_cost",
            x if x == Self::sstore_static().as_u8() => "sstore_static",
            x if x == Self::sstore_set_without_load_cost().as_u8() => {
                "sstore_set_without_load_cost"
            }
            x if x == Self::sstore_reset_without_cold_load_cost().as_u8() => {
                "sstore_reset_without_cold_load_cost"
            }
            x if x == Self::sstore_clearing_slot_refund().as_u8() => "sstore_clearing_slot_refund",
            x if x == Self::selfdestruct_refund().as_u8() => "selfdestruct_refund",
            x if x == Self::call_stipend().as_u8() => "call_stipend",
            x if x == Self::cold_storage_additional_cost().as_u8() => {
                "cold_storage_additional_cost"
            }
            x if x == Self::cold_storage_cost().as_u8() => "cold_storage_cost",
            x if x == Self::new_account_cost_for_selfdestruct().as_u8() => {
                "new_account_cost_for_selfdestruct"
            }
            x if x == Self::code_deposit_cost().as_u8() => "code_deposit_cost",
            x if x == Self::tx_eip7702_regular_gas().as_u8() => "tx_eip7702_regular_gas",
            x if x == Self::tx_token_non_zero_byte_multiplier().as_u8() => {
                "tx_token_non_zero_byte_multiplier"
            }
            x if x == Self::tx_token_cost().as_u8() => "tx_token_cost",
            x if x == Self::tx_floor_cost_per_token().as_u8() => "tx_floor_cost_per_token",
            x if x == Self::tx_floor_cost_base_gas().as_u8() => "tx_floor_cost_base_gas",
            x if x == Self::tx_access_list_address_cost().as_u8() => "tx_access_list_address_cost",
            x if x == Self::tx_access_list_storage_key_cost().as_u8() => {
                "tx_access_list_storage_key_cost"
            }
            x if x == Self::tx_base_stipend().as_u8() => "tx_base_stipend",
            x if x == Self::tx_create_cost().as_u8() => "tx_create_cost",
            x if x == Self::tx_initcode_cost().as_u8() => "tx_initcode_cost",
            x if x == Self::sstore_set_refund().as_u8() => "sstore_set_refund",
            x if x == Self::sstore_reset_refund().as_u8() => "sstore_reset_refund",
            x if x == Self::tx_eip7702_regular_refund().as_u8() => "tx_eip7702_regular_refund",
            x if x == Self::sstore_set_state_gas().as_u8() => "sstore_set_state_gas",
            x if x == Self::new_account_state_gas().as_u8() => "new_account_state_gas",
            x if x == Self::code_deposit_state_gas().as_u8() => "code_deposit_state_gas",
            x if x == Self::create_state_gas().as_u8() => "create_state_gas",
            x if x == Self::tx_eip7702_state_gas_bytecode().as_u8() => {
                "tx_eip7702_state_gas_bytecode"
            }
            x if x == Self::tx_floor_token_zero_byte_multiplier().as_u8() => {
                "tx_floor_token_zero_byte_multiplier"
            }
            x if x == Self::tx_access_list_floor_byte_multiplier().as_u8() => {
                "tx_access_list_floor_byte_multiplier"
            }
            x if x == Self::tx_transfer_log_cost().as_u8() => "tx_transfer_log_cost",
            x if x == Self::tx_account_write_cost().as_u8() => "tx_account_write_cost",
            x if x == Self::tx_create_access_cost().as_u8() => "tx_create_access_cost",
            _ => "unknown",
        }
    }

    /// Converts a string to a `GasId`.
    ///
    /// Returns `None` if the string does not match any known gas identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use revm_context_interface::cfg::gas_params::GasId;
    ///
    /// assert_eq!(GasId::from_name("exp_byte_gas"), Some(GasId::exp_byte_gas()));
    /// assert_eq!(GasId::from_name("memory_linear_cost"), Some(GasId::memory_linear_cost()));
    /// assert_eq!(GasId::from_name("invalid_name"), None);
    /// ```
    pub fn from_name(s: &str) -> Option<GasId> {
        match s {
            "exp_byte_gas" => Some(Self::exp_byte_gas()),
            "extcodecopy_per_word" => Some(Self::extcodecopy_per_word()),
            "copy_per_word" => Some(Self::copy_per_word()),
            "logdata" => Some(Self::logdata()),
            "logtopic" => Some(Self::logtopic()),
            "mcopy_per_word" => Some(Self::mcopy_per_word()),
            "keccak256_per_word" => Some(Self::keccak256_per_word()),
            "memory_linear_cost" => Some(Self::memory_linear_cost()),
            "memory_quadratic_reduction" => Some(Self::memory_quadratic_reduction()),
            "initcode_per_word" => Some(Self::initcode_per_word()),
            "create" => Some(Self::create()),
            "call_stipend_reduction" => Some(Self::call_stipend_reduction()),
            "max_refund_quotient" => Some(Self::max_refund_quotient()),
            "transfer_value_cost" => Some(Self::transfer_value_cost()),
            "cold_account_additional_cost" => Some(Self::cold_account_additional_cost()),
            "new_account_cost" => Some(Self::new_account_cost()),
            "warm_storage_read_cost" => Some(Self::warm_storage_read_cost()),
            "sstore_static" => Some(Self::sstore_static()),
            "sstore_set_without_load_cost" => Some(Self::sstore_set_without_load_cost()),
            "sstore_reset_without_cold_load_cost" => {
                Some(Self::sstore_reset_without_cold_load_cost())
            }
            "sstore_clearing_slot_refund" => Some(Self::sstore_clearing_slot_refund()),
            "selfdestruct_refund" => Some(Self::selfdestruct_refund()),
            "call_stipend" => Some(Self::call_stipend()),
            "cold_storage_additional_cost" => Some(Self::cold_storage_additional_cost()),
            "cold_storage_cost" => Some(Self::cold_storage_cost()),
            "new_account_cost_for_selfdestruct" => Some(Self::new_account_cost_for_selfdestruct()),
            "code_deposit_cost" => Some(Self::code_deposit_cost()),
            "tx_eip7702_regular_gas" => Some(Self::tx_eip7702_regular_gas()),
            "tx_token_non_zero_byte_multiplier" => Some(Self::tx_token_non_zero_byte_multiplier()),
            "tx_token_cost" => Some(Self::tx_token_cost()),
            "tx_floor_cost_per_token" => Some(Self::tx_floor_cost_per_token()),
            "tx_floor_cost_base_gas" => Some(Self::tx_floor_cost_base_gas()),
            "tx_access_list_address_cost" => Some(Self::tx_access_list_address_cost()),
            "tx_access_list_storage_key_cost" => Some(Self::tx_access_list_storage_key_cost()),
            "tx_base_stipend" => Some(Self::tx_base_stipend()),
            "tx_create_cost" => Some(Self::tx_create_cost()),
            "tx_initcode_cost" => Some(Self::tx_initcode_cost()),
            "sstore_set_refund" => Some(Self::sstore_set_refund()),
            "sstore_reset_refund" => Some(Self::sstore_reset_refund()),
            "tx_eip7702_regular_refund" => Some(Self::tx_eip7702_regular_refund()),
            "sstore_set_state_gas" => Some(Self::sstore_set_state_gas()),
            "new_account_state_gas" => Some(Self::new_account_state_gas()),
            "code_deposit_state_gas" => Some(Self::code_deposit_state_gas()),
            "create_state_gas" => Some(Self::create_state_gas()),
            "tx_eip7702_state_gas_bytecode" => Some(Self::tx_eip7702_state_gas_bytecode()),
            "tx_floor_token_zero_byte_multiplier" => {
                Some(Self::tx_floor_token_zero_byte_multiplier())
            }
            "tx_access_list_floor_byte_multiplier" => {
                Some(Self::tx_access_list_floor_byte_multiplier())
            }
            "tx_transfer_log_cost" => Some(Self::tx_transfer_log_cost()),
            "tx_account_write_cost" => Some(Self::tx_account_write_cost()),
            "tx_create_access_cost" => Some(Self::tx_create_access_cost()),
            _ => None,
        }
    }

    /// EXP gas cost per byte
    pub const fn exp_byte_gas() -> GasId {
        Self::new(1)
    }

    /// EXTCODECOPY gas cost per word
    pub const fn extcodecopy_per_word() -> GasId {
        Self::new(2)
    }

    /// Copy copy per word
    pub const fn copy_per_word() -> GasId {
        Self::new(3)
    }

    /// Log data gas cost per byte
    pub const fn logdata() -> GasId {
        Self::new(4)
    }

    /// Log topic gas cost per topic
    pub const fn logtopic() -> GasId {
        Self::new(5)
    }

    /// MCOPY gas cost per word
    pub const fn mcopy_per_word() -> GasId {
        Self::new(6)
    }

    /// KECCAK256 gas cost per word
    pub const fn keccak256_per_word() -> GasId {
        Self::new(7)
    }

    /// Memory linear cost. Memory is additionally added as n*linear_cost.
    pub const fn memory_linear_cost() -> GasId {
        Self::new(8)
    }

    /// Memory quadratic reduction. Memory is additionally added as n*n/quadratic_reduction.
    pub const fn memory_quadratic_reduction() -> GasId {
        Self::new(9)
    }

    /// Initcode word cost
    pub const fn initcode_per_word() -> GasId {
        Self::new(10)
    }

    /// Create gas cost
    pub const fn create() -> GasId {
        Self::new(11)
    }

    /// Call stipend reduction. Call stipend is reduced by 1/64 of the gas limit.
    pub const fn call_stipend_reduction() -> GasId {
        Self::new(12)
    }

    /// Maximum gas refund quotient.
    pub const fn max_refund_quotient() -> GasId {
        Self::new(47)
    }

    /// Transfer value cost
    pub const fn transfer_value_cost() -> GasId {
        Self::new(13)
    }

    /// Additional cold cost. Additional cold cost is added to the gas cost if the account is cold loaded.
    pub const fn cold_account_additional_cost() -> GasId {
        Self::new(14)
    }

    /// New account cost. New account cost is added to the gas cost if the account is empty.
    pub const fn new_account_cost() -> GasId {
        Self::new(15)
    }

    /// Warm storage read cost. Warm storage read cost is added to the gas cost if the account is warm loaded.
    ///
    /// Used in delegated account access to specify delegated account warm gas cost.
    pub const fn warm_storage_read_cost() -> GasId {
        Self::new(16)
    }

    /// Static gas cost for SSTORE opcode. This gas in comparison with other gas const needs
    /// to be deducted after check for minimal stipend gas cost. This is a reason why it is here.
    pub const fn sstore_static() -> GasId {
        Self::new(17)
    }

    /// SSTORE set cost additional amount after SSTORE_RESET is added.
    pub const fn sstore_set_without_load_cost() -> GasId {
        Self::new(18)
    }

    /// SSTORE reset cost
    pub const fn sstore_reset_without_cold_load_cost() -> GasId {
        Self::new(19)
    }

    /// SSTORE clearing slot refund
    pub const fn sstore_clearing_slot_refund() -> GasId {
        Self::new(20)
    }

    /// Selfdestruct refund.
    pub const fn selfdestruct_refund() -> GasId {
        Self::new(21)
    }

    /// Call stipend checked in sstore.
    pub const fn call_stipend() -> GasId {
        Self::new(22)
    }

    /// Cold storage additional cost.
    pub const fn cold_storage_additional_cost() -> GasId {
        Self::new(23)
    }

    /// Cold storage cost
    pub const fn cold_storage_cost() -> GasId {
        Self::new(24)
    }

    /// New account cost for selfdestruct.
    pub const fn new_account_cost_for_selfdestruct() -> GasId {
        Self::new(25)
    }

    /// Code deposit cost. Calculated as len * code_deposit_cost.
    pub const fn code_deposit_cost() -> GasId {
        Self::new(26)
    }

    /// EIP-7702 per-auth intrinsic gas.
    ///
    /// Pre-Amsterdam this holds the pessimistic bundled `PER_EMPTY_ACCOUNT_COST`;
    /// under EIP-2780 it holds the state-independent `REGULAR_PER_AUTH_BASE_COST`
    /// only (the state-dependent remainder is charged at the runtime gas phase).
    /// Exposed via [`GasParams::tx_eip7702_per_empty_account_cost`].
    pub const fn tx_eip7702_regular_gas() -> GasId {
        Self::new(27)
    }

    /// Initial tx gas token non zero byte multiplier.
    pub const fn tx_token_non_zero_byte_multiplier() -> GasId {
        Self::new(28)
    }

    /// Initial tx gas token cost.
    pub const fn tx_token_cost() -> GasId {
        Self::new(29)
    }

    /// Initial tx gas floor cost per token.
    pub const fn tx_floor_cost_per_token() -> GasId {
        Self::new(30)
    }

    /// Initial tx gas floor cost base gas.
    pub const fn tx_floor_cost_base_gas() -> GasId {
        Self::new(31)
    }

    /// Initial tx gas access list address cost.
    pub const fn tx_access_list_address_cost() -> GasId {
        Self::new(32)
    }

    /// Initial tx gas access list storage key cost.
    pub const fn tx_access_list_storage_key_cost() -> GasId {
        Self::new(33)
    }

    /// Initial tx gas base stipend.
    pub const fn tx_base_stipend() -> GasId {
        Self::new(34)
    }

    /// Initial tx gas create cost.
    pub const fn tx_create_cost() -> GasId {
        Self::new(35)
    }

    /// Initial tx gas initcode cost per word.
    pub const fn tx_initcode_cost() -> GasId {
        Self::new(36)
    }

    /// SSTORE set refund. Used in sstore_refund for SSTORE_SET_GAS - SLOAD_GAS refund calculation.
    pub const fn sstore_set_refund() -> GasId {
        Self::new(37)
    }

    /// SSTORE reset refund. Used in sstore_refund for SSTORE_RESET_GAS - SLOAD_GAS refund calculation.
    pub const fn sstore_reset_refund() -> GasId {
        Self::new(38)
    }

    /// EIP-7702 per-auth regular-gas refund (the non-state portion).
    ///
    /// This is the refund given when an authorization is applied to an already
    /// existing account. Pre-EIP-8037 it is `PER_EMPTY_ACCOUNT_COST -
    /// PER_AUTH_BASE_COST` (25000 - 12500 = 12500); under EIP-8037 the refund is
    /// entirely state gas so this is zero. Read it through
    /// [`GasParams::tx_eip7702_auth_refund_regular`].
    pub const fn tx_eip7702_regular_refund() -> GasId {
        Self::new(39)
    }

    /// State gas for new storage slot creation (SSTORE zero → non-zero).
    pub const fn sstore_set_state_gas() -> GasId {
        Self::new(40)
    }

    /// State gas for new account creation.
    pub const fn new_account_state_gas() -> GasId {
        Self::new(41)
    }

    /// State gas per byte for code deposit.
    pub const fn code_deposit_state_gas() -> GasId {
        Self::new(42)
    }

    /// State gas for contract metadata creation.
    pub const fn create_state_gas() -> GasId {
        Self::new(43)
    }

    /// EIP-8037: State bytes for the bytecode (delegation) portion of an EIP-7702 authorization.
    /// Equals `eip8037::AUTH_BASE_BYTES * eip8037::CPSB_GLAMSTERDAM`.
    /// Zero before AMSTERDAM.
    pub const fn tx_eip7702_state_gas_bytecode() -> GasId {
        Self::new(44)
    }

    /// Multiplier for a zero byte in `floor_tokens_in_calldata`.
    ///
    /// `1` under [EIP-7623](https://eips.ethereum.org/EIPS/eip-7623) and raised
    /// to [`tx_token_non_zero_byte_multiplier`](Self::tx_token_non_zero_byte_multiplier)
    /// under [EIP-7976](https://eips.ethereum.org/EIPS/eip-7976), which makes the
    /// floor cost uniform across zero and nonzero calldata bytes. Zero before PRAGUE.
    pub const fn tx_floor_token_zero_byte_multiplier() -> GasId {
        Self::new(45)
    }

    /// Floor tokens contributed per byte of access-list data (EIP-7981).
    ///
    /// Zero before AMSTERDAM. From AMSTERDAM onward, set to `4` so every
    /// access-list byte contributes the same 16 × 4 = 64 gas as a calldata byte
    /// under EIP-7976.
    pub const fn tx_access_list_floor_byte_multiplier() -> GasId {
        Self::new(46)
    }

    /// EIP-2780: regular gas cost of the EIP-7708 transfer log emitted on every
    /// nonzero-value transfer to a different account. Zero before AMSTERDAM.
    pub const fn tx_transfer_log_cost() -> GasId {
        Self::new(48)
    }

    /// EIP-2780/EIP-8038: regular gas cost of an account-leaf write at the
    /// intrinsic level (added when `tx.value > 0` and the recipient differs
    /// from the sender). Zero before AMSTERDAM.
    pub const fn tx_account_write_cost() -> GasId {
        Self::new(49)
    }

    /// EIP-2780/EIP-8038: regular gas cost of a top-level CREATE access, in
    /// addition to [`Self::tx_base_stipend`] and the EIP-8037 state gas.
    /// Zero before AMSTERDAM.
    pub const fn tx_create_access_cost() -> GasId {
        Self::new(50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[cfg(test)]
    mod log2floor_tests {
        use super::*;

        #[test]
        fn test_log2floor_edge_cases() {
            // Test zero
            assert_eq!(log2floor(U256::ZERO), 0);

            // Test powers of 2
            assert_eq!(log2floor(U256::from(1u64)), 0); // log2(1) = 0
            assert_eq!(log2floor(U256::from(2u64)), 1); // log2(2) = 1
            assert_eq!(log2floor(U256::from(4u64)), 2); // log2(4) = 2
            assert_eq!(log2floor(U256::from(8u64)), 3); // log2(8) = 3
            assert_eq!(log2floor(U256::from(256u64)), 8); // log2(256) = 8

            // Test non-powers of 2
            assert_eq!(log2floor(U256::from(3u64)), 1); // log2(3) = 1.58... -> floor = 1
            assert_eq!(log2floor(U256::from(5u64)), 2); // log2(5) = 2.32... -> floor = 2
            assert_eq!(log2floor(U256::from(255u64)), 7); // log2(255) = 7.99... -> floor = 7

            // Test large values
            assert_eq!(log2floor(U256::from(u64::MAX)), 63);
            assert_eq!(log2floor(U256::from(u64::MAX) + U256::from(1u64)), 64);
            assert_eq!(log2floor(U256::MAX), 255);
        }
    }

    #[test]
    fn test_gas_id_name_and_from_str_coverage() {
        let mut unique_names = HashSet::new();
        let mut known_gas_ids = 0;

        // Iterate over all possible GasId values (0..256)
        for i in 0..=255 {
            let gas_id = GasId::new(i);
            let name = gas_id.name();

            // Count unique names (excluding "unknown")
            if name != "unknown" {
                unique_names.insert(name);
            }
        }

        // Now test from_str for each unique name
        for name in &unique_names {
            if let Some(gas_id) = GasId::from_name(name) {
                known_gas_ids += 1;
                // Verify round-trip: name -> GasId -> name should be consistent
                assert_eq!(gas_id.name(), *name, "Round-trip failed for {}", name);
            }
        }

        println!("Total unique named GasIds: {}", unique_names.len());
        println!("GasIds resolvable via from_str: {}", known_gas_ids);

        // All unique names should be resolvable via from_str
        assert_eq!(
            unique_names.len(),
            known_gas_ids,
            "Not all unique names are resolvable via from_str"
        );

        // We should have exactly 50 known GasIds (based on the indices 1-50 used)
        assert_eq!(
            unique_names.len(),
            50,
            "Expected 50 unique GasIds, found {}",
            unique_names.len()
        );
    }

    #[test]
    fn test_max_refund_quotient_defaults_and_override() {
        let frontier = GasParams::new_spec(SpecId::FRONTIER);
        assert_eq!(frontier.max_refund_quotient(), 2);
        assert_eq!(frontier.get(GasId::max_refund_quotient()), 2);

        let london = GasParams::new_spec(SpecId::LONDON);
        assert_eq!(london.max_refund_quotient(), 5);
        assert_eq!(
            GasId::from_name("max_refund_quotient"),
            Some(GasId::max_refund_quotient())
        );
        assert_eq!(GasId::max_refund_quotient().name(), "max_refund_quotient");

        let mut custom = london;
        custom.override_gas([(GasId::max_refund_quotient(), 10)]);
        assert_eq!(custom.max_refund_quotient(), 10);
    }

    #[test]
    fn test_tx_access_list_cost() {
        use crate::cfg::gas;

        // Test with Berlin spec (when access list was introduced)
        let gas_params = GasParams::new_spec(SpecId::BERLIN);

        // Test with 0 accounts and 0 storages
        assert_eq!(gas_params.tx_access_list_cost(0, 0), 0);

        // Test with 1 account and 0 storages
        assert_eq!(
            gas_params.tx_access_list_cost(1, 0),
            gas::ACCESS_LIST_ADDRESS
        );

        // Test with 0 accounts and 1 storage
        assert_eq!(
            gas_params.tx_access_list_cost(0, 1),
            gas::ACCESS_LIST_STORAGE_KEY
        );

        // Test with 2 accounts and 5 storages
        assert_eq!(
            gas_params.tx_access_list_cost(2, 5),
            2 * gas::ACCESS_LIST_ADDRESS + 5 * gas::ACCESS_LIST_STORAGE_KEY
        );

        // Test with large numbers to ensure no overflow
        assert_eq!(
            gas_params.tx_access_list_cost(100, 200),
            100 * gas::ACCESS_LIST_ADDRESS + 200 * gas::ACCESS_LIST_STORAGE_KEY
        );

        // Test with pre-Berlin spec (should return 0)
        let gas_params_pre_berlin = GasParams::new_spec(SpecId::ISTANBUL);
        assert_eq!(gas_params_pre_berlin.tx_access_list_cost(10, 20), 0);
    }

    #[test]
    fn test_initial_state_gas_for_create() {
        // State-dependent charges are applied at the EIP-2780 runtime gas
        // phase, so the intrinsic state gas is zero even for CREATE
        // transactions at AMSTERDAM.
        let gas_params = GasParams::new_spec(SpecId::AMSTERDAM);
        // Test CREATE transaction (is_create = true)
        let create_gas = gas_params.initial_tx_gas(b"", true, 0, 0, 0, None);
        assert_eq!(create_gas.initial_state_gas_final(), 0);

        let create_cost = gas_params.tx_create_cost();
        let initcode_cost = gas_params.tx_initcode_cost(0);
        assert_eq!(
            create_gas.initial_total_gas(),
            gas_params.tx_base_stipend() + create_cost + initcode_cost
        );

        // Test CALL transaction (is_create = false)
        let call_gas = gas_params.initial_tx_gas(b"", false, 0, 0, 0, None);
        assert_eq!(call_gas.initial_state_gas_final(), 0);
        // initial_gas should be unchanged for calls
        assert_eq!(call_gas.initial_total_gas(), gas_params.tx_base_stipend());
    }

    #[test]
    fn test_initial_tx_gas_eip2780_runtime_split() {
        let gas_params = GasParams::new_spec(SpecId::AMSTERDAM);
        let info = || Eip2780TxInfo {
            value: U256::ZERO,
            is_self_transfer: false,
        };

        // Create transaction: the new-account state gas is no longer intrinsic —
        // it moves to the runtime phase, charged only when the deployment
        // target does not already exist.
        let create_gas = gas_params.initial_tx_gas(b"", true, 0, 0, 0, Some(info()));
        assert_eq!(create_gas.initial_state_gas, 0);
        assert_eq!(
            create_gas.initial_regular_gas,
            eip2780::TX_BASE_COST + eip8038::CREATE_ACCESS
        );

        // EIP-7702 authorizations: intrinsic per-auth charge is the
        // state-independent REGULAR_PER_AUTH_BASE_COST (7,816) only; the
        // ACCOUNT_WRITE and state-gas portions are runtime charges.
        assert_eq!(
            gas_params.tx_eip7702_per_empty_account_cost(),
            eip8038::EIP7702_PER_AUTH_BASE_REGULAR
        );
        let auth_gas = gas_params.initial_tx_gas(b"", false, 0, 0, 2, Some(info()));
        assert_eq!(auth_gas.initial_state_gas, 0);
        assert_eq!(
            auth_gas.initial_regular_gas,
            eip2780::TX_BASE_COST
                + eip8038::COLD_ACCOUNT_ACCESS
                + 2 * eip8038::EIP7702_PER_AUTH_BASE_REGULAR
        );

        // Pre-Amsterdam the per-auth charge is the bundled pessimistic
        // PER_EMPTY_ACCOUNT_COST (25,000) and the intrinsic state gas is zero.
        let legacy_params = GasParams::new_spec(SpecId::PRAGUE);
        assert_eq!(
            legacy_params.tx_eip7702_per_empty_account_cost(),
            eip7702::PER_EMPTY_ACCOUNT_COST
        );
        let legacy_auth_gas = legacy_params.initial_tx_gas(b"", false, 0, 0, 1, None);
        assert_eq!(legacy_auth_gas.initial_state_gas, 0);
        assert_eq!(
            legacy_auth_gas.initial_regular_gas,
            legacy_params.tx_base_stipend() + eip7702::PER_EMPTY_ACCOUNT_COST
        );
        let legacy_create_gas = legacy_params.initial_tx_gas(b"", true, 0, 0, 0, None);
        assert_eq!(legacy_create_gas.initial_state_gas, 0);
    }

    #[test]
    fn test_eip7981_access_list_cost_amsterdam() {
        // EIP-7981 folds a 64 gas/byte data charge into the per-item access-list cost
        // and adds 4 floor tokens per access-list byte on top of the EIP-7976 floor.
        // EIP-8038 sets the per-item base to COLD_ACCOUNT_ACCESS / COLD_STORAGE_ACCESS
        // (both 3,000).
        let params = GasParams::new_spec(SpecId::AMSTERDAM);

        // Per-item intrinsic cost: base + bytes * 64
        assert_eq!(params.tx_access_list_address_cost(), 3000 + 20 * 64);
        assert_eq!(params.tx_access_list_storage_key_cost(), 3000 + 32 * 64);
        assert_eq!(params.tx_access_list_cost(1, 0), 3000 + 20 * 64);
        assert_eq!(params.tx_access_list_cost(0, 1), 3000 + 32 * 64);

        // Floor multiplier activates at AMSTERDAM.
        assert_eq!(params.tx_access_list_floor_byte_multiplier(), 4);
        // 2 addresses (40 bytes) + 3 keys (96 bytes) = 136 bytes => 544 floor tokens.
        assert_eq!(params.tx_floor_tokens_in_access_list(2, 3), (40 + 96) * 4);

        // Floor gas includes both calldata (empty here) and access-list contribution.
        let gas = params.initial_tx_gas(b"", false, 2, 3, 0, None);
        let expected_al_floor = (40 + 96) * 4 * params.tx_floor_cost_per_token();
        assert_eq!(
            gas.floor_gas(),
            params.tx_floor_cost_base_gas() + expected_al_floor,
        );

        // Pre-AMSTERDAM the access-list floor contribution is zero.
        let prague = GasParams::new_spec(SpecId::PRAGUE);
        assert_eq!(prague.tx_access_list_floor_byte_multiplier(), 0);
        assert_eq!(prague.tx_floor_tokens_in_access_list(2, 3), 0);
        let prague_gas = prague.initial_tx_gas(b"", false, 2, 3, 0, None);
        assert_eq!(prague_gas.floor_gas(), prague.tx_floor_cost_base_gas());
    }
}
