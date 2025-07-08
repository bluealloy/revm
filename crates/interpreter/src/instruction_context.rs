use crate::{interpreter_types::Jumps, InstructionResult, Interpreter, InterpreterTypes};

use super::Instruction;

/// Context passed to instruction implementations containing the host and interpreter.
/// This struct provides access to both the host interface for external state operations
/// and the interpreter state for stack, memory, and gas operations.
pub struct InstructionContext<'a, H: ?Sized, ITy: InterpreterTypes> {
    /// Reference to the host interface for accessing external blockchain state.
    pub host: &'a mut H,
    /// Reference to the interpreter containing execution state (stack, memory, gas, etc).
    pub interpreter: &'a mut Interpreter<ITy>,
}

impl<H: ?Sized, ITy: InterpreterTypes> std::fmt::Debug for InstructionContext<'_, H, ITy> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstructionContext")
            .field("host", &"<host>")
            .field("interpreter", &"<interpreter>")
            .finish()
    }
}

impl<H: ?Sized, ITy: InterpreterTypes> InstructionContext<'_, H, ITy> {
    /// Executes the instruction at the current instruction pointer.
    ///
    /// Internally it will increment instruction pointer by one.
    #[inline]
    pub(crate) fn step(self, instruction_table: &[Instruction<ITy, H>; 256]) {
        // Get current opcode.
        let opcode = self.interpreter.bytecode.opcode();

        // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.interpreter.bytecode.relative_jump(1);

        // Execute instruction.
        instruction_table[opcode as usize](self)
    }

    /// Halts the execution of the contract with a fatal error.
    #[inline]
    pub fn fatal_halt(&mut self) {
        self.interpreter.halt(InstructionResult::FatalExternalError);
    }
}
