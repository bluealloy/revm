//! EVM gas calculation utilities.

pub use context_interface::cfg::gas::*;

/// Represents the state of gas during execution.
///
/// Supports dual-limit gas accounting (TIP-1016):
/// - `remaining`: gas remaining against `tx.gas_limit` (both execution + state gas)
/// - `cpu_gas_remaining`: gas remaining against CPU cap (execution gas only)
/// - `state_gas`: accumulated storage creation gas
///
/// On mainnet (no state gas), `cpu_gas_remaining = u64::MAX` so the CPU check
/// in `record_cost` is always a no-op (~0 overhead).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gas {
    /// The initial gas limit. This is constant throughout execution.
    limit: u64,
    /// The remaining gas against tx.gas_limit.
    remaining: u64,
    /// Gas remaining against CPU cap. Set to `u64::MAX` on mainnet (no state gas).
    cpu_gas_remaining: u64,
    /// Refunded gas. This is used only at the end of execution.
    refunded: i64,
    /// Memoisation of values for memory expansion cost.
    memory: MemoryGas,
    /// Accumulated storage creation gas (state gas).
    state_gas: u64,
}

impl Gas {
    /// Creates a new `Gas` struct with the given gas limit.
    ///
    /// Sets `cpu_gas_remaining = u64::MAX` so the CPU check in `record_cost`
    /// is always a no-op (standard mainnet behavior).
    #[inline]
    pub const fn new(limit: u64) -> Self {
        Self {
            limit,
            remaining: limit,
            cpu_gas_remaining: u64::MAX,
            refunded: 0,
            memory: MemoryGas::new(),
            state_gas: 0,
        }
    }

    /// Creates a new `Gas` struct with a CPU gas limit (state gas / Tempo enabled).
    ///
    /// `cpu_gas_remaining` tracks gas remaining against the CPU cap.
    #[inline]
    pub const fn new_with_cpu_remaining(limit: u64, cpu_gas_remaining: u64) -> Self {
        Self {
            limit,
            remaining: limit,
            cpu_gas_remaining,
            refunded: 0,
            memory: MemoryGas::new(),
            state_gas: 0,
        }
    }

    /// Creates a new `Gas` struct with the given gas limit, but without any gas remaining.
    #[inline]
    pub const fn new_spent(limit: u64) -> Self {
        Self {
            limit,
            remaining: 0,
            cpu_gas_remaining: 0,
            refunded: 0,
            memory: MemoryGas::new(),
            state_gas: 0,
        }
    }

    /// Returns the gas limit.
    #[inline]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns the memory gas.
    #[inline]
    pub fn memory(&self) -> &MemoryGas {
        &self.memory
    }

