pub mod analysis;
mod contract;
mod shared_memory;
mod stack;

pub use analysis::BytecodeLocked;
pub use contract::Contract;
pub use shared_memory::{next_multiple_of_32, SharedMemory};
pub use stack::{Stack, STACK_LIMIT};

use crate::primitives::Bytes;
use crate::{
    push, push_b256, return_ok, return_revert, CallInputs, CreateInputs, Gas, Host,
    InstructionResult,
};
use alloc::boxed::Box;
use core::cmp::min;
use core::ops::Range;
use revm_primitives::{Address, U256};

pub use self::shared_memory::EMPTY_SHARED_MEMORY;

/// EIP-170: Contract code size limit
///
/// By default this limit is 0x6000 (~25kb)
pub const MAX_CODE_SIZE: usize = 0x6000;

/// EIP-3860: Limit and meter initcode
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;

#[derive(Debug)]
pub struct Interpreter {
    /// Contract information and invoking data
    pub contract: Box<Contract>,
    /// The current instruction pointer.
    pub instruction_pointer: *const u8,
    /// The execution control flag. If this is not set to `Continue`, the interpreter will stop
    /// execution.
    pub instruction_result: InstructionResult,
    /// The gas state.
    pub gas: Gas,
    /// Shared memory.
    ///
    /// Note: This field is only set while running the interpreter loop.
    /// Otherwise it is taken and replaced with empty shared memory.
    pub shared_memory: SharedMemory,
    /// Stack.
    pub stack: Stack,
    /// The return data buffer for internal calls.
    /// It has multi usage:
    ///
    /// * It contains the output bytes of call sub call.
    /// * When this interpreter finishes execution it contains the output bytes of this contract.
    pub return_data_buffer: Bytes,
    /// Whether the interpreter is in "staticcall" mode, meaning no state changes can happen.
    pub is_static: bool,
    /// Actions that the EVM should do.
    ///
    /// Set inside CALL or CREATE instructions and RETURN or REVERT instructions. Additionally those instructions will set
    /// InstructionResult to CallOrCreate/Return/Revert so we know the reason.
    pub next_action: Option<InterpreterAction>,
}

#[derive(Debug, Clone)]
pub struct InterpreterResult {
    pub result: InstructionResult,
    pub output: Bytes,
    pub gas: Gas,
}

#[derive(Debug, Clone)]
pub enum InterpreterAction {
    SubCall {
        /// Call inputs
        inputs: Box<CallInputs>,
        /// The offset into `self.memory` of the return data.
        ///
        /// This value must be ignored if `self.return_len` is 0.
        return_memory_offset: Range<usize>,
    },
    Create {
        inputs: Box<CreateInputs>,
    },
    Return {
        result: InterpreterResult,
    },
}

impl Interpreter {
    /// Create new interpreter
    pub fn new(contract: Box<Contract>, gas_limit: u64, is_static: bool) -> Self {
        Self {
            instruction_pointer: contract.bytecode.as_ptr(),
            contract,
            gas: Gas::new(gas_limit),
            instruction_result: InstructionResult::Continue,
            is_static,
            return_data_buffer: Bytes::new(),
            shared_memory: EMPTY_SHARED_MEMORY,
            stack: Stack::new(),
            next_action: None,
        }
    }

    /// When sub create call returns we can insert output of that call into this interpreter.
    pub fn insert_create_output(&mut self, result: InterpreterResult, address: Option<Address>) {
        let interpreter = self;
        interpreter.return_data_buffer = match result.result {
            // Save data to return data buffer if the create reverted
            return_revert!() => result.output,
            // Otherwise clear it
            _ => Bytes::new(),
        };

        match result.result {
            return_ok!() => {
                push_b256!(interpreter, address.unwrap_or_default().into_word());
                interpreter.gas.erase_cost(result.gas.remaining());
                interpreter.gas.record_refund(result.gas.refunded());
            }
            return_revert!() => {
                push!(interpreter, U256::ZERO);
                interpreter.gas.erase_cost(result.gas.remaining());
            }
            InstructionResult::FatalExternalError => {
                interpreter.instruction_result = InstructionResult::FatalExternalError;
            }
            _ => {
                push!(interpreter, U256::ZERO);
            }
        }
    }

