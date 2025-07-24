use crate::{
    instructions::InstructionReturn, interpreter_types::Jumps, Instruction, InstructionResult,
    InstructionTable, Interpreter, InterpreterAction, InterpreterTypes,
};
use primitives::Bytes;

/// Context passed to instruction implementations containing the host and interpreter.
/// This struct provides access to both the host interface for external state operations
/// and the interpreter state for stack, memory, and gas operations.
pub struct InstructionContext<'a, H: ?Sized, ITy: InterpreterTypes> {
    /// Reference to the interpreter containing execution state (stack, memory, gas, etc).
    pub interpreter: &'a mut Interpreter<ITy>,
    /// Reference to the host interface for accessing external blockchain state.
    pub host: &'a mut H,

    pub(crate) ip: *const u8,
}

impl<H: ?Sized, ITy: InterpreterTypes> std::fmt::Debug for InstructionContext<'_, H, ITy> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstructionContext")
            .field("host", &"<host>")
            .field("interpreter", &"<interpreter>")
            .finish()
    }
}

impl<'a, H: ?Sized, ITy: InterpreterTypes> InstructionContext<'a, H, ITy> {
    /// Create a new instruction context.
    #[inline]
    pub fn new(interpreter: &'a mut Interpreter<ITy>, host: &'a mut H) -> Self {
        Self {
            ip: interpreter.bytecode.ip(),
            interpreter,
            host,
        }
    }

    /// Reborrows the context.
    #[inline]
    pub fn reborrow<'b>(&'b mut self) -> InstructionContext<'b, H, ITy> {
        InstructionContext {
            interpreter: self.interpreter,
            host: self.host,
            ip: self.ip,
        }
    }

    /// Performs EVM memory resize.
    #[inline]
    #[must_use]
    pub fn resize_memory(&mut self, offset: usize, len: usize) -> bool {
        self.interpreter.resize_memory(offset, len)
    }

    /// Takes the next action from the control and returns it.
    #[inline]
    pub fn take_next_action(&mut self) -> InterpreterAction {
        self.interpreter.take_next_action()
    }

    /// Halt the interpreter with the given result.
    ///
    /// This will set the action to [`InterpreterAction::Return`] and set the gas to the current gas.
    #[inline]
    pub fn halt(&mut self, result: InstructionResult) {
        self.flush();
        self.interpreter.halt(result);
    }

    /// Return with the given output.
    ///
    /// This will set the action to [`InterpreterAction::Return`] and set the gas to the current gas.
    #[inline]
    pub fn return_with_output(&mut self, output: Bytes) {
        self.flush();
        self.interpreter.return_with_output(output);
    }

    /// Returns the current opcode.
    #[inline]
    pub fn opcode(&self) -> u8 {
        unsafe { *self.ip }
    }

    /// Relative jump.
    #[inline]
    pub fn relative_jump(&mut self, offset: isize) {
        self.ip = unsafe { self.ip.add(offset as usize) };
    }

    /// Absolute jump.
    #[inline]
    pub fn absolute_jump(&mut self, target: usize) {
        self.ip = unsafe { self.interpreter.bytecode.base().add(target) };
    }

    /// Flush the instruction pointer.
    #[inline]
    pub fn flush(&mut self) {
        self.interpreter.bytecode.set_ip(self.ip);
    }

    /// Executes the instruction in this context.
    #[inline]
    pub fn call(&mut self, f: Instruction<ITy, H>) -> InstructionReturn {
        f(self.interpreter, self.host, self.ip)
    }

    #[inline]
    pub(crate) fn pre_step(&mut self) -> u8 {
        let opcode = self.opcode();
        // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.relative_jump(1);
        opcode
    }

    /// Executes the instruction at the current instruction pointer.
    ///
    /// Internally it will increment instruction pointer by one.
    #[inline]
    pub(crate) fn step(
        &mut self,
        instruction_table: &InstructionTable<ITy, H>,
    ) -> InstructionReturn {
        let opcode = self.pre_step();
        self.flush();
        self.call(instruction_table[opcode as usize])
    }
}
