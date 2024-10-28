mod contract;
#[cfg(feature = "serde")]
pub mod serde;
mod shared_memory;
mod stack;

use bytecode::eof::TypesSection;
pub use contract::Contract;
pub use shared_memory::{num_words, SharedMemory, EMPTY_SHARED_MEMORY};
use specification::hardfork::SpecId;
pub use stack::{Stack, STACK_LIMIT};

use crate::instructions::utility::{read_i16, read_u16};
use crate::{
    gas, push, push_b256, return_ok, return_revert, CallOutcome, CreateOutcome,
    FunctionReturnFrame, FunctionStack, Gas, Host, InstructionResult, InterpreterAction,
};
use bytecode::{Bytecode, Eof};
use core::{cmp::min, ops::Range};
use primitives::{Address, Bytes, B256, U256};
use std::borrow::ToOwned;
use std::sync::Arc;

pub struct RuntimeFlags {
    pub is_static: bool,
    pub is_eof: bool,
    pub is_eof_init: bool,
}

/// Helper function to read immediates data from the bytecode.
pub trait Immediates {
    fn read_i16(&self) -> i16;
    fn read_u16(&self) -> u16;

    fn read_i8(&self) -> i8;
    fn read_u8(&self) -> u8;

    fn read_offset_i16(&self, offset: isize) -> i16;
    fn read_offset_u16(&self, offset: isize) -> u16;

    fn read_slice(&self, len: usize) -> &[u8];
}

pub trait InputsTrait {
    fn target_address(&self) -> Address;
    fn caller_address(&self) -> Address;
    fn input(&self) -> &[u8];
    fn call_value(&self) -> U256;
}

pub trait LegacyBytecode {
    fn bytecode_len(&self) -> usize;
    fn bytecode_slice(&self) -> &[u8];
}

/// Trait for interpreter to be able to jump.
pub trait Jumps {
    /// Relative jumps does not require checking for overflow
    fn relative_jump(&mut self, offset: isize);
    /// Absolute jumps require checking for overflow and if target is a jump destination
    /// from jump table.
    fn absolute_jump(&mut self, offset: usize);
    /// Check legacy jump destionation from jump table.
    fn is_valid_legacy_jump(&mut self, offset: usize) -> bool;
    /// Return current program counter.
    fn pc(&self) -> usize;
}

pub trait MemoryTrait {
    fn mem_set_data(&mut self, memory_offset: usize, data_offset: usize, len: usize, data: &[u8]);
    fn mem_set(&mut self, memory_offset: usize, data: &[u8]);
    //fn mem_slice(&self, offset: usize, len: usize) -> &[u8];
    fn mem_size(&self) -> usize;
    fn mem_copy(&mut self, destination: usize, source: usize, len: usize);

    /// Memory slice with range.
    ///
    /// # Panics
    ///
    /// Panics if range is out of scope of allocated memory.
    fn mem_slice(&self, range: Range<usize>) -> &[u8];

    /// Memory slice len
    ///
    /// Uses [`MemoryTrait::mem_slice`] internally.
    fn mem_slice_len(&self, offset: usize, len: usize) -> &[u8] {
        self.mem_slice(offset..offset + len)
    }

    /// Resize memory to new size.
    ///
    /// # Note
    ///
    /// It checks memory limits.
    fn mem_resize(&mut self, new_size: usize) -> bool;
}

pub trait EofContainer {
    fn eof_container(&self, index: usize) -> Option<&Bytes>;
}

pub trait EofSubRoutine {
    fn subroutine_stack_len(&self) -> usize;

    /// Pushes a new frame to the stack and new code index.
    fn subroutine_push(&mut self, program_counter: usize, new_idx: usize) -> Option<usize>;

    /// Pops subroutine frame, sets previous code index and returns program counter.
    fn subroutine_pop(&mut self) -> Option<usize>;

    /// Sets new code section without touching subroutine stack.
    /// Returns start of this new code section.
    fn set_current_code_section_idx(&mut self, idx: usize) -> Option<usize>;

    /// Return code info from EOF body.
    fn eof_code_info(&self, idx: usize) -> Option<&TypesSection>;
}

