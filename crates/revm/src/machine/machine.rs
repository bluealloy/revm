use crate::{
    alloc::vec::Vec,
    instructions::{eval, Return},
    return_ok, return_revert,
};
use bytes::Bytes;
use core::ops::Range;
use primitive_types::U256;

use super::{contract::Contract, memory::Memory, stack::Stack};
use crate::{spec::Spec, Handler};

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
    pub times: [(std::time::Duration, usize); 256],
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
    #[inline(always)]
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
            program_counter: 0,
            return_range: Range::default(),
            memory: Memory::new(usize::MAX),
            stack: Stack::new(),
            return_data_buffer: Bytes::new(),
            contract,
            gas: Gas::new(gas_limit),
            call_depth,
            times: [(std::time::Duration::ZERO, 0); 256],
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

    /// Return a reference of the program counter.
    pub fn program_counter(&self) -> usize {
        self.program_counter
    }

    /// loop steps until we are finished with execution
    pub fn run<H: Handler, SPEC: Spec>(&mut self, handler: &mut H) -> Return {
        //let timer = std::time::Instant::now();
        loop {
            let ret = self.step::<H, SPEC>(handler);
            if Return::Continue != ret {
                if H::INSPECT {
                    handler.inspect().call_return(ret);
                }
                // let elapsed = timer.elapsed();
                // println!("run took:{:?}", elapsed);
                // let mut it = self
                //     .times
                //     .iter()
                //     .zip(crate::OPCODE_JUMPMAP.iter())
                //     .filter(|((time, _), opcode)| opcode.is_some() && !time.is_zero())
                //     .map(|((dur, num), code)| (code.unwrap(), dur, num, *dur / *num as u32))
                //     .collect::<Vec<_>>();
                // it.sort_by(|a, b| a.2.cmp(&b.2));
                // for i in it {
                //     println!(
                //         "code:{:?}   called:{:?}   time:{:?}   avrg:{:?}",
                //         i.0,
                //         i.2,
                //         i.1,
                //         i.3,
                //     );
                // }
                return ret;
            }
        }
    }

    #[inline(always)]
    /// Step the machine, executing one opcode. It then returns.
    pub fn step<H: Handler, SPEC: Spec>(&mut self, handler: &mut H) -> Return {
        if H::INSPECT {
            handler.inspect().step(self);
        }
        // extract next opcode from code
        let opcode = unsafe { *self.contract.code.get_unchecked(self.program_counter) };

        // evaluate opcode/execute instruction
        self.program_counter += 1;
        let eval = eval::<H, SPEC>(self, opcode, handler);
        if H::INSPECT {
            handler.inspect().eval(eval, self);
        }
        eval
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
