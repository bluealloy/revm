use crate::{
    interpreter_types::InterpreterTypes,
    InstructionResult,
};

use crate::InstructionContext;

/// Data load instruction - loads 32 bytes from data section at given offset
/// Since EOF support has been removed, this will always halt with EOFOpcodeDisabledInLegacy
pub fn data_load<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    context.interpreter.halt(InstructionResult::EOFOpcodeDisabledInLegacy);
}

/// Data load immediate instruction - loads 32 bytes from data section at immediate offset
/// Since EOF support has been removed, this will always halt with EOFOpcodeDisabledInLegacy
pub fn data_loadn<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    context.interpreter.halt(InstructionResult::EOFOpcodeDisabledInLegacy);
}

/// Data size instruction - pushes size of data section to stack
/// Since EOF support has been removed, this will always halt with EOFOpcodeDisabledInLegacy
pub fn data_size<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    context.interpreter.halt(InstructionResult::EOFOpcodeDisabledInLegacy);
}

/// Data copy instruction - copies data from data section to memory
/// Since EOF support has been removed, this will always halt with EOFOpcodeDisabledInLegacy
pub fn data_copy<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    context.interpreter.halt(InstructionResult::EOFOpcodeDisabledInLegacy);
}