pub trait StackTrait {
    /// Pushes values to the stack
    /// Return `true` if push was successful, `false` if stack overflow.
    ///
    /// # Note
    ///
    /// Error is internally set in interpreter.
    fn push(&mut self, value: U256) -> bool;

    /// Returns stack length.
    fn stack_len(&self) -> usize;

    /// Pop value from the stack.
    fn popn<const N: usize>(&mut self) -> Option<[U256; N]>;

    /// Pop N values from the stack and return top value.
    fn popn_top<const POPN: usize>(&mut self) -> Option<([U256; POPN], &mut U256)>;

    /// Return top value from the stack.
    fn top(&mut self) -> Option<&mut U256> {
        self.popn_top::<0>().map(|(_, top)| top)
    }

    /// Pop one value from the stack.
    fn pop(&mut self) -> Option<U256> {
        self.popn::<1>().map(|[value]| value)
    }

    /// Reads N bytes from bytecode and pushes it into stack.
    ///
    /// As pushn is very frequently used, we have this specialized implementation.
    fn pushn(&mut self, size: usize) -> bool;

    /// Exchange two values on the stack.
    ///
    /// Indexes are based from the top of the stack.
    ///
    /// Return `true` if swap was successful, `false` if stack underflow.
    fn exchange(&mut self, n: usize, m: usize) -> bool;

    /// Duplicates the `N`th value from the top of the stack.
    ///
    /// Index is based from the top of the stack.
    ///
    /// Return `true` if duplicate was successful, `false` if stack underflow.
    fn dup(&mut self, n: usize) -> bool;
}

pub trait SubRoutine {}

pub trait EofData {
    fn eof_data(&self) -> &[u8];
    fn eof_data_slice(&self, offset: usize, len: usize) -> &[u8];
    fn eof_data_size(&self) -> usize;
}

pub trait ReturnDataBuffer {
    fn return_data_buffer(&self) -> &[u8];
    fn return_data_buffer_mut(&mut self) -> &mut Bytes;
}

/// TODO wip probably left for follow up PR.
/// Ides is to have trait inside instruction so that Interpreter can
/// be extended even more.
pub trait InterpreterTrait:
    Immediates
    + Jumps
    + StackTrait
    + LegacyBytecode
    + ReturnDataBuffer
    + EofData
    + MemoryTrait
    + InputsTrait
    + EofContainer
    + EofSubRoutine
{
    fn gas(&mut self) -> &mut Gas;
    fn set_instruction_result(&mut self, result: InstructionResult);
    fn set_next_action(&mut self, action: InterpreterAction, result: InstructionResult);

    fn bytecode(&self) -> &Bytecode;
    fn spec_id(&self) -> SpecId;

    fn is_eof(&self) -> bool;
    fn is_static(&self) -> bool;
    fn is_eof_init(&self) -> bool;

    fn jump(&mut self, offset: i32);
}

pub trait Interp {
    type Instruction;
    type Action;

    fn run(&mut self, instructions: &[Self::Instruction; 256]) -> Self::Action;
}

/// EVM bytecode interpreter.
#[derive(Debug)]
pub struct Interpreter {
    /// The current instruction pointer.
    pub instruction_pointer: *const u8,
    /// The gas state.
    pub gas: Gas,
    /// Contract information and invoking data
    pub contract: Contract,
    /// The execution control flag. If this is not set to `Continue`, the interpreter will stop
    /// execution.
    pub instruction_result: InstructionResult,
    /// Currently run Bytecode that instruction result will point to.
    /// Bytecode is owned by the contract.
    pub bytecode: Bytes,
    /// Whether we are Interpreting the Ethereum Object Format (EOF) bytecode.
    /// This is local field that is set from `contract.is_eof()`.
    pub is_eof: bool,
    /// Is init flag for eof create
    pub is_eof_init: bool,
    /// Runtime flags
    /// Shared memory.
    ///
    /// Note: This field is only set while running the interpreter loop.
    /// Otherwise it is taken and replaced with empty shared memory.
    pub shared_memory: SharedMemory,
    /// Stack.
    pub stack: Stack,
    /// EOF function stack.
    pub function_stack: FunctionStack,
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
    /// SPEC ID
    pub spec_id: SpecId,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new(Contract::default(), u64::MAX, false)
    }
}

