//! Core interpreter implementation and components.

/// Extended bytecode functionality.
pub mod ext_bytecode;
mod input;
mod loop_control;
mod return_data;
mod runtime_flags;
mod shared_memory;
mod stack;

use context_interface::cfg::GasParams;
// re-exports
pub use ext_bytecode::ExtBytecode;
pub use input::InputsImpl;
pub use return_data::ReturnDataImpl;
pub use runtime_flags::RuntimeFlags;
pub use shared_memory::{num_words, resize_memory, SharedMemory};
pub use stack::{Stack, STACK_LIMIT};

// imports
use crate::{
    host::DummyHost, instruction_context::InstructionContext, interpreter_types::*, Gas, Host,
    InstructionResult, InstructionTable, InterpreterAction,
};
use bytecode::Bytecode;
use primitives::{hardfork::SpecId, Bytes};

/// Main interpreter structure that contains all components defined in [`InterpreterTypes`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interpreter<WIRE: InterpreterTypes = EthInterpreter> {
    /// Bytecode being executed.
    pub bytecode: WIRE::Bytecode,
    /// Gas tracking for execution costs.
    pub gas: Gas,
    /// EVM stack for computation.
    pub stack: WIRE::Stack,
    /// Buffer for return data from calls.
    pub return_data: WIRE::ReturnData,
    /// EVM memory for data storage.
    pub memory: WIRE::Memory,
    /// Input data for current execution context.
    pub input: WIRE::Input,
    /// Runtime flags controlling execution behavior.
    pub runtime_flag: WIRE::RuntimeFlag,
    /// Extended functionality and customizations.
    pub extend: WIRE::Extend,
}

impl<EXT: Default> Interpreter<EthInterpreter<EXT>> {
    /// Create new interpreter
    pub fn new(
        memory: SharedMemory,
        bytecode: ExtBytecode,
        input: InputsImpl,
        is_static: bool,
        spec_id: SpecId,
        gas_limit: u64,
    ) -> Self {
        Self::new_inner(
            Stack::new(),
            memory,
            bytecode,
            input,
            is_static,
            spec_id,
            gas_limit,
        )
    }

    /// Create a new interpreter with default extended functionality.
    pub fn default_ext() -> Self {
        Self::do_default(Stack::new(), SharedMemory::new())
    }

    /// Create a new invalid interpreter.
    pub fn invalid() -> Self {
        Self::do_default(Stack::invalid(), SharedMemory::invalid())
    }

