//! EVM gas calculation utilities.

pub use context_interface::cfg::gas::*;

/// Represents the state of gas during execution.
///
/// Implements the EIP-8037 reservoir model for dual-limit gas accounting:
/// - `remaining`: regular gas left (`gas_left`). Does NOT include `reservoir`.
/// - `reservoir`: state gas pool (separate from `remaining`). Starts as `execution_gas - gas_left`.
/// - `state_gas_spent`: tracks total state gas spent
///
/// **Regular gas charges** (`record_cost`): deduct from `remaining`, checked against `remaining`.
/// **State gas charges** (`record_state_cost`): deduct from `reservoir` first; when exhausted, spill into `remaining`.
/// Total gas available = `remaining` + `reservoir`.
///
/// On mainnet (no state gas), `reservoir = 0` so all gas is regular gas and behavior is unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gas {
    /// Tracker for gas during execution.
    tracker: GasTracker,
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
            tracker: GasTracker::new(limit, limit, 0),
            memory: MemoryGas::new(),
        }
    }

    /// Returns the tracker for gas during execution.
    #[inline]
    pub const fn tracker(&self) -> &GasTracker {
        &self.tracker
    }

    /// Returns the mutable tracker for gas during execution.
    #[inline]
    pub const fn tracker_mut(&mut self) -> &mut GasTracker {
        &mut self.tracker
    }

    /// Creates a new `Gas` struct with a regular gas budget and reservoir (EIP-8037 reservoir model).
    ///
    /// Following the EIP-8037 spec:
    /// - `remaining = limit` (regular gas available, i.e. `gas_left`)
    /// - `reservoir` = state gas pool (separate from `remaining`)
    /// - Total gas available = `remaining + reservoir = limit + reservoir`
    ///
    /// # Arguments
    /// * `limit`: regular gas budget (capped execution gas, i.e. `gas_left`)
    /// * `reservoir`: state gas pool (execution gas exceeding the regular gas cap)
    #[inline]
    pub const fn new_with_regular_gas_and_reservoir(limit: u64, reservoir: u64) -> Self {
        Self {
            tracker: GasTracker::new(limit, limit, reservoir),
            memory: MemoryGas::new(),
        }
    }

    /// Creates a new `Gas` struct with the given gas limit, but without any gas remaining.
    #[inline]
    pub const fn new_spent_with_reservoir(limit: u64, reservoir: u64) -> Self {
        Self {
            tracker: GasTracker::new(limit, 0, reservoir),
            memory: MemoryGas::new(),
        }
    }

    /// Returns the gas limit.
    #[inline]
    pub const fn limit(&self) -> u64 {
        self.tracker.limit()
    }

    /// Returns the memory gas.
    #[inline]
    pub const fn memory(&self) -> &MemoryGas {
        &self.memory
    }

    /// Returns the memory gas.
    #[inline]
    pub const fn memory_mut(&mut self) -> &mut MemoryGas {
        &mut self.memory
    }

    /// Returns the total amount of gas that was refunded.
    #[inline]
    pub const fn refunded(&self) -> i64 {
        self.tracker.refunded()
    }

    /// Returns the total amount of gas spent.
    #[inline]
    #[deprecated(
        since = "32.0.0",
        note = "After EIP-8037 gas is split on
    regular and state gas, this method is no longer valid.
    Use [`Gas::total_gas_spent`] instead"
    )]
    pub const fn spent(&self) -> u64 {
        self.tracker
            .limit()
            .saturating_sub(self.tracker.remaining())
    }

    /// Returns the regular gas spent.
    #[inline]
    pub const fn total_gas_spent(&self) -> u64 {
        self.tracker
            .limit()
            .saturating_sub(self.tracker.remaining())
    }

    /// Returns the final amount of gas used by subtracting the refund from spent gas.
    #[inline]
    pub const fn used(&self) -> u64 {
        self.total_gas_spent()
            .saturating_sub(self.refunded() as u64)
    }

    /// Returns the total amount of gas spent, minus the refunded gas.
    #[inline]
    pub const fn spent_sub_refunded(&self) -> u64 {
        self.total_gas_spent()
            .saturating_sub(self.tracker.refunded() as u64)
    }

    /// Returns the amount of gas remaining.
    #[inline]
    pub const fn remaining(&self) -> u64 {
        self.tracker.remaining()
    }

    /// Returns the state gas reservoir.
    #[inline]
    pub const fn reservoir(&self) -> u64 {
        self.tracker.reservoir()
    }

    /// Sets the state gas reservoir (used when propagating from child frame).
    #[inline]
    pub const fn set_reservoir(&mut self, val: u64) {
        self.tracker.set_reservoir(val);
    }

    /// Returns total state gas spent so far.
    #[inline]
    pub const fn state_gas_spent(&self) -> u64 {
        self.tracker.state_gas_spent()
    }

    /// Sets the total state gas spent (used when propagating from child frame).
    #[inline]
    pub const fn set_state_gas_spent(&mut self, val: u64) {
        self.tracker.set_state_gas_spent(val);
    }

    /// Erases a gas cost from remaining (returns gas from child frame).
    #[inline]
    pub const fn erase_cost(&mut self, returned: u64) {
        self.tracker.erase_cost(returned);
    }

    /// Spends all remaining gas excluding the reservoir.
    ///
    /// On exceptional halt, the remaining gas must be zeroed
    /// to prevent state operations from succeeding via remaining gas.
    ///
    /// Note that this does not affect the reservoir.
    #[inline]
    pub const fn spend_all(&mut self) {
        self.tracker.spend_all();
    }

    /// Records a refund value.
    ///
    /// `refund` can be negative but `self.refunded` should always be positive
    /// at the end of transact.
    #[inline]
    pub const fn record_refund(&mut self, refund: i64) {
        self.tracker.record_refund(refund);
    }

    /// Set a refund value for final refund.
    ///
    /// Max refund value is limited to Nth part (depending of fork) of gas spend.
    ///
    /// Related to EIP-3529: Reduction in refunds
    #[inline]
    pub fn set_final_refund(&mut self, is_london: bool) {
        let max_refund_quotient = if is_london { 5 } else { 2 };
        // EIP-8037: gas_used = total_gas_spent - reservoir (reservoir is unused state gas)
        let gas_used = self.total_gas_spent().saturating_sub(self.reservoir());
        self.tracker
            .set_refunded((self.refunded() as u64).min(gas_used / max_refund_quotient) as i64);
    }

    /// Set a refund value. This overrides the current refund value.
    #[inline]
    pub const fn set_refund(&mut self, refund: i64) {
        self.tracker.set_refunded(refund);
    }

    /// Set a remaining value. This overrides the current remaining value.
    #[inline]
    pub const fn set_remaining(&mut self, remaining: u64) {
        self.tracker.set_remaining(remaining);
    }

    /// Set a spent value. This overrides the current spent value.
    #[inline]
    pub const fn set_spent(&mut self, spent: u64) {
        self.tracker
            .set_remaining(self.tracker.limit().saturating_sub(spent));
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
    #[deprecated(since = "32.0.0", note = "use record_regular_cost instead")]
    pub const fn record_cost(&mut self, cost: u64) -> bool {
        self.record_regular_cost(cost)
    }

    /// Records an explicit cost without bounds checking (unsafe path).
    ///
    /// Returns `true` if the gas limit is exceeded. Values wrap on underflow.
    /// Only the regular gas check is meaningful here; total remaining can underflow
    /// without consequence if the caller handles it.
    #[inline(always)]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_cost_unsafe(&mut self, cost: u64) -> bool {
        let remaining = self.tracker.remaining();
        let oog = remaining < cost;
        self.tracker.set_remaining(remaining.wrapping_sub(cost));
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
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_state_cost(&mut self, cost: u64) -> bool {
        self.tracker.record_state_cost(cost)
    }

    /// Deducts from `remaining` only (used for child frame gas forwarding).
    /// Does not affect reservoir or regular gas budget.
    /// Used for forwarding gas to child frames.
    #[inline]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_regular_cost(&mut self, cost: u64) -> bool {
        self.tracker.record_regular_cost(cost)
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
    pub const fn set_words_num(
        &mut self,
        words_num: usize,
        mut expansion_cost: u64,
    ) -> Option<u64> {
        self.words_num = words_num;
        core::mem::swap(&mut self.expansion_cost, &mut expansion_cost);
        self.expansion_cost.checked_sub(expansion_cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_state_cost() {
        // Test 1: Cost from reservoir only
        let mut gas = Gas::new_with_regular_gas_and_reservoir(1000, 500);
        assert!(gas.record_state_cost(200));
        assert_eq!(
            (gas.reservoir(), gas.remaining(), gas.state_gas_spent()),
            (300, 1000, 200)
        );

        // Test 2: Exhaust reservoir exactly
        let mut gas = Gas::new_with_regular_gas_and_reservoir(1000, 500);
        assert!(gas.record_state_cost(500));
        assert_eq!(
            (gas.reservoir(), gas.remaining(), gas.state_gas_spent()),
            (0, 1000, 500)
        );

        // Test 3: Spill to remaining (reservoir < cost)
        let mut gas = Gas::new_with_regular_gas_and_reservoir(1000, 300);
        assert!(gas.record_state_cost(500));
        assert_eq!(
            (gas.reservoir(), gas.remaining(), gas.state_gas_spent()),
            (0, 800, 500)
        );

        // Test 4: No reservoir (mainnet standard)
        let mut gas = Gas::new(1000);
        assert!(gas.record_state_cost(200));
        assert_eq!(
            (gas.reservoir(), gas.remaining(), gas.state_gas_spent()),
            (0, 800, 200)
        );

        // Test 5: Zero cost
        let mut gas = Gas::new_with_regular_gas_and_reservoir(100, 50);
        assert!(gas.record_state_cost(0));
        assert_eq!(
            (gas.reservoir(), gas.remaining(), gas.state_gas_spent()),
            (50, 100, 0)
        );

        // Test 6: Out of gas (cost > remaining + reservoir)
        let mut gas = Gas::new_with_regular_gas_and_reservoir(100, 50);
        assert!(!gas.record_state_cost(200));

        // Test 7: Multiple operations accumulate state_gas_spent
        let mut gas = Gas::new_with_regular_gas_and_reservoir(2000, 1000);
        assert!(gas.record_state_cost(100));
        assert!(gas.record_state_cost(200));
        assert!(gas.record_state_cost(150));
        assert_eq!(gas.state_gas_spent(), 450);

        // Test 8: Complex scenario exhausting reservoir then remaining
        let mut gas = Gas::new_with_regular_gas_and_reservoir(500, 300);
        assert!(gas.record_state_cost(150)); // 150 from reservoir
        assert_eq!((gas.reservoir(), gas.remaining()), (150, 500));
        assert!(gas.record_state_cost(200)); // 150 from reservoir, 50 from remaining
        assert_eq!((gas.reservoir(), gas.remaining()), (0, 450));
        assert!(gas.record_state_cost(100)); // 100 from remaining
        assert_eq!(
            (gas.reservoir(), gas.remaining(), gas.state_gas_spent()),
            (0, 350, 450)
        );
    }

    /// A.1: Verify state_gas_spent is incremented even after failed record_state_cost.
    /// On OOG, state_gas_spent is NOT incremented and reservoir is unchanged.
    #[test]
    fn test_record_state_cost_oog_inflates_state_gas_spent() {
        // remaining=30, reservoir=0, cost=100 → OOG
        let mut gas = Gas::new(30);
        assert!(!gas.record_state_cost(100));
        // On OOG, state_gas_spent is NOT incremented (operation failed)
        assert_eq!(gas.state_gas_spent(), 0);

        // With reservoir partially covering: reservoir=20, remaining=30, cost=100
        // spill = 100 - 20 = 80, remaining(30) < 80 → OOG
        let mut gas = Gas::new_with_regular_gas_and_reservoir(30, 20);
        assert!(!gas.record_state_cost(100));
        // On OOG, state_gas_spent is NOT incremented and reservoir is unchanged
        assert_eq!(gas.state_gas_spent(), 0);
        assert_eq!(gas.reservoir(), 20);
    }

    /// A.3: State gas with zero regular remaining but non-zero reservoir.
    #[test]
    fn test_record_state_cost_zero_remaining_with_reservoir() {
        // remaining=0, reservoir=500: state gas draws entirely from reservoir
        let mut gas = Gas::new_with_regular_gas_and_reservoir(0, 500);
        assert!(gas.record_state_cost(200));
        assert_eq!(
            (gas.reservoir(), gas.remaining(), gas.state_gas_spent()),
            (300, 0, 200)
        );

        // Exhaust reservoir exactly
        assert!(gas.record_state_cost(300));
        assert_eq!(
            (gas.reservoir(), gas.remaining(), gas.state_gas_spent()),
            (0, 0, 500)
        );

        // Now any cost → OOG (both remaining and reservoir are 0)
        assert!(!gas.record_state_cost(1));
    }
}
