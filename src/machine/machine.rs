use crate::{collection::vec::Vec, opcode::eval, ExitError};
use bytes::Bytes;
use core::{cmp::max, ops::Range};
use primitive_types::U256;

use super::{contract::Contract, memory::Memory, stack::Stack};
use crate::{error::ExitReason, opcode::Control, spec::Spec, Handler};

pub const STACK_LIMIT: u64 = 1024;
pub const CALL_STACK_LIMIT: u64 = 1024;

pub struct Machine {
    /// Contract information and invoking data
    pub contract: Contract,
    /// Program counter.
    pub program_counter: usize,
    /// Return value.
    pub return_range: Range<U256>,
    /// Memory.
    pub memory: Memory,
    /// Stack.
    pub stack: Stack,
    /// After call returns, its return data is saved here.
    pub return_data_buffer: Bytes,
    /// left gas. Memory gas can be found in Memory field.
    pub gas: Gas,
    /// used only for inspector.
    pub call_depth: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct Gas {
    limit: u64,
    used: u64,
    memory: u64,
    refunded: i64,
}
impl Gas {
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            used: 0,
            memory: 0,
            refunded: 0,
        }
    }

    pub fn reimburse_unspend(&mut self, exit: &ExitReason, other: Gas) {
        match exit {
            ExitReason::Succeed(_) => {
                self.erase_cost(other.remaining());
                self.record_refund(other.refunded());
            }
            ExitReason::Revert(_) => {
                self.erase_cost(other.remaining());
            }
            _ => {}
        }
    }

    pub fn limit_mut(&mut self) -> &mut u64 {
        &mut self.limit
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
        self.used + self.memory
    }

    pub fn remaining(&self) -> u64 {
        (self.limit - self.used) - self.memory
    }

    pub fn erase_cost(&mut self, returned: u64) {
        self.used -= returned;
    }

    pub fn record_refund(&mut self, refund: i64) {
        self.refunded += refund;
    }

    /// Record an explict cost.
    #[inline(always)]
    pub fn record_cost(&mut self, cost: u64) -> bool {
        let all_used_gas: u128 = self.used as u128 + self.memory as u128 + cost as u128;
        if (self.limit as u128) < all_used_gas {
            return false;
        }

        self.used += cost;
        true
    }

    /// used in memory_resize! macro
    pub fn record_memory(&mut self, gas_memory: u64) -> bool {
        let max_memory = max(self.memory, gas_memory);
        let all_used_gas: u128 = self.used as u128 + gas_memory as u128;
        if (self.limit as u128) < all_used_gas {
            return false;
        }
        self.memory = max_memory;
        true
    }

    #[inline(always)]
    pub fn record_cost_control(&mut self, cost: u64) -> Control {
        if !self.record_cost(cost) {
            return Control::Exit(ExitReason::Error(ExitError::OutOfGas));
        }
        Control::Continue
    }

    /// used in gas_refund! macro
    pub fn gas_refund(&mut self, refund: i64) {
        self.refunded += refund;
    }
}

impl Machine {
    pub fn new<SPEC: Spec>(contract: Contract, gas_limit: u64, call_depth: u64) -> Self {
        Self {
            program_counter: 0,
            return_range: Range::default(),
            memory: Memory::new(usize::MAX),
            stack: Stack::new(STACK_LIMIT as usize),
            return_data_buffer: Bytes::new(),
            contract,
            gas: Gas::new(gas_limit),
            call_depth,
        }
    }
    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    pub fn gas(&mut self) -> &Gas {
        &self.gas
    }

    /// Reference of machine stack.
    pub fn stack(&self) -> &Stack {
        &self.stack
    }
    /// Mutable reference of machine stack.
    pub fn stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
    /// Reference of machine memory.
    pub fn memory(&self) -> &Memory {
        &self.memory
    }
    /// Mutable reference of machine memory.
    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }
    /// Return a reference of the program counter.
    pub fn program_counter(&self) -> usize {
        self.program_counter
    }

    /// loop steps until we are finished with execution
    pub fn run<H: Handler, SPEC: Spec>(&mut self, handler: &mut H) -> ExitReason {
        loop {
            if let Err(reason) = self.step::<H, SPEC>(handler) {
                if H::INSPECT {
                    handler.inspect().call_return(reason.clone());
                }
                return reason;
            }
        }
    }

    #[inline]
    /// Step the machine, executing one opcode. It then returns.
    pub fn step<H: Handler, SPEC: Spec>(&mut self, handler: &mut H) -> Result<(), ExitReason> {
        if H::INSPECT {
            handler.inspect().step(self);
        }
        // extract next opcode from code
        let _program_counter = self.program_counter;
        let opcode = self.contract.opcode(self.program_counter)?;

        // evaluate opcode/execute instruction
        let mut eval = eval::<H, SPEC>(self, opcode, self.program_counter, handler);
        if H::INSPECT {
            handler.inspect().eval(&mut eval, self);
        }
        match eval {
            Control::Continue => {
                self.program_counter += 1;
            }
            Control::ContinueN(p) => {
                self.program_counter += p;
            }
            Control::Exit(e) => {
                return Err(e);
            }
            Control::Jump(p) => {
                self.program_counter = p;
            }
        }

        Ok(())
    }

    /// Copy and get the return value of the machine, if any.
    pub fn return_value(&self) -> Bytes {
        if self.return_range.start > U256::from(usize::MAX) {
            let mut ret = Vec::new();
            ret.resize(
                (self.return_range.end - self.return_range.start).as_usize(),
                0,
            );
            Bytes::from(ret)
        } else if self.return_range.end > U256::from(usize::MAX) {
            let mut ret = self
                .memory
                .get(
                    self.return_range.start.as_usize(),
                    usize::MAX - self.return_range.start.as_usize(),
                )
                .to_vec();
            while ret.len() < (self.return_range.end - self.return_range.start).as_usize() {
                ret.push(0);
            }
            Bytes::from(ret)
        } else {
            self.memory.get(
                self.return_range.start.as_usize(),
                (self.return_range.end - self.return_range.start).as_usize(),
            )
        }
    }
}
