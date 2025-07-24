use crate::{
    instructions::InstructionReturn, interpreter_types::Jumps, Instruction, InstructionTable,
    Interpreter, InterpreterTypes,
};

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

    /// Executes the instruction in this context.
    #[inline]
    pub fn call(self, f: Instruction<ITy, H>) -> InstructionReturn {
        f(self.interpreter, self.host, self.ip)
    }

    /// Executes the instruction at the current instruction pointer.
    ///
    /// Internally it will increment instruction pointer by one.
    #[inline]
    pub(crate) fn step(self, instruction_table: &InstructionTable<ITy, H>) -> InstructionReturn {
        // Get current opcode.
        let opcode = self.interpreter.bytecode.opcode();

        // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.interpreter.bytecode.relative_jump(1);

        // Execute instruction.
        self.call(instruction_table[opcode as usize])
    }
}
