//! Core interpreter implementation and components.

/// Extended bytecode functionality.
pub mod ext_bytecode;
mod input;
mod loop_control;
mod return_data;
mod runtime_flags;
mod shared_memory;
mod stack;

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

    /// Sets the specid for the interpreter.
    pub fn set_spec_id(&mut self, spec_id: SpecId) {
        self.runtime_flag.spec_id = spec_id;
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
    pub fn resize_memory(&mut self, offset: usize, len: usize) -> bool {
        resize_memory(&mut self.gas, &mut self.memory, offset, len)
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
        self.step(instruction_table, &mut DummyHost);
    }

    /// Executes the interpreter until it returns or stops.
    #[inline]
    pub fn run_plain<H: Host + ?Sized>(
        &mut self,
        instruction_table: &InstructionTable<IW, H>,
        host: &mut H,
    ) -> InterpreterAction {
        while self.bytecode.is_not_end() {
            self.step(instruction_table, host);
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
