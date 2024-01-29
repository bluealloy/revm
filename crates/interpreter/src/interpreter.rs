pub mod analysis;
mod contract;
mod shared_memory;
mod stack;

pub use analysis::BytecodeLocked;
pub use contract::Contract;
pub use shared_memory::{next_multiple_of_32, SharedMemory};
pub use stack::{Stack, STACK_LIMIT};

use crate::alloc::borrow::ToOwned;
use crate::{
    primitives::Bytes, push, push_b256, return_ok, return_revert, CallInputs, CallOutcome,
    CreateInputs, CreateOutcome, Gas, Host, InstructionResult,
};
use alloc::boxed::Box;
use core::cmp::min;
use core::ops::Range;
use revm_primitives::U256;

pub use self::shared_memory::EMPTY_SHARED_MEMORY;

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
    pub next_action: InterpreterAction,
}

/// The result of an interpreter operation.
#[derive(Debug, Clone)]
pub struct InterpreterResult {
    /// The result of the instruction execution.
    pub result: InstructionResult,
    /// The output of the instruction execution.
    pub output: Bytes,
    /// The gas usage information.
    pub gas: Gas,
}

#[derive(Debug, Default, Clone)]
pub enum InterpreterAction {
    /// CALL, CALLCODE, DELEGATECALL or STATICCALL instruction called.
    Call {
        /// Call inputs
        inputs: Box<CallInputs>,
        /// The offset into `self.memory` of the return data.
        ///
        /// This value must be ignored if `self.return_len` is 0.
        return_memory_offset: Range<usize>,
    },
    /// CREATE or CREATE2 instruction called.
    Create { inputs: Box<CreateInputs> },
    /// Interpreter finished execution.
    Return { result: InterpreterResult },
    /// No action
    #[default]
    None,
}

impl InterpreterAction {
    /// Returns true if action is call.
    pub fn is_call(&self) -> bool {
        matches!(self, InterpreterAction::Call { .. })
    }

    /// Returns true if action is create.
    pub fn is_create(&self) -> bool {
        matches!(self, InterpreterAction::Create { .. })
    }

    /// Returns true if action is return.
    pub fn is_return(&self) -> bool {
        matches!(self, InterpreterAction::Return { .. })
    }

    /// Returns true if action is none.
    pub fn is_none(&self) -> bool {
        matches!(self, InterpreterAction::None)
    }