impl EofContainer for Interpreter {
    fn eof_container(&self, index: usize) -> Option<&Bytes> {
        self.contract
            .bytecode
            .eof()
            .map(|eof| eof.body.container_section.get(index))
            .flatten()
    }
}

impl EofSubRoutine for Interpreter {
    fn subroutine_stack_len(&self) -> usize {
        self.function_stack.return_stack_len()
    }

    fn subroutine_push(&mut self, program_counter: usize, new_idx: usize) -> Option<usize> {
        if self.function_stack.len() >= 1024 {
            self.set_instruction_result(InstructionResult::EOFFunctionStackOverflow);
            return None;
        }
        self.function_stack.push(program_counter, new_idx);
        self.eof()
            .expect("It is EOF")
            .body
            .eof_code_section_start(new_idx)
    }

    fn subroutine_pop(&mut self) -> Option<usize> {
        self.function_stack.pop()
    }

    fn eof_code_info(&self, idx: usize) -> Option<&TypesSection> {
        self.eof()
            .map(|eof| eof.body.types_section.get(idx))
            .flatten()
    }

    fn set_current_code_section_idx(&mut self, idx: usize) -> Option<usize> {
        self.function_stack.current_code_idx = idx;
        self.eof()
            .expect("It is EOF")
            .body
            .eof_code_section_start(idx)
    }
}

impl ReturnDataBuffer for Interpreter {
    fn return_data_buffer(&self) -> &[u8] {
        &self.return_data_buffer
    }

    fn return_data_buffer_mut(&mut self) -> &mut Bytes {
        &mut self.return_data_buffer
    }
}

impl LegacyBytecode for Interpreter {
    fn bytecode_len(&self) -> usize {
        // Inform the optimizer that the bytecode cannot be EOF to remove a bounds check.
        assume!(!self.is_eof());
        self.bytecode.len()
    }

    fn bytecode_slice(&self) -> &[u8] {
        // Inform the optimizer that the bytecode cannot be EOF to remove a bounds check.
        assume!(!self.is_eof());
        self.contract.bytecode.original_byte_slice()
    }
}

impl InputsTrait for Interpreter {
    fn target_address(&self) -> Address {
        self.contract.target_address
    }

    fn caller_address(&self) -> Address {
        self.contract.caller
    }

    fn input(&self) -> &[u8] {
        self.contract.input.as_ref()
    }

    fn call_value(&self) -> U256 {
        self.contract.call_value
    }
}

impl MemoryTrait for Interpreter {
    fn mem_set(&mut self, memory_offset: usize, data: &[u8]) {
        self.shared_memory.set(memory_offset, data);
    }

    fn mem_size(&self) -> usize {
        self.shared_memory.len()
    }

    fn mem_set_data(&mut self, memory_offset: usize, data_offset: usize, len: usize, data: &[u8]) {
        self.shared_memory
            .set_data(memory_offset, data_offset, len, data);
    }

    fn mem_slice(&self, range: Range<usize>) -> &[u8] {
        self.shared_memory.slice_range(range)
    }

    fn mem_copy(&mut self, destination: usize, source: usize, len: usize) {
        self.shared_memory.copy(destination, source, len);
    }

    fn mem_resize(&mut self, new_size: usize) -> bool {
        // Increment and check gas consumption before incrementing memory.
        // This opeations are safe because gas is a limiter.
        let new_size = num_words(new_size as u64) * 32;
        // TODO add memory limit here.
        // TODO set interpreter result in case of error
        self.shared_memory.resize(new_size as usize);
        true
    }
}

impl EofData for Interpreter {
    fn eof_data(&self) -> &[u8] {
        self.contract.bytecode.eof().expect("eof").data()
    }

    fn eof_data_slice(&self, offset: usize, len: usize) -> &[u8] {
        self.contract
            .bytecode
            .eof()
            .expect("eof")
            .data_slice(offset, len)
    }

    fn eof_data_size(&self) -> usize {
        self.eof().expect("eof").header.data_size as usize
    }
}

impl Immediates for Interpreter {
    fn read_i16(&self) -> i16 {
        unsafe { read_i16(self.instruction_pointer) }
    }

