//! EVM gas calculation utilities.

pub use context_interface::cfg::gas::*;

/// Represents the state of gas during execution.
///
/// Implements the EIP-8037 reservoir model for dual-limit gas accounting:
/// - `remaining`: total gas left = `gas_left` + `reservoir`
/// - `reservoir`: state gas pool (gas exceeding regular gas budget)
/// - `gas_spent`: tracks total state gas spent
///
/// **Regular gas charges** (`record_cost`): deduct from `remaining` only, checked against implicit `gas_left`.
/// **State gas charges** (`record_state_cost`): deduct from `reservoir` first; when exhausted, spill into `gas_left`.
///
/// On mainnet (no state gas), `reservoir = 0` so all gas is regular gas and behavior is unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gas {
    /// The initial gas limit. This is constant throughout execution.
    limit: u64,
    /// Total gas remaining: `gas_left` + `reservoir`.
    remaining: u64,
    /// State gas reservoir (gas exceeding TX_MAX_GAS_LIMIT). Starts as `execution_gas - min(execution_gas, regular_gas_budget)`.
    /// When 0, all remaining gas is regular gas with hard cap at `TX_MAX_GAS_LIMIT`.
    reservoir: u64,
    /// Total state gas spent so far.
    state_gas_spent: u64,
    /// Refunded gas. This is used only at the end of execution.
    refunded: i64,
    /// Memoisation of values for memory expansion cost.
    memory: MemoryGas,
}

impl Gas {
    /// Creates a new `Gas` struct with the given gas limit.
    ///
    /// Sets `reservoir = 0` so all gas is regular gas (standard mainnet behavior).
    #[inline]
    pub const fn new(limit: u64) -> Self {
        Self {
            limit,
            remaining: limit,
            reservoir: 0,
            state_gas_spent: 0,
            refunded: 0,
            memory: MemoryGas::new(),
        }
    }

    /// Creates a new `Gas` struct with a regular gas budget and reservoir (EIP-8037 reservoir model).
    ///
    /// Following the EIP-8037 spec:
    /// - `gas_left = min(regular_gas_budget, execution_gas)`
    /// - `state_gas_reservoir = execution_gas - gas_left`
    /// - `remaining = gas_left + state_gas_reservoir = execution_gas`
    ///
    /// # Arguments
    /// * `limit`: total execution gas (tx.gas - intrinsic_gas)
    /// * `regular_gas_budget`: regular gas cap (TX_MAX_GAS_LIMIT - intrinsic_regular_gas)
    #[inline]
    pub const fn new_with_regular_gas_budget(limit: u64, reservoir: u64) -> Self {
        Self {
            limit,
            remaining: limit,
            reservoir,
            state_gas_spent: 0,
            refunded: 0,
            memory: MemoryGas::new(),
        }
    }

    /// Deprecated: use `new_with_regular_gas_budget` instead.
    /// Alias for backwards compatibility.
    #[inline]
    #[deprecated(
        since = "32.0.0",
        note = "use new_with_regular_gas_budget for EIP-8037 semantics"
    )]
    pub const fn new_with_regular_gas_remaining(limit: u64, regular_gas_remaining: u64) -> Self {
        Self::new_with_regular_gas_budget(limit, regular_gas_remaining)
    }

    /// Creates a new `Gas` struct with the given gas limit, but without any gas remaining.
    #[inline]
    pub const fn new_spent(limit: u64) -> Self {
        Self {
            limit,
            remaining: 0,
            reservoir: 0,
            state_gas_spent: 0,
            refunded: 0,
            memory: MemoryGas::new(),
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

    /// Returns the state gas reservoir.
    #[inline]
    pub const fn reservoir(&self) -> u64 {
        self.reservoir
    }

    /// Sets the state gas reservoir (used when propagating from child frame).
    #[inline]
    pub fn set_reservoir(&mut self, val: u64) {
        self.reservoir = val;
    }

    /// Returns total state gas spent so far.
    #[inline]
    pub const fn state_gas_spent(&self) -> u64 {
        self.state_gas_spent
    }

    /// Sets the total state gas spent (used when propagating from child frame).
    #[inline]
    pub fn set_state_gas_spent(&mut self, val: u64) {
        self.state_gas_spent = val;
    }

    /// Erases a gas cost from remaining (returns gas from child frame).
    /// Does NOT affect `regular_gas_remaining` — regular gas flows separately.
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

    /// Set a remaining value. This overrides the current remaining value.
    #[inline]
    pub fn set_remaining(&mut self, remaining: u64) {
        self.remaining = remaining;
    }

    /// Set a spent value. This overrides the current spent value.
    #[inline]
    pub fn set_spent(&mut self, spent: u64) {
        self.remaining = self.limit.saturating_sub(spent);
    }

    /// Records a regular gas cost (EIP-8037 reservoir model).
    ///
    /// Deducts from `remaining` and checks against implicit `gas_left` budget.
    /// Regular gas charges cannot draw from the reservoir.
    ///
    /// Returns `false` if the regular gas limit is exceeded.
    /// On failure, values contain wrapped (invalid) state — callers must not read after OOG.
    #[inline]
    #[must_use = "prefer using `gas!` instead to return an out-of-gas error on failure"]
    pub fn record_cost(&mut self, cost: u64) -> bool {
        if let Some(new_remaining) = self.remaining.checked_sub(cost) {
            self.remaining = new_remaining;
            return true;
        }
        false
    }

    /// Records an explicit cost without bounds checking (unsafe path).
    ///
    /// Returns `true` if the gas limit is exceeded. Values wrap on underflow.
    /// Only the regular gas check is meaningful here; total remaining can underflow
    /// without consequence if the caller handles it.
    #[inline(always)]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub fn record_cost_unsafe(&mut self, cost: u64) -> bool {
        let oog = self.remaining < cost;
        self.remaining = self.remaining.wrapping_sub(cost);
        oog
    }

    /// Records an explicit cost without checking regular gas budget.
    /// In case of underflow the gas will wrap around cost.
    ///
    /// This is the fast path used when state gas is not enabled (mainnet).
    ///
    /// Returns `true` if the gas limit is exceeded.
    #[inline(always)]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub fn record_cost_unsafe_no_regular(&mut self, cost: u64) -> bool {
        let oog = self.remaining < cost;
        self.remaining = self.remaining.wrapping_sub(cost);
        oog
    }

    /// Records a state gas cost (EIP-8037 reservoir model).
    ///
    /// State gas charges deduct from the reservoir first. If the reservoir is exhausted,
    /// remaining charges spill into `gas_left` (requiring total `remaining >= cost`).
    /// Tracks state gas spent.
    ///
    /// Returns `false` if total remaining gas is insufficient.
    #[inline]
    pub fn record_state_cost(&mut self, cost: u64) -> bool {
        // bump state gas spent
        self.state_gas_spent = self.state_gas_spent.saturating_add(cost);

        if self.reservoir >= cost {
            self.reservoir -= cost;
            return true;
        }

        let mut spill = cost;
        if self.reservoir != 0 {
            spill -= self.reservoir;
            self.reservoir = 0;
        }

        self.record_cost(spill)
    }

    /// Deducts from `remaining` only (used for child frame gas forwarding).
    /// Does not affect reservoir or regular gas budget.
    /// Used for forwarding gas to child frames.
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