    fn do_default(stack: Stack, memory: SharedMemory) -> Self {
        Self::new_inner(
            stack,
            memory,
            ExtBytecode::default(),
            InputsImpl::default(),
            false,
            SpecId::default(),
            u64::MAX,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_inner(
        stack: Stack,
        memory: SharedMemory,
        bytecode: ExtBytecode,
        input: InputsImpl,
        is_static: bool,
        spec_id: SpecId,
        gas_limit: u64,
    ) -> Self {
        Self {
            bytecode,
            gas: Gas::new(gas_limit),
            stack,
            return_data: Default::default(),
            memory,
            input,
            runtime_flag: RuntimeFlags { is_static, spec_id },
            extend: Default::default(),
        }
    }

    /// Clears and reinitializes the interpreter with new parameters.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn clear(
        &mut self,
        memory: SharedMemory,
        bytecode: ExtBytecode,
        input: InputsImpl,
        is_static: bool,
        spec_id: SpecId,
        gas_limit: u64,
    ) {
        let Self {
            bytecode: bytecode_ref,
            gas,
            stack,
            return_data,
            memory: memory_ref,
            input: input_ref,
            runtime_flag,
            extend,
        } = self;
        *bytecode_ref = bytecode;
        *gas = Gas::new(gas_limit);
        if stack.data().capacity() == 0 {
            *stack = Stack::new();
        } else {
            stack.clear();
        }
        return_data.0.clear();
        *memory_ref = memory;
        *input_ref = input;
        *runtime_flag = RuntimeFlags { spec_id, is_static };
        *extend = EXT::default();
    }

    /// Sets the bytecode that is going to be executed
    pub fn with_bytecode(mut self, bytecode: Bytecode) -> Self {
        self.bytecode = ExtBytecode::new(bytecode);
        self
    }
}

impl Default for Interpreter<EthInterpreter> {
    fn default() -> Self {
        Self::default_ext()
    }
}

/// Default types for Ethereum interpreter.
#[derive(Debug)]
pub struct EthInterpreter<EXT = (), MG = SharedMemory> {
    _phantom: core::marker::PhantomData<fn() -> (EXT, MG)>,
}

impl<EXT> InterpreterTypes for EthInterpreter<EXT> {
    type Stack = Stack;
    type Memory = SharedMemory;
    type Bytecode = ExtBytecode;
    type ReturnData = ReturnDataImpl;
    type Input = InputsImpl;
    type RuntimeFlag = RuntimeFlags;
    type Extend = EXT;
    type Output = InterpreterAction;
}

impl<IW: InterpreterTypes> Interpreter<IW> {
    /// Performs EVM memory resize.
    #[inline]
    #[must_use]
    pub fn resize_memory(&mut self, gas_params: &GasParams, offset: usize, len: usize) -> bool {
        if let Err(result) = resize_memory(&mut self.gas, &mut self.memory, gas_params, offset, len)
        {
            self.halt(result);
            return false;
        }
        true
    }

    /// Takes the next action from the control and returns it.
    #[inline]
    pub fn take_next_action(&mut self) -> InterpreterAction {
        self.bytecode.reset_action();
        // Return next action if it is some.
        let action = core::mem::take(self.bytecode.action()).expect("Interpreter to set action");
        action
    }

    /// Halt the interpreter with the given result.
    ///
    /// This will set the action to [`InterpreterAction::Return`] and set the gas to the current gas.
    #[cold]
    #[inline(never)]
    pub fn halt(&mut self, result: InstructionResult) {
        self.bytecode
            .set_action(InterpreterAction::new_halt(result, self.gas));
    }

    /// Halt the interpreter with the given result.
    ///
    /// This will set the action to [`InterpreterAction::Return`] and set the gas to the current gas.
    #[cold]
    #[inline(never)]
    pub fn halt_fatal(&mut self) {
        self.bytecode.set_action(InterpreterAction::new_halt(
            InstructionResult::FatalExternalError,
            self.gas,
        ));
    }

    /// Halt the interpreter with an out-of-gas error.
    #[cold]
    #[inline(never)]
    pub fn halt_oog(&mut self) {
        self.gas.spend_all();
        self.halt(InstructionResult::OutOfGas);
    }

    /// Halt the interpreter with an out-of-gas error.
    #[cold]
    #[inline(never)]
    pub fn halt_memory_oog(&mut self) {
        self.halt(InstructionResult::MemoryOOG);
    }

    /// Halt the interpreter with an out-of-gas error.
    #[cold]
    #[inline(never)]
    pub fn halt_memory_limit_oog(&mut self) {
        self.halt(InstructionResult::MemoryLimitOOG);
    }

    /// Halt the interpreter with and overflow error.
    #[cold]
    #[inline(never)]
    pub fn halt_overflow(&mut self) {
        self.halt(InstructionResult::StackOverflow);
    }

    /// Halt the interpreter with and underflow error.
    #[cold]
    #[inline(never)]
    pub fn halt_underflow(&mut self) {
        self.halt(InstructionResult::StackUnderflow);
    }

    /// Halt the interpreter with and not activated error.
    #[cold]
    #[inline(never)]
    pub fn halt_not_activated(&mut self) {
        self.halt(InstructionResult::NotActivated);
    }

