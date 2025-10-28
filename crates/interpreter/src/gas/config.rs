//! Gas table for dynamic gas constants.

use crate::{
    gas::{
        self, log2floor, COLD_SLOAD_COST, ISTANBUL_SLOAD_GAS, SSTORE_RESET, SSTORE_SET,
        WARM_SSTORE_RESET, WARM_STORAGE_READ_COST,
    },
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
    // TODO should we have spec or not.
}

#[cfg(feature = "serde")]
mod serde {
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
            Ok(Self::new(Arc::new(table.table.try_into().unwrap())))
        }
    }
}

impl Default for GasParams {
    fn default() -> Self {
        let table = Arc::new([0; 256]);
        Self::new(table)
    }
}

impl GasParams {
    /// Constants ids

    /// EXP gas cost per byte
    pub const EXP_BYTE_GAS: GasId = 1;
    /// EXTCODECOPY gas cost per word
    pub const EXTCODECOPY_PER_WORD: GasId = 2;
    /// Static gas cost for SSTORE opcode. This gas in comparison with other gas const needs
    /// to be deducted after check for minimal stipend gas check. This is a reason why it is here.
    pub const SSTORE: GasId = 3;
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
    pub const ADDITIONAL_COLD_COST: GasId = 14;
    /// New account cost. New account cost is added to the gas cost if the account is empty.
    pub const NEW_ACCOUNT_COST: GasId = 15;
    /// Warm storage read cost. Warm storage read cost is added to the gas cost if the account is warm loaded.
    ///
    /// Used in delegated account access to specify delegated account warm gas cost.
    pub const WARM_STORAGE_READ_COST: GasId = 16;
    /// Copy copy per word
    pub const COPY_PER_WORD: GasId = 17;
    /// SSTORE set cost
    pub const SSTORE_SET: GasId = 18;
    /// SSTORE reset cost
    pub const SSTORE_RESET: GasId = 19;

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
    /// Use to override default gas cost
    ///
    /// ```rust
    /// let mut gas_table = GasParams::new_spec(SpecId::default());
    /// gas_table.override_gas([(GasParams::MEMORY_LINEAR_COST, 2), (GasParams::MEMORY_QUADRATIC_REDUCTION, 512)].into_iter());
    /// assert_eq!(gas_table.get(GasParams::MEMORY_LINEAR_COST), 2);
    /// assert_eq!(gas_table.get(GasParams::MEMORY_QUADRATIC_REDUCTION), 512);
    /// ```
    pub fn override_gas(&mut self, values: impl IntoIterator<Item = (GasId, u64)>) {
        let mut table = self.table.as_ref().clone();
        for (id, value) in values.into_iter() {
            table[id as usize] = value;
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

        table[Self::SSTORE as usize] = gas::SSTORE_RESET;
        table[Self::EXP_BYTE_GAS as usize] = 10;
        table[Self::LOGDATA as usize] = gas::LOGDATA;
        table[Self::LOGTOPIC as usize] = gas::LOGTOPIC;
        table[Self::EXTCODECOPY_PER_WORD as usize] = gas::COPY;
        table[Self::MCOPY_PER_WORD as usize] = gas::COPY;
        table[Self::KECCAK256_PER_WORD as usize] = gas::KECCAK256WORD;
        table[Self::MEMORY_LINEAR_COST as usize] = gas::MEMORY;
        table[Self::MEMORY_QUADRATIC_REDUCTION as usize] = 512;
        table[Self::INITCODE_PER_WORD as usize] = gas::INITCODE_WORD_COST;
        table[Self::CREATE as usize] = gas::CREATE;
        table[Self::CALL_STIPEND_REDUCTION as usize] = 64;
        table[Self::TRANSFER_VALUE_COST as usize] = gas::CALLVALUE;
        table[Self::ADDITIONAL_COLD_COST as usize] = 0;
        table[Self::NEW_ACCOUNT_COST as usize] = gas::NEWACCOUNT;
        table[Self::WARM_STORAGE_READ_COST as usize] = gas::WARM_STORAGE_READ_COST;
        table[Self::COPY_PER_WORD as usize] = gas::COPY;
        table[Self::SSTORE_SET as usize] = SSTORE_SET - SSTORE_RESET;
        // resset is not used in frontier fork.
        table[Self::SSTORE_RESET as usize] = 0;

        if spec.is_enabled_in(SpecId::SPURIOUS_DRAGON) {
            table[Self::EXP_BYTE_GAS as usize] = 50;
        }

        if spec.is_enabled_in(SpecId::ISTANBUL) {
            table[Self::SSTORE as usize] = gas::ISTANBUL_SLOAD_GAS;
            table[Self::SSTORE_SET as usize] = SSTORE_SET - ISTANBUL_SLOAD_GAS;
            table[Self::SSTORE_RESET as usize] = SSTORE_RESET - ISTANBUL_SLOAD_GAS;
        }

        if spec.is_enabled_in(SpecId::BERLIN) {
            table[Self::SSTORE as usize] = gas::WARM_STORAGE_READ_COST;
            table[Self::ADDITIONAL_COLD_COST as usize] = gas::COLD_ACCOUNT_ACCESS_COST_ADDITIONAL;
            table[Self::SSTORE_SET as usize] = SSTORE_SET - gas::WARM_STORAGE_READ_COST;
            table[Self::SSTORE_RESET as usize] = SSTORE_RESET - gas::WARM_STORAGE_READ_COST;
        }

        Self::new(Arc::new(table))
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
        self.get(Self::SSTORE)
    }

    /// SSTORE set cost
    #[inline]
    pub fn sstore_set_cost(&self) -> u64 {
        self.get(Self::SSTORE_SET)
    }

    /// SSTORE reset cost
    #[inline]
    pub fn sstore_reset_cost(&self) -> u64 {
        self.get(Self::SSTORE_RESET)
    }

    /// Dynamic gas cost for SSTORE opcode
    #[inline]
    pub fn sstore_dynamic_gas(&self, is_istanbul: bool, vals: &SStoreResult, is_cold: bool) -> u64 {
        let mut gas = 0;

        // this will be zero before berlin fork.
        if is_cold {
            gas += self.additional_cold_cost();
        }

        if vals.new_values_changes_present() {
            let sstore_set = self.sstore_set_cost();
            if is_istanbul {
                if vals.is_original_eq_present() {
                    let sstore_reset = self.sstore_reset_cost();
                    // cost for changing storage slot (called in EIP as reset gas)
                    gas += sstore_reset;
                    if vals.is_original_zero() {
                        // add additional gas for creating storage slot (Zero slot means it is not existing).
                        gas += sstore_set - sstore_reset;
                    }
                }
            } else if vals.is_original_zero() {
                // frontier logic gets charged for every SSTORE operation if original value is zero.
                // this behaviour is fixed in istanbul fork.
                gas = sstore_set;
            }
        }
        gas
    }

    /// SSTORE refund calculation.
    #[inline]
    pub fn sstore_refund(&self, before_istanbul: bool, vals: &SStoreResult) -> i64 {
        // EIP-3529: Reduction in refunds
        let sstore_clears_schedule = 0;
        let sstore_set = self.sstore_set_cost() as i64;
        let sstore_reset = self.sstore_reset_cost() as i64;

        if before_istanbul {
            if !vals.is_present_zero() && vals.is_new_zero() {
                return sstore_clears_schedule;
            }
            return 0;
        }

        // If current value equals new value (this is a no-op), SLOAD_GAS is deducted.
        if vals.is_new_eq_present() {
            return 0;
        }

        // refund for the clearing of storage slot.
        // As new is not equal to present, new values zero means that original and present values are not zero
        if vals.is_original_eq_present() && vals.is_new_zero() {
            return sstore_clears_schedule;
        }

        let mut refund = 0;
        if !vals.is_original_zero() {
            // if present is zero (and new is not zero), we are removing the refund
            // that was previously added for clearing the storage slot.
            if vals.is_present_zero() {
                refund -= sstore_clears_schedule;
            } else if vals.is_new_zero() {
                // if new is zero (and present is not zero), we are adding the refund for clearing the storage slot.
                refund += sstore_clears_schedule;
            }
        }

        // if original is equal to new.
        if vals.is_original_eq_new() {
            if vals.is_original_zero() {
                refund += sstore_set;
            } else {
                refund += sstore_reset;
            }
        }
        return refund;
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
                len.saturating_mul(len)
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

    /// Call stipend reduction. Call stipend is reduced by 1/64 of the gas limit.
    #[inline]
    pub fn call_stipend_reduction(&self, gas_limit: u64) -> u64 {
        gas_limit.saturating_sub(gas_limit.saturating_div(self.get(Self::CALL_STIPEND_REDUCTION)))
    }

    /// Transfer value cost
    #[inline]
    pub fn transfer_value_cost(&self) -> u64 {
        self.get(Self::TRANSFER_VALUE_COST)
    }

    /// Additional cold cost. Additional cold cost is added to the gas cost if the account is cold loaded.
    #[inline]
    pub fn additional_cold_cost(&self) -> u64 {
        self.get(Self::ADDITIONAL_COLD_COST)
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