    /// Returns true if action is some.
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    /// Returns result if action is return.
    pub fn into_result_return(self) -> Option<InterpreterResult> {
        match self {
            InterpreterAction::Return { result } => Some(result),
            _ => None,
        }
    }
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
            next_action: InterpreterAction::None,
        }
    }

    /// Inserts the output of a `create` call into the interpreter.
    ///
    /// This function is used after a `create` call has been executed. It processes the outcome
    /// of that call and updates the state of the interpreter accordingly.
    ///
    /// # Arguments
    ///
    /// * `create_outcome` - A `CreateOutcome` struct containing the results of the `create` call.
    ///
    /// # Behavior
    ///
    /// The function updates the `return_data_buffer` with the data from `create_outcome`.
    /// Depending on the `InstructionResult` indicated by `create_outcome`, it performs one of the following:
    ///
    /// - `Ok`: Pushes the address from `create_outcome` to the stack, updates gas costs, and records any gas refunds.
    /// - `Revert`: Pushes `U256::ZERO` to the stack and updates gas costs.
    /// - `FatalExternalError`: Sets the `instruction_result` to `InstructionResult::FatalExternalError`.
    /// - `Default`: Pushes `U256::ZERO` to the stack.
    ///
    /// # Side Effects
    ///
    /// - Updates `return_data_buffer` with the data from `create_outcome`.
    /// - Modifies the stack by pushing values depending on the `InstructionResult`.
    /// - Updates gas costs and records refunds in the interpreter's `gas` field.
    /// - May alter `instruction_result` in case of external errors.
    pub fn insert_create_outcome(&mut self, create_outcome: CreateOutcome) {
        let instruction_result = create_outcome.instruction_result();

        self.return_data_buffer = if instruction_result.is_revert() {
            // Save data to return data buffer if the create reverted
            create_outcome.output().to_owned()
        } else {
            // Otherwise clear it
            Bytes::new()
        };

        match instruction_result {
            return_ok!() => {
                let address = create_outcome.address;
                push_b256!(self, address.unwrap_or_default().into_word());
                self.gas.erase_cost(create_outcome.gas().remaining());
                self.gas.record_refund(create_outcome.gas().refunded());
            }
            return_revert!() => {
                push!(self, U256::ZERO);
                self.gas.erase_cost(create_outcome.gas().remaining());
            }
            InstructionResult::FatalExternalError => {
                self.instruction_result = InstructionResult::FatalExternalError;
            }
            _ => {
                push!(self, U256::ZERO);
            }
        }
    }

    /// Inserts the outcome of a call into the virtual machine's state.
    ///
    /// This function takes the result of a call, represented by `CallOutcome`,
    /// and updates the virtual machine's state accordingly. It involves updating
    /// the return data buffer, handling gas accounting, and setting the memory
    /// in shared storage based on the outcome of the call.
    ///
    /// # Arguments
    ///
    /// * `shared_memory` - A mutable reference to the shared memory used by the virtual machine.
    /// * `call_outcome` - The outcome of the call to be processed, containing details such as
    ///   instruction result, gas information, and output data.
    ///
    /// # Behavior
    ///
    /// The function first copies the output data from the call outcome to the virtual machine's
    /// return data buffer. It then checks the instruction result from the call outcome:
    ///
    /// - `return_ok!()`: Processes successful execution, refunds gas, and updates shared memory.
    /// - `return_revert!()`: Handles a revert by only updating the gas usage and shared memory.
    /// - `InstructionResult::FatalExternalError`: Sets the instruction result to a fatal external error.
    /// - Any other result: No specific action is taken.
    pub fn insert_call_outcome(
        &mut self,
        shared_memory: &mut SharedMemory,
        call_outcome: CallOutcome,
    ) {
        let out_offset = call_outcome.memory_start();
        let out_len = call_outcome.memory_length();

        self.return_data_buffer = call_outcome.output().to_owned();
        let target_len = min(out_len, self.return_data_buffer.len());

        match call_outcome.instruction_result() {
            return_ok!() => {
                // return unspend gas.
                let remaining = call_outcome.gas().remaining();
                let refunded = call_outcome.gas().refunded();
                self.gas.erase_cost(remaining);
                self.gas.record_refund(refunded);
                shared_memory.set(out_offset, &self.return_data_buffer[..target_len]);
                push!(self, U256::from(1));
            }
            return_revert!() => {
                self.gas.erase_cost(call_outcome.gas().remaining());
                shared_memory.set(out_offset, &self.return_data_buffer[..target_len]);
                push!(self, U256::ZERO);
            }
            InstructionResult::FatalExternalError => {
                self.instruction_result = InstructionResult::FatalExternalError;
            }
            _ => {
                push!(self, U256::ZERO);
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

        // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
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
        self.next_action = InterpreterAction::None;
        self.instruction_result = InstructionResult::Continue;
        self.shared_memory = shared_memory;
        // main loop
        while self.instruction_result == InstructionResult::Continue {
            self.step(instruction_table, host);
        }

        // Return next action if it is some.
        if self.next_action.is_some() {
            return core::mem::take(&mut self.next_action);
        }
        // If not, return action without output as it is a halt.
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

impl InterpreterResult {
    /// Returns whether the instruction result is a success.
    #[inline]
    pub const fn is_ok(&self) -> bool {
        self.result.is_ok()
    }

    /// Returns whether the instruction result is a revert.
    #[inline]
    pub const fn is_revert(&self) -> bool {
        self.result.is_revert()
    }

    /// Returns whether the instruction result is an error.
    #[inline]
    pub const fn is_error(&self) -> bool {
        self.result.is_error()
    }
}
