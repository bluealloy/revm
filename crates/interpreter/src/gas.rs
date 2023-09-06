pub mod calc;
pub mod constants;

pub use calc::*;
pub use constants::*;

/// Represents the state of gas during execution.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gas {
    /// The initial gas limit.
    limit: u64,
    /// The total used gas.
    all_used_gas: u64,
    /// Used gas without memory expansion.
    used: u64,
    /// Used gas for memory expansion.
    memory: u64,
    /// Refunded gas. This is used only at the end of execution.
    refunded: i64,
}

impl Gas {
    /// Creates a new `Gas` struct with the given gas limit.
    #[inline]
    pub const fn new(limit: u64) -> Self {
        Self {
            limit,
            used: 0,
            memory: 0,
            refunded: 0,
            all_used_gas: 0,
        }
    }

    /// Returns the gas limit.
    #[inline]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns the amount of gas that was used.
    #[inline]
    pub const fn memory(&self) -> u64 {
        self.memory
    }

    /// Returns the amount of gas that was refunded.
    #[inline]
    pub const fn refunded(&self) -> i64 {
        self.refunded
    }

    /// Returns all the gas used in the execution.
    #[inline]
    pub const fn spend(&self) -> u64 {
        self.all_used_gas
    }

    /// Returns the amount of gas remaining.
    #[inline]
    pub const fn remaining(&self) -> u64 {
        self.limit - self.all_used_gas
    }

    /// Erases a gas cost from the totals.
    #[inline]
    pub fn erase_cost(&mut self, returned: u64) {
        self.used -= returned;
        self.all_used_gas -= returned;
    }

    /// Records a refund value.
    ///
    /// `refund` can be negative but `self.refunded` should always be positive.
    #[inline]
    pub fn record_refund(&mut self, refund: i64) {
        self.refunded += refund;
    }

    /// Records an explicit cost.
    ///
    /// This function is called on every instruction in the interpreter if the feature
    /// `no_gas_measuring` is not enabled.
    #[inline(always)]
    pub fn record_cost(&mut self, cost: u64) -> bool {
        let all_used_gas = self.all_used_gas.saturating_add(cost);
        if self.limit < all_used_gas {
            return false;
        }

        self.used += cost;
        self.all_used_gas = all_used_gas;
        true
    }

    /// Consumes the remaining gas.
    #[cfg(not(feature = "optimism"))]
    #[inline]
    pub fn consume_gas(&mut self, ret_gas: Gas) {
        self.erase_cost(ret_gas.remaining());
        self.record_refund(ret_gas.refunded());
    }

    /// Consume the revert gas.
    #[cfg(not(feature = "optimism"))]
    #[inline]
    pub fn consume_revert_gas(&mut self, ret_gas: Gas) {
        self.erase_cost(ret_gas.remaining());
    }

    /// Consume revert gas limit.
    #[cfg(feature = "optimism")]
    #[inline]
    pub fn consume_revert_gas(
        &mut self,
        is_optimism: bool,
        is_deposit: bool,
        is_regolith: bool,
        ret_gas: Gas,
    ) {
        // On Optimism, deposit transactions report gas usage uniquely to other
        // transactions due to them being pre-paid on L1.
        //
        // Hardfork Behavior:
        // - Bedrock (revert path):
        //   - Deposit transactions (all) report the gas limit as the amount of gas
        //     used on failure. No refunds.
        //   - Regular transactions receive a refund on remaining gas as normal.
        // - Regolith (revert path):
        //   - Deposit transactions (all) report the actual gas used as the amount of
        //     gas used on failure. Refunds on remaining gas enabled.
        //   - Regular transactions receive a refund on remaining gas as normal.
        if is_optimism && (!is_deposit || is_regolith) {
            self.erase_cost(ret_gas.remaining());
        }
    }

    /// Consume remaining gas.
    #[cfg(feature = "optimism")]
    #[inline]
    pub fn consume_gas(
        &mut self,
        is_optimism: bool,
        is_deposit: bool,
        is_regolith: bool,
        tx_system: Option<bool>,
        gas_limit: u64,
        ret_gas: Gas,
    ) {
        // On Optimism, deposit transactions report gas usage uniquely to other
        // transactions due to them being pre-paid on L1.
        //
        // Hardfork Behavior:
        // - Bedrock (success path):
        //   - Deposit transactions (non-system) report their gas limit as the usage.
        //     No refunds.
        //   - Deposit transactions (system) report 0 gas used. No refunds.
        //   - Regular transactions report gas usage as normal.
        // - Regolith (success path):
        //   - Deposit transactions (all) report their gas used as normal. Refunds
        //     enabled.
        //   - Regular transactions report their gas used as normal.
        if is_optimism && (!is_deposit || is_regolith) {
            // For regular transactions prior to Regolith and all transactions after
            // Regolith, gas is reported as normal.
            self.erase_cost(ret_gas.remaining());
            self.record_refund(ret_gas.refunded());
        } else if is_deposit && tx_system.unwrap_or(false) {
            // System transactions were a special type of deposit transaction in
            // the Bedrock hardfork that did not incur any gas costs.
            self.erase_cost(gas_limit);
        }
    }

    /// used in memory_resize! macro to record gas used for memory expansion.
    #[inline]
    pub fn record_memory(&mut self, gas_memory: u64) -> bool {
        if gas_memory > self.memory {
            let all_used_gas = self.used.saturating_add(gas_memory);
            if self.limit < all_used_gas {
                return false;
            }
            self.memory = gas_memory;
            self.all_used_gas = all_used_gas;
        }
        true
    }

    #[inline]
    #[deprecated = "Use `record_refund` instead"]
    #[doc(hidden)]
    pub fn gas_refund(&mut self, refund: i64) {
        self.record_refund(refund);
    }
}
