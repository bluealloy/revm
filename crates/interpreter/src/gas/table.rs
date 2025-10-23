//! Gas table for dynamic gas constants.

use crate::{
    gas::{self, log2floor},
    num_words, tri,
};
use context_interface::context::SStoreResult;
use primitives::{
    hardfork::SpecId::{self},
    U256,
};
use std::sync::Arc;

/// Gas table for dynamic gas constants.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GasTable {
    /// Table of gas costs for operations
    table: Arc<[u64; 256]>,
    /// Pointer to the table.
    ptr: *const u64,
    // TODO should we have spec or not.
}

#[cfg(feature = "serde")]
mod serde {
    use super::{Arc, GasTable};

    #[derive(serde::Serialize, serde::Deserialize)]
    struct GasTableSerde {
        table: Vec<u64>,
    }

    #[cfg(feature = "serde")]
    impl serde::Serialize for GasTable {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            GasTableSerde {
                table: self.table.to_vec(),
            }
            .serialize(serializer)
        }
    }

    impl<'de> serde::Deserialize<'de> for GasTable {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let table = GasTableSerde::deserialize(deserializer)?;
            if table.table.len() != 256 {
                return Err(serde::de::Error::custom("Invalid gas table length"));
            }
            Ok(Self::new(Arc::new(table.table.try_into().unwrap())))
        }
    }
}

impl Default for GasTable {
    fn default() -> Self {
        let table = Arc::new([0; 256]);
        Self::new(table)
    }
}

impl GasTable {
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

    /// Creates a new `GasTable` with the given table.
    #[inline]
    pub fn new(table: Arc<[u64; 256]>) -> Self {
        Self {
            ptr: table.as_ptr(),
            table,
        }
    }

    /// Creates a new `GasTable` for the given spec.
    #[inline]
    pub fn new_spec(spec: SpecId) -> Self {
        let mut table = [0; 256];

        table[Self::SSTORE as usize] = gas::SSTORE_RESET;
        table[Self::LOGDATA as usize] = gas::LOGDATA;
        table[Self::LOGTOPIC as usize] = gas::LOGTOPIC;
        table[Self::EXTCODECOPY_PER_WORD as usize] = gas::COPY;
        table[Self::MCOPY_PER_WORD as usize] = gas::COPY;
        table[Self::KECCAK256_PER_WORD as usize] = gas::KECCAK256WORD;

        if spec.is_enabled_in(SpecId::ISTANBUL) {
            table[Self::SSTORE as usize] = gas::ISTANBUL_SLOAD_GAS;
        }

        if spec.is_enabled_in(SpecId::BERLIN) {
            table[Self::SSTORE as usize] = gas::WARM_STORAGE_READ_COST;
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

    /// Dynamic gas cost for SSTORE opcode
    #[inline]
    pub fn sstore_dynamic_gas(&self, vals: &SStoreResult, is_cold: bool) -> u64 {
        //gas::dyn_sstore_cost(spec_id, vals, is_cold)
        todo!("Implement dynamic gas cost for SSTORE opcode");
        0
    }

    /// `LOG` opcode cost calculation.

    #[inline]
    pub const fn log_cost(&self, n: u8, len: u64) -> Option<u64> {
        tri!(self.get(Self::LOGDATA).checked_mul(len))
            .checked_add(self.get(Self::LOGTOPIC) * n as u64)
    }

    /// KECCAK256 gas cost per word
    #[inline]
    pub fn keccak256_cost(&self, len: usize) -> u64 {
        self.get(Self::KECCAK256_PER_WORD)
            .saturating_mul(num_words(len) as u64)
    }
}

/// Calculate the cost of a byte operation.
#[inline]
pub fn byte_cost(base_cost: u64, word_cost: u64, len: usize) -> u64 {
    base_cost.saturating_add(word_cost.saturating_mul(num_words(len) as u64))
}

/// Gas identifier
pub type GasId = u8;