    /// When sub call returns we can insert output of that call into this interpreter.
    ///
    /// Note that shared memory is required as a input field.
    /// As SharedMemory inside Interpreter is taken and replaced with empty (not valid) memory.
    pub fn insert_call_output(
        &mut self,
        shared_memory: &mut SharedMemory,
        result: InterpreterResult,
        memory_return_offset: Range<usize>,
    ) {
        let out_offset = memory_return_offset.start;
        let out_len = memory_return_offset.len();

        let interpreter = self;
        interpreter.return_data_buffer = result.output;
        let target_len = min(out_len, interpreter.return_data_buffer.len());

        match result.result {
            return_ok!() => {
                // return unspend gas.
                interpreter.gas.erase_cost(result.gas.remaining());
                interpreter.gas.record_refund(result.gas.refunded());
                shared_memory.set(out_offset, &interpreter.return_data_buffer[..target_len]);
                push!(interpreter, U256::from(1));
            }
            return_revert!() => {
                interpreter.gas.erase_cost(result.gas.remaining());
                shared_memory.set(out_offset, &interpreter.return_data_buffer[..target_len]);
                push!(interpreter, U256::ZERO);
            }
            InstructionResult::FatalExternalError => {
                interpreter.instruction_result = InstructionResult::FatalExternalError;
            }
            _ => {
                push!(interpreter, U256::ZERO);
            }
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

    /// Returns a reference to the interpreter's stack.
    #[inline]
    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    /// Returns the current program counter.
    #[inline]
    pub fn program_counter(&self) -> usize {
        // SAFETY: `instruction_pointer` should be at an offset from the start of the bytecode.
        // In practice this is always true unless a caller modifies the `instruction_pointer` field manually.
        unsafe {
            self.instruction_pointer
                .offset_from(self.contract.bytecode.as_ptr()) as usize
        }
    }

    /// Executes the instruction at the current instruction pointer.
    ///
    /// Internally it will increment instruction pointer by one.
    #[inline(always)]
    fn step<FN, H: Host>(&mut self, instruction_table: &[FN; 256], host: &mut H)
    where
        FN: Fn(&mut Interpreter, &mut H),
    {
        // Get current opcode.
        let opcode = unsafe { *self.instruction_pointer };

        // Safety: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.instruction_pointer = unsafe { self.instruction_pointer.offset(1) };

        // execute instruction.
        (instruction_table[opcode as usize])(self, host)
    }

    /// Take memory and replace it with empty memory.
    pub fn take_memory(&mut self) -> SharedMemory {
        core::mem::replace(&mut self.shared_memory, EMPTY_SHARED_MEMORY)
    }

    /// Executes the interpreter until it returns or stops.
    pub fn run<FN, H: Host>(
        &mut self,
        shared_memory: SharedMemory,
        instruction_table: &[FN; 256],
        host: &mut H,
    ) -> InterpreterAction
    where
        FN: Fn(&mut Interpreter, &mut H),
    {
        self.next_action = None;
        self.instruction_result = InstructionResult::Continue;
        self.shared_memory = shared_memory;
        // main loop
        while self.instruction_result == InstructionResult::Continue {
            self.step(instruction_table, host);
        }

        // Return next action if it is some.
        if let Some(action) = self.next_action.take() {
            return action;
        }
        // If not, return action without output.
        InterpreterAction::Return {
            result: InterpreterResult {
                result: self.instruction_result,
                // return empty bytecode
                output: Bytes::new(),
                gas: self.gas,
            },
        }
    }
}
