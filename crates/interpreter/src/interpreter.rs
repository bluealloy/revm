pub mod bytecode;
mod contract;
pub(crate) mod memory;
mod stack;

pub use bytecode::{Bytecode, BytecodeLocked, BytecodeState};
pub use contract::Contract;
pub use memory::Memory;
pub use stack::Stack;

use crate::{
    instructions::{eval, Return},
    Gas, Host, Spec, USE_GAS,
};
use bytes::Bytes;
use core::ops::Range;

pub const STACK_LIMIT: u64 = 1024;
pub const CALL_STACK_LIMIT: u64 = 1024;

pub struct Interpreter {
    /// Contract information and invoking data
    pub contract: Contract,
    /// Instruction pointer.
    pub instruction_pointer: *const u8,
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
    /// Return is main control flag, it tell us if we should continue interpreter or break from it
    pub instruction_result: Return,
    /// Is interpreter call static.
    pub is_static: bool,
    /// Memory limit. See [`crate::CfgEnv`].
    #[cfg(feature = "memory_limit")]
    pub memory_limit: u64,
}

impl Interpreter {
    pub fn current_opcode(&self) -> u8 {
        unsafe { *self.instruction_pointer }
    }

    pub fn new<SPEC: Spec>(contract: Contract, gas_limit: u64, is_static: bool) -> Self {
        #[cfg(not(feature = "memory_limit"))]
        {
            Self {
                instruction_pointer: contract.bytecode.as_ptr(),
                return_range: Range::default(),
                memory: Memory::new(),
                stack: Stack::new(),
                return_data_buffer: Bytes::new(),
                contract,
                instruction_result: Return::Continue,
                is_static,
                gas: Gas::new(gas_limit),
            }
        }

        #[cfg(feature = "memory_limit")]
        {
            Self::new_with_memory_limit::<SPEC>(contract, gas_limit, is_static, u64::MAX)
        }
    }

    #[cfg(feature = "memory_limit")]
    pub fn new_with_memory_limit<SPEC: Spec>(
        contract: Contract,
        gas_limit: u64,
        is_static: bool,
        memory_limit: u64,
    ) -> Self {
        Self {
            instruction_pointer: contract.bytecode.as_ptr(),
            return_range: Range::default(),
            memory: Memory::new(),
            stack: Stack::new(),
            return_data_buffer: Bytes::new(),
            contract,
            instruction_result: Return::Continue,
            is_static,
            gas: Gas::new(gas_limit),
            memory_limit,
        }
    }

    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    pub fn gas(&self) -> &Gas {
        &self.gas
    }

    /// Reference of interp stack.
    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    #[inline(always)]
    pub fn add_next_gas_block(&mut self, pc: usize) -> Option<Return> {
        if USE_GAS {
            let gas_block = self.contract.gas_block(pc);
            if !self.gas.record_cost(gas_block) {
                return Some(Return::OutOfGas);
            }
        }
        None
    }

    /// Return a reference of the program counter.
    pub fn program_counter(&self) -> usize {
        // Safety: this is just subtraction of pointers, it is safe to do.
        unsafe {
            self.instruction_pointer
                .offset_from(self.contract.bytecode.as_ptr()) as usize
        }
    }

    /// loop steps until we are finished with execution
    pub fn run<H: Host, SPEC: Spec>(&mut self, host: &mut H) -> Return {
        // add first gas_block
        if USE_GAS && !self.gas.record_cost(self.contract.first_gas_block()) {
            return Return::OutOfGas;
        }
        while self.instruction_result == Return::Continue {
            // step.
            let opcode = unsafe { *self.instruction_pointer };
            // Safety: In analysis we are doing padding of bytecode so that we are sure that last
            // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
            // it will do noop and just stop execution of this contract
            self.instruction_pointer = unsafe { self.instruction_pointer.offset(1) };
            eval::<H, SPEC>(opcode, self, host);
        }
        self.instruction_result
    }

    /// loop steps until we are finished with execution
    pub fn run_inspect<H: Host, SPEC: Spec>(&mut self, host: &mut H) -> Return {
        // add first gas_block
        if USE_GAS && !self.gas.record_cost(self.contract.first_gas_block()) {
            return Return::OutOfGas;
        }
        while self.instruction_result == Return::Continue {
            // step
            let ret = host.step(self, self.is_static);
            if ret != Return::Continue {
                return ret;
            }
            let opcode = unsafe { *self.instruction_pointer };
            // Safety: In analysis we are doing padding of bytecode so that we are sure that last.
            // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
            // it will do noop and just stop execution of this contract
            self.instruction_pointer = unsafe { self.instruction_pointer.offset(1) };
            eval::<H, SPEC>(opcode, self, host);

            // step ends
            let ret = host.step_end(self, self.is_static, self.instruction_result);
            if ret != Return::Continue {
                return ret;
            }
        }
        self.instruction_result
    }

    /// Copy and get the return value of the interp, if any.
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