    fn read_u16(&self) -> u16 {
        unsafe { read_u16(self.instruction_pointer) }
    }

    fn read_i8(&self) -> i8 {
        unsafe { core::mem::transmute(*self.instruction_pointer) }
    }

    fn read_u8(&self) -> u8 {
        unsafe { *self.instruction_pointer }
    }

    fn read_slice(&self, len: usize) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.instruction_pointer, len) }
    }

    fn read_offset_i16(&self, offset: isize) -> i16 {
        unsafe {
            read_i16(
                self.instruction_pointer
                    // offset for max_index that is one byte
                    .offset(offset),
            )
        }
    }
    fn read_offset_u16(&self, offset: isize) -> u16 {
        unsafe {
            read_u16(
                self.instruction_pointer
                    // offset for max_index that is one byte
                    .offset(offset),
            )
        }
    }
}

impl Jumps for Interpreter {
    fn relative_jump(&mut self, offset: isize) {
        self.instruction_pointer = unsafe { self.instruction_pointer.offset(offset) };
    }

    fn absolute_jump(&mut self, offset: usize) {
        self.instruction_pointer = unsafe { self.bytecode.as_ptr().add(offset) };
    }

    fn is_valid_legacy_jump(&mut self, offset: usize) -> bool {
        self.contract.is_valid_jump(offset)
    }

    fn pc(&self) -> usize {
        self.program_counter()
    }
}

impl StackTrait for Interpreter {
    fn stack_len(&self) -> usize {
        self.stack.len()
    }

    fn pushn(&mut self, num: usize) -> bool {
        true
    }

    #[inline]
    fn popn<const N: usize>(&mut self) -> Option<[U256; N]> {
        if self.stack.len() < N {
            self.set_instruction_result(InstructionResult::StackUnderflow);
            return None;
        }
        // SAFETY: stack length is checked above.
        Some(unsafe { self.stack.popn::<N>() })
    }

    #[inline]
    fn popn_top<const POPN: usize>(&mut self) -> Option<([U256; POPN], &mut U256)> {
        if self.stack.len() < POPN + 1 {
            self.set_instruction_result(InstructionResult::StackUnderflow);
            return None;
        }
        // SAFETY: stack length is checked above.
        Some(unsafe { self.stack.popn_top::<POPN>() })
    }

    fn exchange(&mut self, n: usize, m: usize) -> bool {
        if let Err(instruction_result) = self.stack.exchange(n, m) {
            self.set_instruction_result(instruction_result);
            return false;
        }
        return true;
    }

    fn dup(&mut self, n: usize) -> bool {
        if let Err(instruction_result) = self.stack.dup(n) {
            self.set_instruction_result(instruction_result);
            return false;
        }
        true
    }

    fn push(&mut self, value: U256) -> bool {
        if let Err(e) = self.stack.push(value) {
            self.set_instruction_result(e);
            return false;
        }
        true
    }
}

impl InterpreterTrait for Interpreter {
    fn gas(&mut self) -> &mut Gas {
        &mut self.gas
    }

    fn set_next_action(&mut self, action: InterpreterAction, result: InstructionResult) {
        self.set_instruction_result(result);
        self.next_action = action;
    }

    fn set_instruction_result(&mut self, result: InstructionResult) {
        self.instruction_result = result;
    }

    fn bytecode(&self) -> &Bytecode {
        &self.contract.bytecode
    }

    fn spec_id(&self) -> SpecId {
        self.spec_id
    }

    fn jump(&mut self, offset: i32) {
        let offset = offset as isize;
        self.instruction_pointer = unsafe { self.instruction_pointer.offset(offset) };
    }

    fn is_eof(&self) -> bool {
        self.is_eof
    }

    fn is_static(&self) -> bool {
        self.is_static
    }

    fn is_eof_init(&self) -> bool {
        self.is_eof_init
    }
}
/*
Frame with generic bytecode

Frame needs to have generic dispatch.

Interpreter {
    Operational {
        stack
        memory
        instruction_pointer.
    }

    EOF operations {
        function stack
    }

    control: {
        NextAction.
        Instruction result,
    }

    Info {
        contract target/invoker.
        bytecode hash.
        etc.
    }

    Runtime Flags {
        is_static: bool,
        is_eof: bool,
        is_eof_init: bool,
    }
}


*/