    /// Return with the given output.
    ///
    /// This will set the action to [`InterpreterAction::Return`] and set the gas to the current gas.
    pub fn return_with_output(&mut self, output: Bytes) {
        self.bytecode.set_action(InterpreterAction::new_return(
            InstructionResult::Return,
            output,
            self.gas,
        ));
    }

    /// Executes the instruction at the current instruction pointer.
    ///
    /// Internally it will increment instruction pointer by one.
    #[inline]
    pub fn step<H: Host + ?Sized>(
        &mut self,
        instruction_table: &InstructionTable<IW, H>,
        host: &mut H,
    ) {
        // Get current opcode.
        let opcode = self.bytecode.opcode();

        // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.bytecode.relative_jump(1);

        let instruction = unsafe { instruction_table.get_unchecked(opcode as usize) };

        if self.gas.record_cost_unsafe(instruction.static_gas()) {
            return self.halt_oog();
        }
        let context = InstructionContext {
            interpreter: self,
            host,
        };
        instruction.execute(context);
    }

    /// Executes the instruction at the current instruction pointer.
    ///
    /// Internally it will increment instruction pointer by one.
    ///
    /// This uses dummy Host.
    #[inline]
    pub fn step_dummy(&mut self, instruction_table: &InstructionTable<IW, DummyHost>) {
        self.step(instruction_table, &mut DummyHost::default());
    }

