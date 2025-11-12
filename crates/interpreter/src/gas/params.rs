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
    /// SPEC ID
    spec: SpecId,
}

#[cfg(feature = "serde")]
mod serde {
    use primitives::hardfork::SpecId;

    use super::{Arc, GasParams};

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
            Ok(Self::new(
                SpecId::default(),
                Arc::new(table.table.try_into().unwrap()),
            ))
        }
    }
}

impl Default for GasParams {
    fn default() -> Self {
        let table = Arc::new([0; 256]);
        Self::new(SpecId::default(), table)
    }
}

impl GasParams {
    // Constants ids

    /// EXP gas cost per byte
    pub const EXP_BYTE_GAS: GasId = 1;
    /// EXTCODECOPY gas cost per word
    pub const EXTCODECOPY_PER_WORD: GasId = 2;
    /// Copy copy per word
    pub const COPY_PER_WORD: GasId = 3;
    /// Log data gas cost per byte
    pub const LOGDATA: GasId = 4;
    /// Log topic gas cost per topic
    pub const LOGTOPIC: GasId = 5;
    /// MCOPY gas cost per word
    pub const MCOPY_PER_WORD: GasId = 6;
    /// KECCAK256 gas cost per word
    pub const KECCAK256_PER_WORD: GasId = 7;
    /// Memory linear cost. Memory is additionally added as n*linear_cost.
    pub const MEMORY_LINEAR_COST: GasId = 8;
    /// Memory quadratic reduction. Memory is additionally added as n*n/quadratic_reduction.
    pub const MEMORY_QUADRATIC_REDUCTION: GasId = 9;
    /// Initcode word cost
    pub const INITCODE_PER_WORD: GasId = 10;
    /// Create gas cost
    pub const CREATE: GasId = 11;
    /// Call stipend reduction. Call stipend is reduced by 1/64 of the gas limit.
    pub const CALL_STIPEND_REDUCTION: GasId = 12;
    /// Transafer value cost
    pub const TRANSFER_VALUE_COST: GasId = 13;
    /// Additional cold cost. Additional cold cost is added to the gas cost if the account is cold loaded.
    pub const COLD_ACCOUNT_ADDITIONAL_COST: GasId = 14;
    /// New account cost. New account cost is added to the gas cost if the account is empty.
    pub const NEW_ACCOUNT_COST: GasId = 15;
    /// Warm storage read cost. Warm storage read cost is added to the gas cost if the account is warm loaded.
    ///
    /// Used in delegated account access to specify delegated account warm gas cost.
    pub const WARM_STORAGE_READ_COST: GasId = 16;
    /// Static gas cost for SSTORE opcode. This gas in comparison with other gas const needs
    /// to be deducted after check for minimal stipend gas cost. This is a reason why it is here.
    pub const SSTORE_STATIC: GasId = 17;
    /// SSTORE set cost additional amount after SSTORE_RESET is added.
    pub const SSTORE_SET_WITHOUT_LOAD_COST: GasId = 18;
    /// SSTORE reset cost
    pub const SSTORE_RESET_WITHOUT_COLD_LOAD_COST: GasId = 19;
    /// SSTORE clearing slot refund
    pub const SSTORE_CLEARING_SLOT_REFUND: GasId = 20;
    /// Selfdestruct refund.
    pub const SELFDESTRUCT_REFUND: GasId = 21;
    /// Call stipend checked in sstore.
    pub const CALL_STIPEND: GasId = 22;
    /// Cold storage additional cost.
    pub const COLD_STORAGE_ADDITIONAL_COST: GasId = 23;
    /// Colst storage cost
    pub const COLD_STORAGE_COST: GasId = 24;
    /// New account cost for selfdestruct.
    pub const NEW_ACCOUNT_COST_FOR_SELFDESTRUCT: GasId = 25;

    /// Creates a new `GasParams` with the given table.
    #[inline]
    pub fn new(spec: SpecId, table: Arc<[u64; 256]>) -> Self {
        Self {
            ptr: table.as_ptr(),
            table,
            spec,
        }
    }

    /// Returns the spec id.
    #[inline]
    pub fn spec(&self) -> SpecId {
        self.spec
    }