    /// Returns the memory gas.
    #[inline]
    pub fn memory_mut(&mut self) -> &mut MemoryGas {
        &mut self.memory
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

    /// Returns the final amount of gas used by subtracting the refund from spent gas.
    #[inline]
    pub const fn used(&self) -> u64 {
        self.spent().saturating_sub(self.refunded() as u64)
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

    /// Returns the accumulated state gas.
    #[inline]
    pub const fn state_gas(&self) -> u64 {
        self.state_gas
    }

    /// Returns the CPU gas remaining.
    #[inline]
    pub const fn cpu_gas_remaining(&self) -> u64 {
        self.cpu_gas_remaining
    }

    /// Execution gas spent = total spent - state gas.
    #[inline]
    pub const fn execution_gas_spent(&self) -> u64 {
        self.spent().saturating_sub(self.state_gas)
    }

    /// Sets cpu_gas_remaining (used when propagating from child frame).
    #[inline]
    pub fn set_cpu_gas_remaining(&mut self, val: u64) {
        self.cpu_gas_remaining = val;
    }

    /// Adds state gas from a child frame.
    #[inline]
    pub fn add_state_gas(&mut self, gas: u64) {
        self.state_gas += gas;
    }

    /// Erases a gas cost from remaining (returns gas from child frame).
    /// Does NOT affect cpu_gas_remaining — CPU flows separately.
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

    /// Records an execution gas cost. Checks BOTH `remaining` AND `cpu_gas_remaining`.
    ///
    /// On mainnet (`cpu_gas_remaining = u64::MAX`), the CPU check is always a no-op.
    ///
    /// Returns `false` if the gas limit is exceeded.
    /// On failure, `remaining` and `cpu_gas_remaining` contain wrapped (invalid) values —
    /// callers must not read them after an out-of-gas condition.
    #[inline]
    #[must_use = "prefer using `gas!` instead to return an out-of-gas error on failure"]
    pub fn record_cost(&mut self, cost: u64) -> bool {
        let (new_remaining, o1) = self.remaining.overflowing_sub(cost);
        let (new_cpu, o2) = self.cpu_gas_remaining.overflowing_sub(cost);
        self.remaining = new_remaining;
        self.cpu_gas_remaining = new_cpu;
        !(o1 | o2)
    }

    /// Records an explicit cost. In case of underflow the gas will wrap around cost.
    ///
    /// On mainnet: `cpu_gas_remaining = u64::MAX`, so `cpu < cost` is always false.
    ///
    /// Returns `true` if the gas limit is exceeded.
    #[inline(always)]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub fn record_cost_unsafe(&mut self, cost: u64) -> bool {
        let oog = self.remaining < cost || self.cpu_gas_remaining < cost;
        self.remaining = self.remaining.wrapping_sub(cost);
        self.cpu_gas_remaining = self.cpu_gas_remaining.wrapping_sub(cost);
        oog
    }

    /// Records storage creation gas. ONLY deducts from `remaining` (not cpu).
    /// State gas counts toward `tx.gas_limit` but NOT toward CPU cap.
    #[inline]
    pub fn record_state_gas(&mut self, cost: u64) -> bool {
        if let Some(new_remaining) = self.remaining.checked_sub(cost) {
            self.remaining = new_remaining;
            self.state_gas += cost;
            return true;
        }
        false
    }

    /// Deducts from `remaining` only, without tracking as state gas.
    /// Used for forwarding gas to child frames (not execution, not state).
    #[inline]
    pub fn record_remaining_cost(&mut self, cost: u64) -> bool {
        if let Some(new_remaining) = self.remaining.checked_sub(cost) {
            self.remaining = new_remaining;
            return true;
        }
        false
    }
}

/// Result of attempting to extend memory during execution.
#[derive(Debug)]
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
    /// Creates a new `MemoryGas` instance with zero memory allocation.
    #[inline]
    pub const fn new() -> Self {
        Self {
            words_num: 0,
            expansion_cost: 0,
        }
    }

    /// Sets the number of words and the expansion cost.
    ///
    /// Returns the difference between the new and old expansion cost.
    #[inline]
    pub fn set_words_num(&mut self, words_num: usize, mut expansion_cost: u64) -> Option<u64> {
        self.words_num = words_num;
        core::mem::swap(&mut self.expansion_cost, &mut expansion_cost);
        self.expansion_cost.checked_sub(expansion_cost)
    }

    /// Records a new memory length and calculates additional cost if memory is expanded.
    /// Returns the additional gas cost required, or None if no expansion is needed.
    #[inline]
    pub fn record_new_len(
        &mut self,
        new_num: usize,
        linear_cost: u64,
        quadratic_cost: u64,
    ) -> Option<u64> {
        if new_num <= self.words_num {
            return None;
        }
        self.words_num = new_num;
        let mut cost = memory_gas(new_num, linear_cost, quadratic_cost);
        core::mem::swap(&mut self.expansion_cost, &mut cost);
        // Safe to subtract because we know that new_len > length
        // Notice the swap above.
        Some(self.expansion_cost - cost)
    }
}

/// Standalone wrapper for [`Gas::record_cost`] to inspect assembly via `cargo asm`.
#[inline(never)]
pub fn record_cost_asm(gas: &mut Gas, cost: u64) -> bool {
    gas.record_cost(cost)
}

/// Memory expansion cost calculation for a given number of words.
#[inline]
pub const fn memory_gas(num_words: usize, linear_cost: u64, quadratic_cost: u64) -> u64 {
    let num_words = num_words as u64;
    linear_cost
        .saturating_mul(num_words)
        .saturating_add(num_words.saturating_mul(num_words) / quadratic_cost)
}
