use crate::{interpreter_types::Jumps, Host, Interpreter, InterpreterTypes};

use super::Instruction;

pub struct InstructionContext<'a, H: ?Sized, ITy: InterpreterTypes> {
    pub host: &'a mut H,
    pub interpreter: &'a mut Interpreter<ITy>,
}

impl<H: Host + ?Sized, ITy: InterpreterTypes> InstructionContext<'_, H, ITy> {
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
}
