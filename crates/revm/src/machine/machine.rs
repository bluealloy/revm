use crate::{
    instructions::{eval, Return},
    return_ok, return_revert, USE_GAS,
};
use bytes::Bytes;
use core::ops::Range;

use super::{contract::Contract, memory::Memory, stack::Stack};
use crate::{spec::Spec, Host};

pub const STACK_LIMIT: u64 = 1024;
pub const CALL_STACK_LIMIT: u64 = 1024;

pub struct Machine {
    /// Contract information and invoking data
    pub contract: Contract,
    /// Program counter.
    pub program_counter: *const u8,
    /// Memory.
    pub memory: Memory,
    /// Stack.
    pub stack: Stack,
    /// left gas. Memory gas can be found in Memory field.
    pub gas: Gas,
    /// After call returns, its return data is saved here.
    pub return_data_buffer: Bytes,
    /// Return value.
    pub return_range: Range<usize>,
    /// used only for inspector.
    pub call_depth: u64,
}

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

impl Machine {
    pub fn new<SPEC: Spec>(contract: Contract, gas_limit: u64, call_depth: u64) -> Self {
        Self {
            program_counter: contract.code.as_ptr(),
            return_range: Range::default(),
            memory: Memory::new(),
            stack: Stack::new(),
            return_data_buffer: Bytes::new(),
            contract,
            gas: Gas::new(gas_limit),
            call_depth,
            //times: [(std::time::Duration::ZERO, 0); 256],
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

    pub fn add_next_gas_block(&mut self, pc: usize) -> Return {
        if USE_GAS {
            let gas_block = self.contract.gas_block(pc);
            if !self.gas.record_cost(gas_block) {
                return Return::OutOfGas;
            }
        }
        Return::Continue
    }

    /// Return a reference of the program counter.
    pub fn program_counter(&self) -> usize {
        // Sefety: this is just substraction of pointers, it is safe to do.
        unsafe {
            self.program_counter
                .offset_from(self.contract.code.as_ptr()) as usize
        }
    }

    /// loop steps until we are finished with execution
    pub fn run<H: Host, SPEC: Spec>(&mut self, host: &mut H) -> Return {
        //let timer = std::time::Instant::now();
        let mut ret = Return::Continue;
        // add first gas_block
        if USE_GAS && !self.gas.record_cost(self.contract.first_gas_block()) {
            return Return::OutOfGas;
        }
        while ret == Return::Continue {
            // step
            if H::INSPECT {
                let ret = host.step(self, SPEC::IS_STATIC_CALL);
                if ret != Return::Continue {
                    return ret;
                }
            }
            let opcode = unsafe { *self.program_counter };
            // Safety: In analazis we are doing padding of bytecode so that we are sure that last.
            // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
            // it will do noop and just stop execution of this contract
            self.program_counter = unsafe { self.program_counter.offset(1) };
            ret = eval::<H, SPEC>(opcode, self, host);

            if H::INSPECT {
                let ret = host.step_end(ret, self);
                if ret != Return::Continue {
                    return ret;
                }
            }
        }
        ret
    }

    /// Copy and get the return value of the machine, if any.
    pub fn return_value(&self) -> Bytes {
        // if start is usize max it means that our return len is zero and we need to return empty
        if self.return_range.start == usize::MAX {
            Bytes::new()
        } else {
            Bytes::copy_from_slice(self.memory.get_slice(
                self.return_range.start,
                self.return_range.end - self.return_range.start,
            ))
        }
    }
}