    /// Executes the interpreter until it returns or stops.
    ///
    /// Uses direct dispatch for the most common opcodes to enable inlining
    /// and avoid indirect function pointer call overhead. Less common opcodes
    /// fall back to the instruction table.
    #[inline]
    pub fn run_plain<H: Host + ?Sized>(
        &mut self,
        instruction_table: &InstructionTable<IW, H>,
        host: &mut H,
    ) -> InterpreterAction {
        use bytecode::opcode::*;
        use crate::instructions::{arithmetic, bitwise, control, memory, stack, system};

        while self.bytecode.is_not_end() {
            let opcode = self.bytecode.opcode();
            self.bytecode.relative_jump(1);

            let instruction = unsafe { instruction_table.get_unchecked(opcode as usize) };

            if self.gas.record_cost_unsafe(instruction.static_gas()) {
                self.halt_oog();
                continue;
            }

            match opcode {
                STOP => control::stop(InstructionContext { interpreter: self, host }),
                JUMPDEST => {}

                PUSH0 => stack::push0(InstructionContext { interpreter: self, host }),
                PUSH1 => stack::push::<1, _, _>(InstructionContext { interpreter: self, host }),
                PUSH2 => stack::push::<2, _, _>(InstructionContext { interpreter: self, host }),
                PUSH3 => stack::push::<3, _, _>(InstructionContext { interpreter: self, host }),
                PUSH4 => stack::push::<4, _, _>(InstructionContext { interpreter: self, host }),
                PUSH5 => stack::push::<5, _, _>(InstructionContext { interpreter: self, host }),
                PUSH6 => stack::push::<6, _, _>(InstructionContext { interpreter: self, host }),
                PUSH7 => stack::push::<7, _, _>(InstructionContext { interpreter: self, host }),
                PUSH8 => stack::push::<8, _, _>(InstructionContext { interpreter: self, host }),
                PUSH9 => stack::push::<9, _, _>(InstructionContext { interpreter: self, host }),
                PUSH10 => stack::push::<10, _, _>(InstructionContext { interpreter: self, host }),
                PUSH11 => stack::push::<11, _, _>(InstructionContext { interpreter: self, host }),
                PUSH12 => stack::push::<12, _, _>(InstructionContext { interpreter: self, host }),
                PUSH13 => stack::push::<13, _, _>(InstructionContext { interpreter: self, host }),
                PUSH14 => stack::push::<14, _, _>(InstructionContext { interpreter: self, host }),
                PUSH15 => stack::push::<15, _, _>(InstructionContext { interpreter: self, host }),
                PUSH16 => stack::push::<16, _, _>(InstructionContext { interpreter: self, host }),
                PUSH17 => stack::push::<17, _, _>(InstructionContext { interpreter: self, host }),
                PUSH18 => stack::push::<18, _, _>(InstructionContext { interpreter: self, host }),
                PUSH19 => stack::push::<19, _, _>(InstructionContext { interpreter: self, host }),
                PUSH20 => stack::push::<20, _, _>(InstructionContext { interpreter: self, host }),
                PUSH21 => stack::push::<21, _, _>(InstructionContext { interpreter: self, host }),
                PUSH22 => stack::push::<22, _, _>(InstructionContext { interpreter: self, host }),
                PUSH23 => stack::push::<23, _, _>(InstructionContext { interpreter: self, host }),
                PUSH24 => stack::push::<24, _, _>(InstructionContext { interpreter: self, host }),
                PUSH25 => stack::push::<25, _, _>(InstructionContext { interpreter: self, host }),
                PUSH26 => stack::push::<26, _, _>(InstructionContext { interpreter: self, host }),
                PUSH27 => stack::push::<27, _, _>(InstructionContext { interpreter: self, host }),
                PUSH28 => stack::push::<28, _, _>(InstructionContext { interpreter: self, host }),
                PUSH29 => stack::push::<29, _, _>(InstructionContext { interpreter: self, host }),
                PUSH30 => stack::push::<30, _, _>(InstructionContext { interpreter: self, host }),
                PUSH31 => stack::push::<31, _, _>(InstructionContext { interpreter: self, host }),
                PUSH32 => stack::push::<32, _, _>(InstructionContext { interpreter: self, host }),

                POP => stack::pop(InstructionContext { interpreter: self, host }),

                DUP1 => stack::dup::<1, _, _>(InstructionContext { interpreter: self, host }),
                DUP2 => stack::dup::<2, _, _>(InstructionContext { interpreter: self, host }),
                DUP3 => stack::dup::<3, _, _>(InstructionContext { interpreter: self, host }),
                DUP4 => stack::dup::<4, _, _>(InstructionContext { interpreter: self, host }),
                DUP5 => stack::dup::<5, _, _>(InstructionContext { interpreter: self, host }),
                DUP6 => stack::dup::<6, _, _>(InstructionContext { interpreter: self, host }),
                DUP7 => stack::dup::<7, _, _>(InstructionContext { interpreter: self, host }),
                DUP8 => stack::dup::<8, _, _>(InstructionContext { interpreter: self, host }),
                DUP9 => stack::dup::<9, _, _>(InstructionContext { interpreter: self, host }),
                DUP10 => stack::dup::<10, _, _>(InstructionContext { interpreter: self, host }),
                DUP11 => stack::dup::<11, _, _>(InstructionContext { interpreter: self, host }),
                DUP12 => stack::dup::<12, _, _>(InstructionContext { interpreter: self, host }),
                DUP13 => stack::dup::<13, _, _>(InstructionContext { interpreter: self, host }),
                DUP14 => stack::dup::<14, _, _>(InstructionContext { interpreter: self, host }),
                DUP15 => stack::dup::<15, _, _>(InstructionContext { interpreter: self, host }),
                DUP16 => stack::dup::<16, _, _>(InstructionContext { interpreter: self, host }),

                SWAP1 => stack::swap::<1, _, _>(InstructionContext { interpreter: self, host }),
                SWAP2 => stack::swap::<2, _, _>(InstructionContext { interpreter: self, host }),
                SWAP3 => stack::swap::<3, _, _>(InstructionContext { interpreter: self, host }),
                SWAP4 => stack::swap::<4, _, _>(InstructionContext { interpreter: self, host }),
                SWAP5 => stack::swap::<5, _, _>(InstructionContext { interpreter: self, host }),
                SWAP6 => stack::swap::<6, _, _>(InstructionContext { interpreter: self, host }),
                SWAP7 => stack::swap::<7, _, _>(InstructionContext { interpreter: self, host }),
                SWAP8 => stack::swap::<8, _, _>(InstructionContext { interpreter: self, host }),
                SWAP9 => stack::swap::<9, _, _>(InstructionContext { interpreter: self, host }),
                SWAP10 => stack::swap::<10, _, _>(InstructionContext { interpreter: self, host }),
                SWAP11 => stack::swap::<11, _, _>(InstructionContext { interpreter: self, host }),
                SWAP12 => stack::swap::<12, _, _>(InstructionContext { interpreter: self, host }),
                SWAP13 => stack::swap::<13, _, _>(InstructionContext { interpreter: self, host }),
                SWAP14 => stack::swap::<14, _, _>(InstructionContext { interpreter: self, host }),
                SWAP15 => stack::swap::<15, _, _>(InstructionContext { interpreter: self, host }),
                SWAP16 => stack::swap::<16, _, _>(InstructionContext { interpreter: self, host }),

                ADD => arithmetic::add(InstructionContext { interpreter: self, host }),
                MUL => arithmetic::mul(InstructionContext { interpreter: self, host }),
                SUB => arithmetic::sub(InstructionContext { interpreter: self, host }),
                DIV => arithmetic::div(InstructionContext { interpreter: self, host }),
                MOD => arithmetic::rem(InstructionContext { interpreter: self, host }),
                ADDMOD => arithmetic::addmod(InstructionContext { interpreter: self, host }),
                MULMOD => arithmetic::mulmod(InstructionContext { interpreter: self, host }),

                LT => bitwise::lt(InstructionContext { interpreter: self, host }),
                GT => bitwise::gt(InstructionContext { interpreter: self, host }),
                EQ => bitwise::eq(InstructionContext { interpreter: self, host }),
                ISZERO => bitwise::iszero(InstructionContext { interpreter: self, host }),
                AND => bitwise::bitand(InstructionContext { interpreter: self, host }),
                OR => bitwise::bitor(InstructionContext { interpreter: self, host }),
                XOR => bitwise::bitxor(InstructionContext { interpreter: self, host }),
                NOT => bitwise::not(InstructionContext { interpreter: self, host }),
                SHL => bitwise::shl(InstructionContext { interpreter: self, host }),
                SHR => bitwise::shr(InstructionContext { interpreter: self, host }),

                MLOAD => memory::mload(InstructionContext { interpreter: self, host }),
                MSTORE => memory::mstore(InstructionContext { interpreter: self, host }),
                MSTORE8 => memory::mstore8(InstructionContext { interpreter: self, host }),

                JUMP => control::jump(InstructionContext { interpreter: self, host }),
                JUMPI => control::jumpi(InstructionContext { interpreter: self, host }),

                CALLDATALOAD => system::calldataload(InstructionContext { interpreter: self, host }),
                CALLDATASIZE => system::calldatasize(InstructionContext { interpreter: self, host }),
                CALLER => system::caller(InstructionContext { interpreter: self, host }),
                CALLVALUE => system::callvalue(InstructionContext { interpreter: self, host }),
                ADDRESS => system::address(InstructionContext { interpreter: self, host }),
                GAS => system::gas(InstructionContext { interpreter: self, host }),

                _ => instruction.execute(InstructionContext { interpreter: self, host }),
            }
        }
        self.take_next_action()
    }
}

/* used for cargo asm
pub fn asm_step(
    interpreter: &mut Interpreter<EthInterpreter>,
    instruction_table: &InstructionTable<EthInterpreter, DummyHost>,
    host: &mut DummyHost,
) {
    interpreter.step(instruction_table, host);
}

pub fn asm_run(
    interpreter: &mut Interpreter<EthInterpreter>,
    instruction_table: &InstructionTable<EthInterpreter, DummyHost>,
    host: &mut DummyHost,
) {
    interpreter.run_plain(instruction_table, host);
}
*/

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