impl Interpreter {
    /// Create new interpreter
    pub fn new(contract: Contract, gas_limit: u64, is_static: bool) -> Self {
        if !contract.bytecode.is_execution_ready() {
            panic!("Contract is not execution ready {:?}", contract.bytecode);
        }
        let is_eof = contract.bytecode.is_eof();
        let bytecode = contract.bytecode.bytecode().clone();
        Self {
            instruction_pointer: bytecode.as_ptr(),
            bytecode,
            contract,
            gas: Gas::new(gas_limit),
            instruction_result: InstructionResult::Continue,
            function_stack: FunctionStack::default(),
            is_static,
            is_eof,
            is_eof_init: false,
            return_data_buffer: Bytes::new(),
            shared_memory: EMPTY_SHARED_MEMORY,
            stack: Stack::new(),
            next_action: InterpreterAction::None,
            // TODO set this in constructor
            spec_id: SpecId::LATEST,
        }
    }

    /// Set is_eof_init to true, this is used to enable `RETURNCONTRACT` opcode.
    #[inline]
    pub fn set_is_eof_init(&mut self) {
        self.is_eof_init = true;
    }

    #[inline]
    pub fn eof(&self) -> Option<&Arc<Eof>> {
        self.contract.bytecode.eof()
    }

    /// Test related helper
    #[cfg(test)]
    pub fn new_bytecode(bytecode: Bytecode) -> Self {
        Self::new(
            Contract::new(
                Bytes::new(),
                bytecode,
                None,
                primitives::Address::default(),
                None,
                primitives::Address::default(),
                U256::ZERO,
            ),
            0,
            false,
        )
    }

