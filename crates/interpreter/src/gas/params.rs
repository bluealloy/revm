//! Gas table for dynamic gas constants.

use crate::{
    gas::{self, log2floor, ISTANBUL_SLOAD_GAS, SSTORE_RESET, SSTORE_SET, WARM_SSTORE_RESET},
    num_words,
};
use context_interface::context::SStoreResult;
use primitives::{
    hardfork::SpecId::{self},
    U256,
};
use std::sync::Arc;

/// Gas table for dynamic gas constants.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GasParams {
    /// Table of gas costs for operations
    table: Arc<[u64; 256]>,
    /// Pointer to the table.
    ptr: *const u64,
}

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
    fn default() -> Self {
        Self::new_spec(SpecId::default())
    }
}

impl GasParams {
    /// Creates a new `GasParams` with the given table.
    #[inline]
    pub fn new(table: Arc<[u64; 256]>) -> Self {
        Self {
            ptr: table.as_ptr(),
            table,
        }
    }

    /// Overrides the gas cost for the given gas id.
    ///
    /// It will clone underlying table and override the values.
    ///
    /// Use to override default gas cost
    ///
    /// ```rust
    /// use revm_interpreter::gas::params::{GasParams, GasId};
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
    #[inline]
    pub fn new_spec(spec: SpecId) -> Self {
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
        table[GasId::transfer_value_cost().as_usize()] = gas::CALLVALUE;
        table[GasId::cold_account_additional_cost().as_usize()] = 0;
        table[GasId::new_account_cost().as_usize()] = gas::NEWACCOUNT;
        table[GasId::warm_storage_read_cost().as_usize()] = 0;
        // Frontiers had fixed 5k cost.
        table[GasId::sstore_static().as_usize()] = SSTORE_RESET;
        // SSTORE SET
        table[GasId::sstore_set_without_load_cost().as_usize()] = SSTORE_SET - SSTORE_RESET;
        // SSTORE RESET Is covered in SSTORE_STATIC.
        table[GasId::sstore_reset_without_cold_load_cost().as_usize()] = 0;
        // SSTORE CLEARING SLOT REFUND
        table[GasId::sstore_clearing_slot_refund().as_usize()] = 15000;
        table[GasId::selfdestruct_refund().as_usize()] = 24000;
        table[GasId::call_stipend().as_usize()] = gas::CALL_STIPEND;
        table[GasId::cold_storage_additional_cost().as_usize()] = 0;
        table[GasId::cold_storage_cost().as_usize()] = 0;
        table[GasId::new_account_cost_for_selfdestruct().as_usize()] = 0;

        if spec.is_enabled_in(SpecId::TANGERINE) {
            table[GasId::new_account_cost_for_selfdestruct().as_usize()] = gas::NEWACCOUNT;
        }

        if spec.is_enabled_in(SpecId::SPURIOUS_DRAGON) {
            table[GasId::exp_byte_gas().as_usize()] = 50;
        }

        if spec.is_enabled_in(SpecId::ISTANBUL) {
            table[GasId::sstore_static().as_usize()] = gas::ISTANBUL_SLOAD_GAS;
            table[GasId::sstore_set_without_load_cost().as_usize()] =
                SSTORE_SET - ISTANBUL_SLOAD_GAS;
            table[GasId::sstore_reset_without_cold_load_cost().as_usize()] =
                SSTORE_RESET - ISTANBUL_SLOAD_GAS;
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
                WARM_SSTORE_RESET - gas::WARM_STORAGE_READ_COST;
            table[GasId::sstore_set_without_load_cost().as_usize()] =
                SSTORE_SET - gas::WARM_STORAGE_READ_COST;
        }

        if spec.is_enabled_in(SpecId::LONDON) {
            // EIP-3529: Reduction in refunds

            // Replace SSTORE_CLEARS_SCHEDULE (as defined in EIP-2200) with
            // SSTORE_RESET_GAS + ACCESS_LIST_STORAGE_KEY_COST (4,800 gas as of EIP-2929 + EIP-2930)
            table[GasId::sstore_clearing_slot_refund().as_usize()] =
                WARM_SSTORE_RESET + gas::ACCESS_LIST_STORAGE_KEY;

            table[GasId::selfdestruct_refund().as_usize()] = 0;
        }

        Self::new(Arc::new(table))
    }

    /// Gets the gas cost for the given gas id.
    #[inline]
    pub const fn get(&self, id: GasId) -> u64 {
        unsafe { *self.ptr.add(id.as_usize()) }
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
                refund += self.sstore_set_without_load_cost() as i64;
            // Otherwise
            } else {
                // add SSTORE_RESET_GAS - SLOAD_GAS gas to refund counter.
                refund += self.sstore_reset_without_cold_load_cost() as i64;
            }
        }
        refund
    }

    /// `LOG` opcode cost calculation.
    #[inline]
    pub const fn log_cost(&self, n: u8, len: u64) -> u64 {
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
}

/// Gas identifier that maps onto index in gas table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GasId(u8);

impl GasId {
    /// Creates a new `GasId` with the given id.
    pub const fn new(id: u8) -> Self {
        Self(id)
    }

    /// Returns the id of the gas.
    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    /// Returns the id of the gas as a usize.
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    /// Returns the name of the gas identifier as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use revm_interpreter::gas::params::GasId;
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
    /// use revm_interpreter::gas::params::GasId;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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

        // We should have exactly 25 known GasIds (based on the indices 1-25 used)
        assert_eq!(
            unique_names.len(),
            25,
            "Expected 25 unique GasIds, found {}",
            unique_names.len()
        );
    }
}