    /// Returns a new `InterpreterResult` for an out-of-gas error with the given gas limit.
    pub fn new_oog(gas_limit: u64) -> Self {
        Self {
            result: InstructionResult::OutOfGas,
            output: Bytes::default(),
            gas: Gas::new_spent(gas_limit),
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

// Special implementation for types where Output can be created from InterpreterAction
impl<IW: InterpreterTypes> Interpreter<IW>
where
    IW::Output: From<InterpreterAction>,
{
    /// Takes the next action from the control and returns it as the specific Output type.
    #[inline]
    pub fn take_next_action_as_output(&mut self) -> IW::Output {
        From::from(self.take_next_action())
    }

    /// Executes the interpreter until it returns or stops, returning the specific Output type.
    #[inline]
    pub fn run_plain_as_output<H: Host + ?Sized>(
        &mut self,
        instruction_table: &InstructionTable<IW, H>,
        host: &mut H,
    ) -> IW::Output {
        From::from(self.run_plain(instruction_table, host))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "serde")]
    fn test_interpreter_serde() {
        use super::*;
        use bytecode::Bytecode;
        use primitives::Bytes;

        let bytecode = Bytecode::new_raw(Bytes::from(&[0x60, 0x00, 0x60, 0x00, 0x01][..]));
        let interpreter = Interpreter::<EthInterpreter>::new(
            SharedMemory::new(),
            ExtBytecode::new(bytecode),
            InputsImpl::default(),
            false,
            SpecId::default(),
            u64::MAX,
        );

        let serialized = serde_json::to_string_pretty(&interpreter).unwrap();
        let deserialized: Interpreter<EthInterpreter> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            interpreter.bytecode.pc(),
            deserialized.bytecode.pc(),
            "Program counter should be preserved"
        );
    }
}

#[test]
fn test_mstore_big_offset_memory_oog() {
    use super::*;
    use crate::{host::DummyHost, instructions::instruction_table};
    use bytecode::Bytecode;
    use primitives::Bytes;

    let code = Bytes::from(
        &[
            0x60, 0x00, // PUSH1 0x00
            0x61, 0x27, 0x10, // PUSH2 0x2710  (10,000)
            0x52, // MSTORE
            0x00, // STOP
        ][..],
    );
    let bytecode = Bytecode::new_raw(code);

    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(bytecode),
        InputsImpl::default(),
        false,
        SpecId::default(),
        1000,
    );

