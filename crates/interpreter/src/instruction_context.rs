use bytecode::opcode::OPCODE_INFO;

use crate::{
    gas,
    interpreter_types::{Jumps, StackTr},
    InstructionResult, Interpreter, InterpreterTypes, STACK_LIMIT,
};

use super::Instruction;

/// Context passed to instruction implementations containing the host and interpreter.
/// This struct provides access to both the host interface for external state operations
/// and the interpreter state for stack, memory, and gas operations.
pub struct InstructionContext<'a, H: ?Sized, ITy: InterpreterTypes> {
    /// Reference to the interpreter containing execution state (stack, memory, gas, etc).
    pub interpreter: &'a mut Interpreter<ITy>,
    /// Reference to the host interface for accessing external blockchain state.
    pub host: &'a mut H,
}

impl<H: ?Sized, ITy: InterpreterTypes> std::fmt::Debug for InstructionContext<'_, H, ITy> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstructionContext")
            .field("host", &"<host>")
            .field("interpreter", &"<interpreter>")
            .finish()
    }
}

static GAS_COST: [u32; 256] = {
    let mut gas_cost = [0; 256];
    let mut i: u32 = 0;
    while i < 256 {
        gas_cost[i as usize] = i % 10;
        i += 1;
    }
    gas_cost
};

impl<H: ?Sized, ITy: InterpreterTypes> InstructionContext<'_, H, ITy> {
    /// Executes the instruction at the current instruction pointer.
    ///
    /// Internally it will increment instruction pointer by one.
    #[inline]
    pub(crate) fn step(self, instruction_table: &[Instruction<ITy, H>; 256]) {
        // Get current opcode.
        let opcode = self.interpreter.bytecode.opcode();
        let opcode_info = unsafe { OPCODE_INFO[opcode as usize].unwrap_unchecked() };

        // Check if stack has enough inputs for this instruction
        let stack_len = self.interpreter.stack.len();
        let underflow = stack_len < opcode_info.inputs() as usize;
        let overflow = (stack_len as isize + opcode_info.io_diff() as isize) as usize > STACK_LIMIT;
        let oog = !self
            .interpreter
            .gas
            .record_cost(opcode_info.static_gas() as u64);

        // Check if stack will overflow after this instruction
        if underflow || overflow || oog {
            self.interpreter.halt(InstructionResult::StackUnderflow);
            return;
        }

        // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.interpreter.bytecode.relative_jump(1);

        // Execute instruction.
        instruction_table[opcode as usize](self)
    }
}
