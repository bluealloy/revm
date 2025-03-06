#![allow(clippy::wrong_self_convention)]

use crate::{
    instructions::{control, instruction},
    interpreter::Interpreter,
    interpreter_types::InterpreterTypes,
    Host,
};
use std::boxed::Box;

/// EVM opcode function signature.
pub type Instruction<W, H> = for<'a> fn(&'a mut Interpreter<W>, &'a mut H);

/// Instruction table is list of instruction function pointers mapped to 256 EVM opcodes.
pub type InstructionTable<W, H> = [Instruction<W, H>; 256];

/// A table of boxed instructions.
pub type CustomInstructionTable<IT> = [IT; 256];

pub trait CustomInstruction {
    type Wire: InterpreterTypes;
    type Host: ?Sized;

    fn exec(&self, interpreter: &mut Interpreter<Self::Wire>, host: &mut Self::Host);

    fn from_base(instruction: Instruction<Self::Wire, Self::Host>) -> Self;
}

/// Either a plain, static instruction table, or a boxed, dynamic instruction table.
///
/// Note that `Plain` variant is about 10-20% faster in Interpreter execution.
pub enum InstructionTables<
    W: InterpreterTypes,
    H: ?Sized,
    CI: CustomInstruction<Host = H, Wire = W>,
> {
    Plain(Box<InstructionTable<W, H>>),
    Custom(Box<CustomInstructionTable<CI>>),
}

impl<WIRE, H, CI> InstructionTables<WIRE, H, CI>
where
    WIRE: InterpreterTypes,
    H: Host + ?Sized,
    CI: CustomInstruction<Host = H, Wire = WIRE>,
{
    /// Inserts the instruction into the table with the specified index.
    #[inline]
    pub fn insert(&mut self, opcode: u8, instruction: Instruction<WIRE, H>) {
        match self {
            Self::Plain(table) => table[opcode as usize] = instruction,
            Self::Custom(table) => table[opcode as usize] = CI::from_base(instruction),
        }
    }

    /// Converts the current instruction table to a boxed variant if it is not already, and returns
    /// a mutable reference to the boxed table.
    #[inline]
    pub fn to_custom(&mut self) -> &mut CustomInstructionTable<CI> {
        self.to_custom_with(|i| CI::from_base(i))
    }

    /// Converts the current instruction table to a boxed variant if it is not already with `f`,
    /// and returns a mutable reference to the boxed table.
    #[inline]
    pub fn to_custom_with<F>(&mut self, f: F) -> &mut CustomInstructionTable<CI>
    where
        F: FnMut(Instruction<WIRE, H>) -> CI,
    {
        match self {
            Self::Plain(_) => self.to_custom_with_slow(f),
            Self::Custom(boxed) => boxed,
        }
    }

    #[cold]
    fn to_custom_with_slow<F>(&mut self, f: F) -> &mut CustomInstructionTable<CI>
    where
        F: FnMut(Instruction<WIRE, H>) -> CI,
    {
        let Self::Plain(table) = self else {
            unreachable!()
        };
        *self = Self::Custom(Box::new(make_custom_instruction_table(table, f)));
        let Self::Custom(boxed) = self else {
            unreachable!()
        };
        boxed
    }

    /// Returns a mutable reference to the boxed instruction at the specified index.
    #[inline]
    pub fn get_custom(&mut self, opcode: u8) -> &mut CI {
        &mut self.to_custom()[opcode as usize]
    }

    /// Inserts a boxed instruction into the table at the specified index.
    #[inline]
    pub fn insert_custom(&mut self, opcode: u8, instruction: CI) {
        *self.get_custom(opcode) = instruction;
    }

    /// Replaces a boxed instruction into the table at the specified index, returning the previous
    /// instruction.
    #[inline]
    pub fn replace_boxed(&mut self, opcode: u8, instruction: CI) -> CI {
        core::mem::replace(self.get_custom(opcode), instruction)
    }
}

/// Make instruction table.
#[inline]
pub const fn make_instruction_table<WIRE: InterpreterTypes, H: Host + ?Sized>(
) -> InstructionTable<WIRE, H> {
    let mut table: InstructionTable<WIRE, H> = [control::unknown; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = instruction::<WIRE, H>(i as u8);
        i += 1;
    }
    table
}

/// Make boxed instruction table that calls `f` closure for every instruction.
#[inline]
pub fn make_custom_instruction_table<W, H, FN, CI: CustomInstruction<Wire = W, Host = H>>(
    table: &InstructionTable<W, H>,
    mut f: FN,
) -> CustomInstructionTable<CI>
where
    W: InterpreterTypes,
    H: Host + ?Sized,
    FN: FnMut(Instruction<W, H>) -> CI,
{
    core::array::from_fn(|i| f(table[i]))
}

// TODO
// /// Updates a boxed instruction with a new one.
// #[inline]
// pub fn update_custom_instruction<W, H, F>(
//     instruction: &mut impl CustomInstruction<Wire = W, Host = H>,
//     f: F,
// ) where
//     W: InterpreterTypes,
//     H: Host + ?Sized,
//     F: Fn(&DynInstruction<W, H>, &mut W, &mut H),
// {
//     // Note: This first allocation gets elided by the compiler.
//     let prev = core::mem::replace(instruction, Box::new(|_, _| {}));
//     *instruction = Box::new(move |i, h| f(&prev, i, h));
// }