    // /// Overrides the gas cost for the given gas id.
    // ///
    // /// Use to override default gas cost
    // ///
    // /// ```rust
    // /// let mut gas_table = GasParams::new_spec(SpecId::default());
    // /// gas_table.override_gas([(GasParams::MEMORY_LINEAR_COST, 2), (GasParams::MEMORY_QUADRATIC_REDUCTION, 512)].into_iter());
    // /// assert_eq!(gas_table.get(GasParams::MEMORY_LINEAR_COST), 2);
    // /// assert_eq!(gas_table.get(GasParams::MEMORY_QUADRATIC_REDUCTION), 512);
    // /// ```
    // pub fn override_gas(&mut self, values: impl IntoIterator<Item = (GasId, u64)>) {
    //     let mut table = self.table.as_ref().clone();
    //     for (id, value) in values.into_iter() {
    //         table[id as usize] = value;
    //     }
    //     *self = Self::new(Arc::new(table));
    // }

    /// Returns the table.
    #[inline]
    pub fn table(&self) -> &[u64; 256] {
        &self.table
    }

    /// Creates a new `GasParams` for the given spec.
    #[inline]
    pub fn new_spec(spec: SpecId) -> Self {
        let mut table = [0; 256];

        table[Self::EXP_BYTE_GAS as usize] = 10;
        table[Self::LOGDATA as usize] = gas::LOGDATA;
        table[Self::LOGTOPIC as usize] = gas::LOGTOPIC;
        table[Self::COPY_PER_WORD as usize] = gas::COPY;
        table[Self::EXTCODECOPY_PER_WORD as usize] = gas::COPY;
        table[Self::MCOPY_PER_WORD as usize] = gas::COPY;
        table[Self::KECCAK256_PER_WORD as usize] = gas::KECCAK256WORD;
        table[Self::MEMORY_LINEAR_COST as usize] = gas::MEMORY;
        table[Self::MEMORY_QUADRATIC_REDUCTION as usize] = 512;
        table[Self::INITCODE_PER_WORD as usize] = gas::INITCODE_WORD_COST;
        table[Self::CREATE as usize] = gas::CREATE;
        table[Self::CALL_STIPEND_REDUCTION as usize] = 64;
        table[Self::TRANSFER_VALUE_COST as usize] = gas::CALLVALUE;
        table[Self::COLD_ACCOUNT_ADDITIONAL_COST as usize] = 0;
        table[Self::NEW_ACCOUNT_COST as usize] = gas::NEWACCOUNT;
        table[Self::WARM_STORAGE_READ_COST as usize] = 0;
        // Frontiers had fixed 5k cost.
        table[Self::SSTORE_STATIC as usize] = 5000;
        // SSTORE SET
        table[Self::SSTORE_SET_WITHOUT_LOAD_COST as usize] = SSTORE_SET - 5000;
        // SSTORE RESET Is covered in SSTORE_STATIC.
        table[Self::SSTORE_RESET_WITHOUT_COLD_LOAD_COST as usize] = 0;
        // SSTORE CLEARING SLOT REFUND
        table[Self::SSTORE_CLEARING_SLOT_REFUND as usize] = 15000;
        table[Self::SELFDESTRUCT_REFUND as usize] = 24000;
        table[Self::CALL_STIPEND as usize] = gas::CALL_STIPEND;
        table[Self::COLD_STORAGE_ADDITIONAL_COST as usize] = 0;
        table[Self::COLD_STORAGE_COST as usize] = 0;
        table[Self::NEW_ACCOUNT_COST_FOR_SELFDESTRUCT as usize] = 0;

        if spec.is_enabled_in(SpecId::TANGERINE) {
            table[Self::NEW_ACCOUNT_COST_FOR_SELFDESTRUCT as usize] = gas::NEWACCOUNT;
        }

        if spec.is_enabled_in(SpecId::SPURIOUS_DRAGON) {
            table[Self::EXP_BYTE_GAS as usize] = 50;
        }

        if spec.is_enabled_in(SpecId::ISTANBUL) {
            table[Self::SSTORE_STATIC as usize] = gas::ISTANBUL_SLOAD_GAS;
            table[Self::SSTORE_SET_WITHOUT_LOAD_COST as usize] = SSTORE_SET - ISTANBUL_SLOAD_GAS;
            table[Self::SSTORE_RESET_WITHOUT_COLD_LOAD_COST as usize] =
                SSTORE_RESET - ISTANBUL_SLOAD_GAS;
        }

        if spec.is_enabled_in(SpecId::BERLIN) {
            table[Self::SSTORE_STATIC as usize] = gas::WARM_STORAGE_READ_COST;
            table[Self::COLD_ACCOUNT_ADDITIONAL_COST as usize] =
                gas::COLD_ACCOUNT_ACCESS_COST_ADDITIONAL;
            table[Self::COLD_STORAGE_ADDITIONAL_COST as usize] =
                gas::COLD_SLOAD_COST - gas::WARM_STORAGE_READ_COST;
            table[Self::COLD_STORAGE_COST as usize] = gas::COLD_SLOAD_COST;
            table[Self::WARM_STORAGE_READ_COST as usize] = gas::WARM_STORAGE_READ_COST;

            table[Self::SSTORE_RESET_WITHOUT_COLD_LOAD_COST as usize] =
                WARM_SSTORE_RESET - gas::WARM_STORAGE_READ_COST;
            table[Self::SSTORE_SET_WITHOUT_LOAD_COST as usize] =
                SSTORE_SET - gas::WARM_STORAGE_READ_COST;
        }

        if spec.is_enabled_in(SpecId::LONDON) {
            // EIP-3529: Reduction in refunds
            // Replace SSTORE_CLEARS_SCHEDULE (as defined in EIP-2200) with
            // SSTORE_RESET_GAS + ACCESS_LIST_STORAGE_KEY_COST (4,800 gas as of EIP-2929 + EIP-2930)
            table[Self::SSTORE_CLEARING_SLOT_REFUND as usize] =
                WARM_SSTORE_RESET + gas::ACCESS_LIST_STORAGE_KEY;

            // EIP-3529: Reduction in refunds
            table[Self::SELFDESTRUCT_REFUND as usize] = 0;
        }

        Self::new(spec, Arc::new(table))
    }

