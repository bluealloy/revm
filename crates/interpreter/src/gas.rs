//! EVM gas calculation utilities.

mod calc;
mod constants;

pub use calc::*;
pub use constants::*;

/// Represents the state of gas during execution.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gas {
    /// The initial gas limit. This is constant throughout execution.
    limit: u64,
    /// The remaining gas.
    remaining: u64,
    /// Refunded gas. This is used only at the end of execution.
    refunded: i64,
    /// Memoisation of values for memory expansion cost.
    pub memory: MemoryGas,
}

impl Gas {
    /// Creates a new `Gas` struct with the given gas limit.
    #[inline]
    pub const fn new(limit: u64) -> Self {
        Self {
            limit,
            remaining: limit,
            refunded: 0,
            memory: MemoryGas::new(),
        }
    }

    /// Creates a new `Gas` struct with the given gas limit, but without any gas remaining.
    #[inline]
    pub const fn new_spent(limit: u64) -> Self {
        Self {
            limit,
            remaining: 0,
            refunded: 0,
            memory: MemoryGas::new(),
        }
    }

    /// Returns the gas limit.
    #[inline]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns the **last** memory expansion cost.
    #[inline]
    #[deprecated = "memory expansion cost is not tracked anymore; \
                    calculate it using `SharedMemory::current_expansion_cost` instead"]
    #[doc(hidden)]
    pub const fn memory(&self) -> u64 {
        0
    }

    /// Returns the total amount of gas that was refunded.
    #[inline]
    pub const fn refunded(&self) -> i64 {
        self.refunded
    }

    /// Returns the total amount of gas spent.
    #[inline]
    pub const fn spent(&self) -> u64 {
        self.limit - self.remaining
    }

    /// Returns the total amount of gas spent, minus the refunded gas.
    #[inline]
    pub const fn spent_sub_refunded(&self) -> u64 {
        self.spent().saturating_sub(self.refunded as u64)
    }

    /// Returns the amount of gas remaining.
    #[inline]
    pub const fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Return remaining gas after subtracting 63/64 parts.
    pub const fn remaining_63_of_64_parts(&self) -> u64 {
        self.remaining - self.remaining / 64
    }

    /// Erases a gas cost from the totals.
    #[inline]
    pub fn erase_cost(&mut self, returned: u64) {
        self.remaining += returned;
    }

    /// Spends all remaining gas.
    #[inline]
    pub fn spend_all(&mut self) {
        self.remaining = 0;
    }

    /// Records a refund value.
    ///
    /// `refund` can be negative but `self.refunded` should always be positive
    /// at the end of transact.
    #[inline]
    pub fn record_refund(&mut self, refund: i64) {
        self.refunded += refund;
    }

    /// Set a refund value for final refund.
    ///
    /// Max refund value is limited to Nth part (depending of fork) of gas spend.
    ///
    /// Related to EIP-3529: Reduction in refunds
    #[inline]
    pub fn set_final_refund(&mut self, is_london: bool) {
        let max_refund_quotient = if is_london { 5 } else { 2 };
        self.refunded = (self.refunded() as u64).min(self.spent() / max_refund_quotient) as i64;
    }

    /// Set a refund value. This overrides the current refund value.
    #[inline]
    pub fn set_refund(&mut self, refund: i64) {
        self.refunded = refund;
    }

    /// Set a spent value. This overrides the current spent value.
    #[inline]
    pub fn set_spent(&mut self, spent: u64) {
        self.remaining = self.limit.saturating_sub(spent);
    }

    /// Records an explicit cost.
    ///
    /// Returns `false` if the gas limit is exceeded.
    #[inline]
    #[must_use = "prefer using `gas!` instead to return an out-of-gas error on failure"]
    pub fn record_cost(&mut self, cost: u64) -> bool {
        if let Some(new_remaining) = self.remaining.checked_sub(cost) {
            self.remaining = new_remaining;
            return true;
        }
        false
    }

    /// Record memory expansion
    #[inline]
    #[must_use = "internally uses record_cost that flags out of gas error"]
    pub fn record_memory_expansion(&mut self, new_len: usize) -> MemoryExtensionResult {
        let Some(additional_cost) = self.memory.record_new_len(new_len) else {
            return MemoryExtensionResult::Same;
        };

        if !self.record_cost(additional_cost) {
            return MemoryExtensionResult::OutOfGas;
        }

        MemoryExtensionResult::Extended
    }
}

pub enum MemoryExtensionResult {
    /// Memory was extended.
    Extended,
    /// Memory size stayed the same.
    Same,
    /// Not enough gas to extend memory.
    OutOfGas,
}

/// Utility struct that speeds up calculation of memory expansion
/// It contains the current memory length and its memory expansion cost.
///
/// It allows us to split gas accounting from memory structure.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemoryGas {
    /// Current memory length
    pub words_num: usize,
    /// Current memory expansion cost
    pub expansion_cost: u64,
}

impl MemoryGas {
    pub const fn new() -> Self {
        Self {
            words_num: 0,
            expansion_cost: 0,
        }
    }

    #[inline]
    pub fn record_new_len(&mut self, new_num: usize) -> Option<u64> {
        if new_num <= self.words_num {
            return None;
        }
        self.words_num = new_num;
        let mut cost = crate::gas::calc::memory_gas(new_num);
        core::mem::swap(&mut self.expansion_cost, &mut cost);
        // Safe to subtract because we know that new_len > length
        // Notice the swap above.
        Some(self.expansion_cost - cost)
    }
}