    /// Load EOF code into interpreter. PC is assumed to be correctly set
    pub(crate) fn load_eof_code(&mut self, idx: usize, pc: usize) {
        // SAFETY: eof flag is true only if bytecode is Eof.
        let Bytecode::Eof(eof) = &self.contract.bytecode else {
            panic!("Expected EOF code section")
        };
        let Some(code) = eof.body.code(idx) else {
            panic!("Code not found")
        };
        self.bytecode = code.clone();
        self.instruction_pointer = unsafe { self.bytecode.as_ptr().add(pc) };
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
        self.instruction_result = InstructionResult::Continue;

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
                panic!("Fatal external error in insert_create_outcome");
            }
            _ => {
                push!(self, U256::ZERO);
            }
        }
    }

    pub fn insert_eofcreate_outcome(&mut self, create_outcome: CreateOutcome) {
        self.instruction_result = InstructionResult::Continue;
        let instruction_result = create_outcome.instruction_result();

        self.return_data_buffer = if *instruction_result == InstructionResult::Revert {
            // Save data to return data buffer if the create reverted
            create_outcome.output().to_owned()
        } else {
            // Otherwise clear it. Note that RETURN opcode should abort.
            Bytes::new()
        };

        match instruction_result {
            InstructionResult::ReturnContract => {
                push_b256!(
                    self,
                    create_outcome.address.expect("EOF Address").into_word()
                );
                self.gas.erase_cost(create_outcome.gas().remaining());
                self.gas.record_refund(create_outcome.gas().refunded());
            }
            return_revert!() => {
                push!(self, U256::ZERO);
                self.gas.erase_cost(create_outcome.gas().remaining());
            }
            InstructionResult::FatalExternalError => {
                panic!("Fatal external error in insert_eofcreate_outcome");
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
        self.instruction_result = InstructionResult::Continue;

        let out_offset = call_outcome.memory_start();
        let out_len = call_outcome.memory_length();
        let out_ins_result = *call_outcome.instruction_result();
        let out_gas = call_outcome.gas();
        self.return_data_buffer = call_outcome.result.output;

        let target_len = min(out_len, self.return_data_buffer.len());
        match out_ins_result {
            return_ok!() => {
                // return unspend gas.
                self.gas.erase_cost(out_gas.remaining());
                self.gas.record_refund(out_gas.refunded());
                shared_memory.set(out_offset, &self.return_data_buffer[..target_len]);
                push!(
                    self,
                    if self.is_eof {
                        U256::ZERO
                    } else {
                        U256::from(1)
                    }
                );
            }
            return_revert!() => {
                self.gas.erase_cost(out_gas.remaining());
                shared_memory.set(out_offset, &self.return_data_buffer[..target_len]);
                push!(
                    self,
                    if self.is_eof {
                        U256::from(1)
                    } else {
                        U256::ZERO
                    }
                );
            }
            InstructionResult::FatalExternalError => {
                panic!("Fatal external error in insert_call_outcome");
            }
            _ => {
                push!(
                    self,
                    if self.is_eof {
                        U256::from(2)
                    } else {
                        U256::ZERO
                    }
                );
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

    /// Returns a mutable reference to the interpreter's stack.
    #[inline]
    pub fn stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }

    /// Returns the current program counter.
    #[inline]
    pub fn program_counter(&self) -> usize {
        // SAFETY: `instruction_pointer` should be at an offset from the start of the bytecode.
        // In practice this is always true unless a caller modifies the `instruction_pointer` field manually.
        unsafe { self.instruction_pointer.offset_from(self.bytecode.as_ptr()) as usize }
    }

    /// Executes the instruction at the current instruction pointer.
    ///
    /// Internally it will increment instruction pointer by one.
    #[inline]
    pub(crate) fn step<FN, H: Host + ?Sized>(&mut self, instruction_table: &[FN; 256], host: &mut H)
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
    pub fn run<FN, H: Host + ?Sized>(
        &mut self,
        shared_memory: SharedMemory,
        instruction_table: &[FN; 256],
        host: &mut H,
    ) -> InterpreterAction
    where
        FN: Fn(&mut Interpreter, &mut H),
    {
        self.next_action = InterpreterAction::None;
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

    /// Resize the memory to the new size. Returns whether the gas was enough to resize the memory.
    #[inline]
    #[must_use]
    pub fn resize_memory(&mut self, new_size: usize) -> bool {
        resize_memory(&mut self.shared_memory, &mut self.gas, new_size)
    }
}

/// The result of an interpreter operation.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct InterpreterResult {
    /// The result of the instruction execution.
    pub result: InstructionResult,
    /// The output of the instruction execution.
    pub output: Bytes,
    /// The gas usage information.
    pub gas: Gas,
}

impl InterpreterResult {
    /// Returns a new `InterpreterResult` with the given values.
    pub fn new(result: InstructionResult, output: Bytes, gas: Gas) -> Self {
        Self {
            result,
            output,
            gas,
        }
    }

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

/// Resize the memory to the new size. Returns whether the gas was enough to resize the memory.
#[inline(never)]
#[cold]
#[must_use]
pub fn resize_memory(memory: &mut SharedMemory, gas: &mut Gas, new_size: usize) -> bool {
    let new_words = num_words(new_size as u64);
    let new_cost = gas::memory_gas(new_words);
    let current_cost = memory.current_expansion_cost();
    let cost = new_cost - current_cost;
    let success = gas.record_cost(cost);
    if success {
        memory.resize((new_words as usize) * 32);
    }
    success
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{table::InstructionTable, DummyHost};
    use wiring::DefaultEthereumWiring;

    #[test]
    fn object_safety() {
        let mut interp = Interpreter::new(Contract::default(), u64::MAX, false);
        interp.spec_id = SpecId::CANCUN;
        let mut host = crate::DummyHost::<DefaultEthereumWiring>::default();
        let table: &InstructionTable<DummyHost<DefaultEthereumWiring>> =
            &crate::table::make_instruction_table::<Interpreter, DummyHost<DefaultEthereumWiring>>(
            );
        let _ = interp.run(EMPTY_SHARED_MEMORY, table, &mut host);

        let host: &mut dyn Host<EvmWiringT = DefaultEthereumWiring> =
            &mut host as &mut dyn Host<EvmWiringT = DefaultEthereumWiring>;
        let table: &InstructionTable<dyn Host<EvmWiringT = DefaultEthereumWiring>> =
            &crate::table::make_instruction_table::<
                Interpreter,
                dyn Host<EvmWiringT = DefaultEthereumWiring>,
            >();
        let _ = interp.run(EMPTY_SHARED_MEMORY, table, host);
    }
}
