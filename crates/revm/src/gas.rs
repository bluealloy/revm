mod calc;
mod constants;

pub use calc::*;
pub use constants::*;

use crate::{instructions::Return, return_ok, return_revert};

#[derive(Clone, Copy, Debug)]
pub struct Gas {
    limit: u64,
    used: u64,
    memory: u64,
    refunded: i64,
    all_used_gas: u64,
}
impl Gas {
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            used: 0,
            memory: 0,
            refunded: 0,
            all_used_gas: 0,
        }
    }

    pub fn reimburse_unspend(&mut self, exit: &Return, other: Gas) {
        match *exit {
            return_ok!() => {
                self.erase_cost(other.remaining());
                self.record_refund(other.refunded());
            }
            return_revert!() => {
                self.erase_cost(other.remaining());
            }
            _ => {}
        }
    }

    pub fn limit(&self) -> u64 {
        self.limit
    }

    pub fn memory(&self) -> u64 {
        self.memory
    }

    pub fn refunded(&self) -> i64 {
        self.refunded
    }

    pub fn spend(&self) -> u64 {
        self.all_used_gas
    }

    pub fn remaining(&self) -> u64 {
        self.limit - self.all_used_gas
    }

    pub fn erase_cost(&mut self, returned: u64) {
        self.used -= returned;
        self.all_used_gas -= returned;
    }

    pub fn record_refund(&mut self, refund: i64) {
        self.refunded += refund;
    }

    /// Record an explict cost.
    #[inline(always)]
    pub fn record_cost(&mut self, cost: u64) -> bool {
        let (all_used_gas, overflow) = self.all_used_gas.overflowing_add(cost);
        if overflow || self.limit < all_used_gas {
            return false;
        }

        self.used += cost;
        self.all_used_gas = all_used_gas;
        true
    }

    /// used in memory_resize! macro

    pub fn record_memory(&mut self, gas_memory: u64) -> bool {
        if gas_memory > self.memory {
            let (all_used_gas, overflow) = self.used.overflowing_add(gas_memory);
            if overflow || self.limit < all_used_gas {
                return false;
            }
            self.memory = gas_memory;
            self.all_used_gas = all_used_gas;
        }
        true
    }

    /// used in gas_refund! macro
    pub fn gas_refund(&mut self, refund: i64) {
        self.refunded += refund;
    }
}
