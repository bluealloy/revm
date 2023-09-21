pub mod analysis;
mod contract;
pub mod memory;
mod stack;

use crate::primitives::{Bytes, Spec};
use crate::{alloc::boxed::Box, opcode::eval, Gas, Host, InstructionResult};

pub use analysis::BytecodeLocked;
pub use contract::Contract;
pub use memory::Memory;
pub use stack::{Stack, STACK_LIMIT};

pub const CALL_STACK_LIMIT: u64 = 1024;

/// EIP-170: Contract code size limit
///
/// By default this limit is 0x6000 (~25kb)
pub const MAX_CODE_SIZE: usize = 0x6000;

/// EIP-3860: Limit and meter initcode
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;

#[derive(Debug)]
pub struct Interpreter {
    /// Contract information and invoking data.
    pub contract: Box<Contract>,
    /// The current instruction pointer.
    pub instruction_pointer: *const u8,
    /// The execution control flag. If this is not set to `Continue`, the interpreter will stop
    /// execution.
    pub instruction_result: InstructionResult,
    /// The gas state.
    pub gas: Gas,
    /// The memory.
    pub memory: Memory,
    /// The stack.
    pub stack: Stack,
    /// The return data buffer for internal calls.
    pub return_data_buffer: Bytes,
    /// The offset into `self.memory` of the return data.
    ///
    /// This value must be ignored if `self.return_len` is 0.
    pub return_offset: usize,
    /// The length of the return data.
    pub return_len: usize,
    /// Whether the interpreter is in "staticcall" mode, meaning no state changes can happen.
    pub is_static: bool,
    /// Memory limit. See [`crate::CfgEnv`].
    #[cfg(feature = "memory_limit")]
    pub memory_limit: u64,
}

impl Interpreter {
    /// Instantiates a new interpreter.
    #[inline]
    pub fn new(contract: Box<Contract>, gas_limit: u64, is_static: bool) -> Self {
        Self {
            instruction_pointer: contract.bytecode.as_ptr(),
            contract,
            instruction_result: InstructionResult::Continue,
            gas: Gas::new(gas_limit),
            memory: Memory::new(),
            stack: Stack::new(),
            return_data_buffer: Bytes::new(),
            return_offset: 0,
            return_len: 0,
            is_static,
            #[cfg(feature = "memory_limit")]
            memory_limit: u64::MAX,
        }
    }

    /// Instantiates a new interpreter with the given memory limit.
    #[cfg(feature = "memory_limit")]
    #[inline]
    pub fn new_with_memory_limit(
        contract: Box<Contract>,
        gas_limit: u64,
        is_static: bool,
        memory_limit: u64,
    ) -> Self {
        Self {
            memory_limit,
            ..Self::new(contract, gas_limit, is_static)
        }
    }

    /// Returns the opcode at the current instruction pointer.
    #[inline]
    pub fn current_opcode(&self) -> u8 {
        unsafe { *self.instruction_pointer }
    }

    /// Returns a reference to the contract.
    #[inline]
    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    /// Returns a reference to the interpreter's gas state.
    #[inline]
    pub fn gas(&self) -> &Gas {
        &self.gas
    }

    /// Returns a reference to the interpreter's memory.
    #[inline]
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    /// Returns a reference to the interpreter's stack.
    #[inline]
    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    /// Returns the current program counter.
    #[inline]
    pub fn program_counter(&self) -> usize {
        // Safety: this is just subtraction of pointers, it is safe to do.
        unsafe {
            self.instruction_pointer
                .offset_from(self.contract.bytecode.as_ptr()) as usize
        }
    }

    /// Executes the instruction at the current instruction pointer.
    #[inline(always)]
    pub fn step<H: Host, SPEC: Spec>(&mut self, host: &mut H) {
        // step.
        let opcode = unsafe { *self.instruction_pointer };
        // Safety: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.instruction_pointer = unsafe { self.instruction_pointer.offset(1) };
        eval::<H, SPEC>(opcode, self, host);
    }

    /// Executes the interpreter until it returns or stops.
    pub fn run<H: Host, SPEC: Spec>(&mut self, host: &mut H) -> InstructionResult {
        while self.instruction_result == InstructionResult::Continue {
            self.step::<H, SPEC>(host);
        }
        self.instruction_result
    }

    /// Executes the interpreter until it returns or stops. Same as `run` but with
    /// calls to the [`Host::step`] and [`Host::step_end`] callbacks.
    pub fn run_inspect<H: Host, SPEC: Spec>(&mut self, host: &mut H) -> InstructionResult {
        while self.instruction_result == InstructionResult::Continue {
            // step
            let result = host.step(self);
            if result != InstructionResult::Continue {
                return result;
            }

            self.step::<H, SPEC>(host);

            // step ends
            let result = host.step_end(self, self.instruction_result);
            if result != InstructionResult::Continue {
                return result;
            }
        }
        self.instruction_result
    }

    /// Returns a copy of the interpreter's return value, if any.
    #[inline]
    pub fn return_value(&self) -> Bytes {
        self.return_value_slice().to_vec().into()
    }

    /// Returns a reference to the interpreter's return value, if any.
    #[inline]
    pub fn return_value_slice(&self) -> &[u8] {
        if self.return_len == 0 {
            &[]
        } else {
            self.memory.slice(self.return_offset, self.return_len)
        }
    }
}
