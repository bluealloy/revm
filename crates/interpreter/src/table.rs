#![allow(clippy::wrong_self_convention)]

use crate::{
    instructions::{control, instruction},
    interpreter::InterpreterTrait,
    Host, Interpreter,
};
use specification::hardfork::Spec;
use std::boxed::Box;

/// EVM opcode function signature.
pub type Instruction<I, H> = fn(&mut I, &mut H);

/// Instruction table is list of instruction function pointers mapped to 256 EVM opcodes.
pub type InstructionTable<I, H> = [Instruction<I, H>; 256];

/// EVM dynamic opcode function signature.
pub type DynInstruction<I, H> = dyn Fn(&mut I, &mut H);

/// A table of boxed instructions.
pub type CustomInstructionTable<IT> = [IT; 256];

pub trait CustomInstruction {
    type Interpreter;
    type Host: ?Sized;

    fn exec(&mut self, interpreter: &mut Self::Interpreter, host: &mut Self::Host);
}

pub struct EthInstructionImpl<I, H: ?Sized> {
    function: Instruction<I, H>,
}

impl<I, H: ?Sized> CustomInstruction for EthInstructionImpl<I, H> {
    type Interpreter = I;
    type Host = H;

    fn exec(&mut self, interpreter: &mut Self::Interpreter, host: &mut Self::Host) {
        (self.function)(interpreter, host);
    }
}

/// Either a plain, static instruction table, or a boxed, dynamic instruction table.
///
/// Note that `Plain` variant is about 10-20% faster in Interpreter execution.
pub enum InstructionTables<I, H: ?Sized, CI: CustomInstruction<Host = H, Interpreter = I>> {
    Plain(InstructionTable<I, H>),
    Custom(CustomInstructionTable<CI>),
}

impl<I: InterpreterTrait, H: Host + ?Sized> InstructionTables<I, H, EthInstructionImpl<I, H>> {
    /// Creates a plain instruction table for the given spec. See [`make_instruction_table`].
    #[inline]
    pub const fn new_plain<SPEC: Spec>() -> Self {
        Self::Plain(make_instruction_table::<I, H>())
    }
}

impl<I, H, CI> InstructionTables<I, H, CI>
where
    I: InterpreterTrait,
    H: Host + ?Sized,
    CI: CustomInstruction<Host = H, Interpreter = I>,
{
    /// Inserts the instruction into the table with the specified index.
    #[inline]
    pub fn insert(&mut self, opcode: u8, instruction: Instruction<I, H>) {
        match self {
            Self::Plain(table) => table[opcode as usize] = instruction,
            Self::Boxed(table) => table[opcode as usize] = Box::new(instruction),
        }
    }

    /// Converts the current instruction table to a boxed variant if it is not already, and returns
    /// a mutable reference to the boxed table.
    #[inline]
    pub fn to_boxed(&mut self) -> &mut BoxedInstructionTable<'a, H> {
        self.to_boxed_with(|i| Box::new(i))
    }

    /// Converts the current instruction table to a boxed variant if it is not already with `f`,
    /// and returns a mutable reference to the boxed table.
    #[inline]
    pub fn to_boxed_with<F>(&mut self, f: F) -> &mut BoxedInstructionTable<'a, H>
    where
        F: FnMut(Instruction<H>) -> BoxedInstruction<'a, H>,
    {
        match self {
            Self::Plain(_) => self.to_boxed_with_slow(f),
            Self::Boxed(boxed) => boxed,
        }
    }

    #[cold]
    fn to_boxed_with_slow<OCI: CustomInstruction<Interpreter = I, Host = H>, F>(
        &mut self,
        f: F,
    ) -> &mut BoxedInstructionTable<I, H, OCI>
    where
        F: FnMut(Instruction<I, H>) -> OCI,
    {
        let Self::Plain(table) = self else {
            unreachable!()
        };
        *self = Self::Boxed(make_boxed_instruction_table(table, f));
        let Self::Boxed(boxed) = self else {
            unreachable!()
        };
        boxed
    }

    /// Returns a mutable reference to the boxed instruction at the specified index.
    #[inline]
    pub fn get_boxed(&mut self, opcode: u8) -> &mut BoxedInstruction<'a, H> {
        &mut self.to_boxed()[opcode as usize]
    }

    /// Inserts a boxed instruction into the table at the specified index.
    #[inline]
    pub fn insert_boxed(&mut self, opcode: u8, instruction: BoxedInstruction<'a, H>) {
        *self.get_boxed(opcode) = instruction;
    }

    /// Replaces a boxed instruction into the table at the specified index, returning the previous
    /// instruction.
    #[inline]
    pub fn replace_boxed(
        &mut self,
        opcode: u8,
        instruction: BoxedInstruction<I, H>,
    ) -> BoxedInstruction<I, H> {
        core::mem::replace(self.get_boxed(opcode), instruction)
    }

    /// Updates a single instruction in the table at the specified index with `f`.
    #[inline]
    pub fn update_boxed<F>(&mut self, opcode: u8, f: F)
    where
        F: Fn(&DynInstruction<I, H>, &mut Interpreter, &mut H) + 'a,
    {
        update_boxed_instruction(self.get_boxed(opcode), f)
    }

    /// Updates every instruction in the table by calling `f`.
    #[inline]
    pub fn update_all<F>(&mut self, f: F)
    where
        F: Fn(&DynInstruction<I, H>, &mut Interpreter, &mut H) + Copy + 'a,
    {
        // Don't go through `to_boxed` to avoid allocating the plain table twice.
        match self {
            Self::Plain(_) => {
                self.to_boxed_with(|prev| Box::new(move |i, h| f(&prev, i, h)));
            }
            Self::Boxed(boxed) => boxed
                .iter_mut()
                .for_each(|instruction| update_boxed_instruction(instruction, f)),
        }
    }
}

/// Make instruction table.
#[inline]
pub const fn make_instruction_table<I: InterpreterTrait, H: Host + ?Sized>(
) -> InstructionTable<I, H> {
    const {
        let mut tables: InstructionTable<I, H> = [control::unknown; 256];
        let mut i = 0;
        while i < 256 {
            tables[i] = instruction::<I, H>(i as u8);
            i += 1;
        }
        tables
    }
}

/// Make boxed instruction table that calls `f` closure for every instruction.
#[inline]
pub fn make_custom_instruction_table<I, H, FN, CI: CustomInstruction<Interpreter = I, Host = H>>(
    table: &InstructionTable<I, H>,
    mut f: FN,
) -> CustomInstructionTable<CI>
where
    H: Host + ?Sized,
    FN: FnMut(Instruction<I, H>) -> CI,
{
    core::array::from_fn(|i| f(table[i]))
}

/// Updates a boxed instruction with a new one.
#[inline]
pub fn update_custom_instruction<'a, H, F>(instruction: &mut CustomInstruction<I, H>, f: F)
where
    H: Host + ?Sized + 'a,
    F: Fn(&DynInstruction<'a, H>, &mut Interpreter, &mut H) + 'a,
{
    // NOTE: This first allocation gets elided by the compiler.
    let prev = core::mem::replace(instruction, Box::new(|_, _| {}));
    *instruction = Box::new(move |i, h| f(&prev, i, h));
}