    let table = instruction_table::<EthInterpreter, DummyHost>();
    let mut host = DummyHost::default();
    let action = interpreter.run_plain(&table, &mut host);

    assert!(action.is_return());
    assert_eq!(
        action.instruction_result(),
        Some(InstructionResult::MemoryOOG)
    );
}

#[test]
#[cfg(feature = "memory_limit")]
fn test_mstore_big_offset_memory_limit_oog() {
    use super::*;
    use crate::{host::DummyHost, instructions::instruction_table};
    use bytecode::Bytecode;
    use primitives::Bytes;

    let code = Bytes::from(
        &[
            0x60, 0x00, // PUSH1 0x00
            0x61, 0x27, 0x10, // PUSH2 0x2710  (10,000)
            0x52, // MSTORE
            0x00, // STOP
        ][..],
    );
    let bytecode = Bytecode::new_raw(code);

    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new_with_memory_limit(1000),
        ExtBytecode::new(bytecode),
        InputsImpl::default(),
        false,
        SpecId::default(),
        100000,
    );

    let table = instruction_table::<EthInterpreter, DummyHost>();
    let mut host = DummyHost::default();
    let action = interpreter.run_plain(&table, &mut host);

    assert!(action.is_return());
    assert_eq!(
        action.instruction_result(),
        Some(InstructionResult::MemoryLimitOOG)
    );
}