    /// Gets the gas cost for the given gas id.
    #[inline]
    pub const fn get(&self, id: GasId) -> u64 {
        unsafe { *self.ptr.add(id as usize) }
    }

    /// `EXP` opcode cost calculation.
    #[inline]
    pub fn exp_cost(&self, power: U256) -> u64 {
        if power.is_zero() {
            return 0;
        }
        // EIP-160: EXP cost increase
        self.get(Self::EXP_BYTE_GAS)
            .saturating_mul(log2floor(power) / 8 + 1)
    }

    /// Selfdestruct refund.
    #[inline]
    pub fn selfdestruct_refund(&self) -> i64 {
        self.get(Self::SELFDESTRUCT_REFUND) as i64
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
            gas += self.cold_account_additional_cost() + self.warm_storage_read_cost();
        }
        gas
    }

    /// EXTCODECOPY gas cost
    #[inline]
    pub fn extcodecopy(&self, len: usize) -> u64 {
        self.get(Self::EXTCODECOPY_PER_WORD)
            .saturating_mul(num_words(len) as u64)
    }

    /// MCOPY gas cost
    #[inline]
    pub fn mcopy_cost(&self, len: usize) -> u64 {
        self.get(Self::MCOPY_PER_WORD)
            .saturating_mul(num_words(len) as u64)
    }

    /// Static gas cost for SSTORE opcode
    #[inline]
    pub fn sstore_static_gas(&self) -> u64 {
        self.get(Self::SSTORE_STATIC)
    }

    /// SSTORE set cost
    #[inline]
    pub fn sstore_set_without_load_cost(&self) -> u64 {
        self.get(Self::SSTORE_SET_WITHOUT_LOAD_COST)
    }

    /// SSTORE reset cost
    #[inline]
    pub fn sstore_reset_without_cold_load_cost(&self) -> u64 {
        self.get(Self::SSTORE_RESET_WITHOUT_COLD_LOAD_COST)
    }

    /// SSTORE clearing slot refund
    #[inline]
    pub fn sstore_clearing_slot_refund(&self) -> u64 {
        self.get(Self::SSTORE_CLEARING_SLOT_REFUND)
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
            // // before instanbul fork, refund was always awarded without checking original state.
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
        self.get(Self::LOGDATA)
            .saturating_mul(len)
            .saturating_add(self.get(Self::LOGTOPIC) * n as u64)
    }

    /// KECCAK256 gas cost per word
    #[inline]
    pub fn keccak256_cost(&self, len: usize) -> u64 {
        self.get(Self::KECCAK256_PER_WORD)
            .saturating_mul(num_words(len) as u64)
    }

    /// Memory gas cost
    #[inline]
    pub fn memory_cost(&self, len: usize) -> u64 {
        let len = len as u64;
        self.get(Self::MEMORY_LINEAR_COST)
            .saturating_mul(len)
            .saturating_add(
                (len.saturating_mul(len))
                    .saturating_div(self.get(Self::MEMORY_QUADRATIC_REDUCTION)),
            )
    }

    /// Initcode word cost
    #[inline]
    pub fn initcode_cost(&self, len: usize) -> u64 {
        self.get(Self::INITCODE_PER_WORD)
            .saturating_mul(num_words(len) as u64)
    }

    /// Create gas cost
    #[inline]
    pub fn create_cost(&self) -> u64 {
        self.get(Self::CREATE)
    }

    /// Create2 gas cost.
    #[inline]
    pub fn create2_cost(&self, len: usize) -> u64 {
        self.get(Self::CREATE).saturating_add(
            self.get(Self::KECCAK256_PER_WORD)
                .saturating_mul(num_words(len) as u64),
        )
    }

    /// Call stipend.
    #[inline]
    pub fn call_stipend(&self) -> u64 {
        self.get(Self::CALL_STIPEND)
    }

    /// Call stipend reduction. Call stipend is reduced by 1/64 of the gas limit.
    #[inline]
    pub fn call_stipend_reduction(&self, gas_limit: u64) -> u64 {
        gas_limit - gas_limit / self.get(Self::CALL_STIPEND_REDUCTION)
    }

    /// Transfer value cost
    #[inline]
    pub fn transfer_value_cost(&self) -> u64 {
        self.get(Self::TRANSFER_VALUE_COST)
    }

    /// Additional cold cost. Additional cold cost is added to the gas cost if the account is cold loaded.
    #[inline]
    pub fn cold_account_additional_cost(&self) -> u64 {
        self.get(Self::COLD_ACCOUNT_ADDITIONAL_COST)
    }

    /// Cold storage additional cost.
    #[inline]
    pub fn cold_storage_additional_cost(&self) -> u64 {
        self.get(Self::COLD_STORAGE_ADDITIONAL_COST)
    }

    /// Cold storage cost.
    #[inline]
    pub fn cold_storage_cost(&self) -> u64 {
        self.get(Self::COLD_STORAGE_COST)
    }

    /// New account cost. New account cost is added to the gas cost if the account is empty.
    #[inline]
    pub fn new_account_cost(&self, is_spurious_dragon: bool, transfers_value: bool) -> u64 {
        // EIP-161: State trie clearing (invariant-preserving alternative)
        // Pre-Spurious Dragon: always charge for new account
        // Post-Spurious Dragon: only charge if value is transferred
        if !is_spurious_dragon || transfers_value {
            return self.get(Self::NEW_ACCOUNT_COST);
        }
        0
    }

    /// New account cost for selfdestruct.
    #[inline]
    pub fn new_account_cost_for_selfdestruct(&self) -> u64 {
        self.get(Self::NEW_ACCOUNT_COST_FOR_SELFDESTRUCT)
    }

    /// Warm storage read cost. Warm storage read cost is added to the gas cost if the account is warm loaded.
    #[inline]
    pub fn warm_storage_read_cost(&self) -> u64 {
        self.get(Self::WARM_STORAGE_READ_COST)
    }

    /// Copy cost
    #[inline]
    pub fn copy_cost(&self, len: usize) -> u64 {
        self.copy_per_word_cost(num_words(len))
    }

    /// Copy per word cost
    #[inline]
    pub fn copy_per_word_cost(&self, word_num: usize) -> u64 {
        self.get(Self::COPY_PER_WORD)
            .saturating_mul(word_num as u64)
    }
}

/// Gas identifier
pub type GasId = u8;
