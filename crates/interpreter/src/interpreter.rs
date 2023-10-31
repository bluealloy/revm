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
use revm_primitives::{Address, U256};

/// EIP-170: Contract code size limit
///
/// By default this limit is 0x6000 (~25kb)
pub const MAX_CODE_SIZE: usize = 0x6000;

/// EIP-3860: Limit and meter initcode
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;

#[derive(Debug)]
pub struct Interpreter<'a> {
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
    pub shared_memory: Option<&'a mut SharedMemory>,
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
    /// Actions that is expected
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
        return_offset: usize,
        /// The length of the return data.
        return_len: usize,
    },
    Create {
        inputs: Box<CreateInputs>,
    },
    Return {
        result: InterpreterResult,
    },
}

impl<'a> Interpreter<'a> {
    /// Create new interpreter
    pub fn new(contract: Box<Contract>, gas_limit: u64, is_static: bool) -> Self {
        Self {
            instruction_pointer: contract.bytecode.as_ptr(),
            contract,
            gas: Gas::new(gas_limit),
            instruction_result: InstructionResult::Continue,
            is_static,
            return_data_buffer: Bytes::new(),
            shared_memory: None,
            stack: Stack::new(),
            next_action: None,
        }
    }

    /// Returns shared memory.
    pub fn shared_memory(&mut self) -> &mut SharedMemory {
        self.shared_memory.as_mut().unwrap()
    }

    /// When sub create call returns we can insert output of that call into this interpreter.
    ///
    /// Note: SharedMemory is not available here because we are not executing sub call.
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

                if crate::USE_GAS {
                    interpreter.gas.erase_cost(result.gas.remaining());
                    interpreter.gas.record_refund(result.gas.refunded());
                }
            }
            return_revert!() => {
                push!(interpreter, U256::ZERO);

                if crate::USE_GAS {
                    interpreter.gas.erase_cost(result.gas.remaining());
                }
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
    pub fn insert_call_output(
        &mut self,
        shared_memory: &mut SharedMemory,
        result: InterpreterResult,
    ) {
        let (out_offset, out_len) = match self.next_action {
            Some(InterpreterAction::SubCall {
                return_offset,
                return_len,
                ..
            }) => (return_offset, return_len),
            _ => (0, 0),
        };
        let interpreter = self;

        interpreter.return_data_buffer = result.output;
        let target_len = min(out_len, interpreter.return_data_buffer.len());

        match result.result {
            return_ok!() => {
                // return unspend gas.
                if crate::USE_GAS {
                    interpreter.gas.erase_cost(result.gas.remaining());
                    interpreter.gas.record_refund(result.gas.refunded());
                }
                shared_memory.set(out_offset, &interpreter.return_data_buffer[..target_len]);
                push!(interpreter, U256::from(1));
            }
            return_revert!() => {
                if crate::USE_GAS {
                    interpreter.gas.erase_cost(result.gas.remaining());
                }
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
    #[inline(always)]
    pub fn step<FN, H: Host>(&mut self, instruction_table: &[FN; 256], host: &mut H)
    where
        FN: Fn(&mut Interpreter<'_>, &mut H),
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

    pub fn next_action(&mut self) -> InterpreterAction {
        // return next action
        if self.instruction_result == InstructionResult::CallOrCreate {
            // Set instruction result to continue so that run can continue working
            self.instruction_result = InstructionResult::Continue;
            // next action is already set by one of CALL or CREATE instructions.
            // Probably can be done differently without clone, but this is easier.
            self.next_action.clone().unwrap()
        } else {
            InterpreterAction::Return {
                result: InterpreterResult {
                    result: self.instruction_result,
                    output: self.return_data_buffer.clone(),
                    gas: self.gas,
                },
            }
        }
    }

    /// Executes the interpreter until it returns or stops.
    pub fn run<FN, H: Host>(
        &mut self,
        shared_memory: &'a mut SharedMemory,
        instruction_table: &[FN; 256],
        host: &mut H,
    ) -> InterpreterAction
    where
        FN: Fn(&mut Interpreter<'_>, &mut H),
    {
        self.shared_memory = Some(shared_memory);
        // main loop
        while self.instruction_result == InstructionResult::Continue {
            self.step(instruction_table, host);
        }

        self.next_action()
    }
}
