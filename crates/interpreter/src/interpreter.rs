pub mod ext_bytecode;
mod input;
mod loop_control;
mod return_data;
mod runtime_flags;
mod shared_memory;
mod stack;
mod subroutine_stack;

use crate::{
    interpreter_types::*, Gas, Host, Instruction, InstructionResult, InstructionTable,
    InterpreterAction,
};
use core::cell::RefCell;
pub use ext_bytecode::ExtBytecode;
pub use input::InputsImpl;
use loop_control::LoopControl as LoopControlImpl;
use primitives::{hardfork::SpecId, Bytes};
use return_data::ReturnDataImpl;
pub use runtime_flags::RuntimeFlags;
pub use shared_memory::{num_words, MemoryGetter, SharedMemory, EMPTY_SHARED_MEMORY};
pub use stack::{Stack, STACK_LIMIT};
use std::rc::Rc;
use subroutine_stack::SubRoutineImpl;

/// Main interpreter structure that contains all components defines in [`InterpreterTypes`].s
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Interpreter<WIRE: InterpreterTypes = EthInterpreter> {
    pub bytecode: WIRE::Bytecode,
    pub stack: WIRE::Stack,
    pub return_data: WIRE::ReturnData,
    pub memory: WIRE::Memory,
    pub input: WIRE::Input,
    pub sub_routine: WIRE::SubRoutineStack,
    pub control: WIRE::Control,
    pub runtime_flag: WIRE::RuntimeFlag,
    pub extend: WIRE::Extend,
}

impl<EXT: Default, MG: MemoryGetter> Interpreter<EthInterpreter<EXT, MG>> {
    /// Create new interpreter
    pub fn new(
        memory: Rc<RefCell<MG>>,
        bytecode: ExtBytecode,
        inputs: InputsImpl,
        is_static: bool,
        is_eof_init: bool,
        spec_id: SpecId,
        gas_limit: u64,
    ) -> Self {
        let runtime_flag = RuntimeFlags {
            spec_id,
            is_static,
            is_eof: bytecode.is_eof(),
            is_eof_init,
        };

        Self {
            bytecode,
            stack: Stack::new(),
            return_data: ReturnDataImpl::default(),
            memory,
            input: inputs,
            sub_routine: SubRoutineImpl::default(),
            control: LoopControlImpl::new(gas_limit),
            runtime_flag,
            extend: EXT::default(),
        }
    }
}

/// Default types for Ethereum interpreter.
pub struct EthInterpreter<EXT = (), MG = SharedMemory> {
    _phantom: core::marker::PhantomData<fn() -> (EXT, MG)>,
}

impl<EXT, MG: MemoryGetter> InterpreterTypes for EthInterpreter<EXT, MG> {
    type Stack = Stack;
    type Memory = Rc<RefCell<MG>>;
    type Bytecode = ExtBytecode;
    type ReturnData = ReturnDataImpl;
    type Input = InputsImpl;
    type SubRoutineStack = SubRoutineImpl;
    type Control = LoopControlImpl;
    type RuntimeFlag = RuntimeFlags;
    type Extend = EXT;
    type Output = InterpreterAction;
}

// TODO InterpreterAction should be replaces with InterpreterTypes::Output.
impl<IW: InterpreterTypes> Interpreter<IW> {
    /// Executes the instruction at the current instruction pointer.
    ///
    /// Internally it will increment instruction pointer by one.
    #[inline]
    pub(crate) fn step<H: Host + ?Sized>(
        &mut self,
        instruction_table: &[Instruction<IW, H>; 256],
        host: &mut H,
    ) {
        // Get current opcode.
        let opcode = self.bytecode.opcode();

        // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.bytecode.relative_jump(1);

        // Execute instruction.
        instruction_table[opcode as usize](self, host)
    }

    /// Resets the control to the initial state. so that we can run the interpreter again.
    #[inline]
    pub fn reset_control(&mut self) {
        self.control
            .set_next_action(InterpreterAction::None, InstructionResult::Continue);
    }

    /// Takes the next action from the control and returns it.
    #[inline]
    pub fn take_next_action(&mut self) -> InterpreterAction {
        // Return next action if it is some.
        let action = self.control.take_next_action();
        if action.is_some() {
            return action;
        }
        // If not, return action without output as it is a halt.
        InterpreterAction::Return {
            result: InterpreterResult {
                result: self.control.instruction_result(),
                // Return empty bytecode
                output: Bytes::new(),
                gas: *self.control.gas(),
            },
        }
    }

    /// Executes the interpreter until it returns or stops.
    #[inline]
    pub fn run_plain<H: Host + ?Sized>(
        &mut self,
        instruction_table: &InstructionTable<IW, H>,
        host: &mut H,
    ) -> InterpreterAction {
        self.reset_control();

        // Main loop
        while self.control.instruction_result().is_continue() {
            self.step(instruction_table, host);
        }

        self.take_next_action()
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

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "serde")]
    fn test_interpreter_serde() {
        use super::*;
        use bytecode::Bytecode;
        use primitives::{Address, Bytes, U256};

        let bytecode = Bytecode::new_raw(Bytes::from(&[0x60, 0x00, 0x60, 0x00, 0x01][..]));
        let interpreter = Interpreter::<EthInterpreter>::new(
            Rc::new(RefCell::new(SharedMemory::new())),
            ExtBytecode::new(bytecode),
            InputsImpl {
                target_address: Address::ZERO,
                caller_address: Address::ZERO,
                input: Bytes::default(),
                call_value: U256::ZERO,
            },
            false,
            false,
            SpecId::LATEST,
            u64::MAX,
        );

        let serialized = bincode::serialize(&interpreter).unwrap();

        let deserialized: Interpreter<EthInterpreter> = bincode::deserialize(&serialized).unwrap();

        assert_eq!(
            interpreter.bytecode.pc(),
            deserialized.bytecode.pc(),
            "Program counter should be preserved"
        );
    }
}
