#![allow(clippy::wrong_self_convention)]

use super::instruction;
use crate::{instructions::control, primitives::Spec, Host, Interpreter};
use std::boxed::Box;

/// EVM opcode function signature.
pub type Instruction<H> = fn(&mut Interpreter, &mut H);

/// Instruction table is list of instruction function pointers mapped to 256 EVM opcodes.
pub type InstructionTable<H> = [Instruction<H>; 256];

/// EVM dynamic opcode function signature.
pub type DynInstruction<'a, H> = dyn Fn(&mut Interpreter, &mut H) + 'a;

/// EVM boxed dynamic opcode function signature.
pub type BoxedInstruction<'a, H> = Box<DynInstruction<'a, H>>;

/// A table of boxed instructions.
pub type BoxedInstructionTable<'a, H> = [BoxedInstruction<'a, H>; 256];

/// Either a plain, static instruction table, or a boxed, dynamic instruction table.
///
/// Note that `Plain` variant is about 10-20% faster in Interpreter execution.
pub enum InstructionTables<'a, H: ?Sized> {
    Plain(InstructionTable<H>),
    Boxed(BoxedInstructionTable<'a, H>),
}

impl<'a, H: Host + ?Sized> InstructionTables<'a, H> {
    /// Creates a plain instruction table for the given spec. See [`make_instruction_table`].
    #[inline]
    pub const fn new_plain<SPEC: Spec>() -> Self {
        Self::Plain(make_instruction_table::<H, SPEC>())
    }
}

impl<'a, H: Host + ?Sized + 'a> InstructionTables<'a, H> {
    /// Inserts the instruction into the table with the specified index.
    #[inline]
    pub fn insert(&mut self, opcode: u8, instruction: Instruction<H>) {
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
    fn to_boxed_with_slow<F>(&mut self, f: F) -> &mut BoxedInstructionTable<'a, H>
    where
        F: FnMut(Instruction<H>) -> BoxedInstruction<'a, H>,
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
        instruction: BoxedInstruction<'a, H>,
    ) -> BoxedInstruction<'a, H> {
        core::mem::replace(self.get_boxed(opcode), instruction)
    }

    /// Updates a single instruction in the table at the specified index with `f`.
    #[inline]
    pub fn update_boxed<F>(&mut self, opcode: u8, f: F)
    where
        F: Fn(&DynInstruction<'a, H>, &mut Interpreter, &mut H) + 'a,
    {
        update_boxed_instruction(self.get_boxed(opcode), f)
    }

    /// Updates every instruction in the table by calling `f`.
    #[inline]
    pub fn update_all<F>(&mut self, f: F)
    where
        F: Fn(&DynInstruction<'a, H>, &mut Interpreter, &mut H) + Copy + 'a,
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
pub const fn make_instruction_table<H: Host + ?Sized, SPEC: Spec>() -> InstructionTable<H> {
    // Force const-eval of the table creation, making this function trivial.
    // TODO: Replace this with a `const {}` block once it is stable.
    struct ConstTable<H: Host + ?Sized, SPEC: Spec> {
        _host: core::marker::PhantomData<H>,
        _spec: core::marker::PhantomData<SPEC>,
    }
    impl<H: Host + ?Sized, SPEC: Spec> ConstTable<H, SPEC> {
        const NEW: InstructionTable<H> = {
            let mut tables: InstructionTable<H> = [control::unknown; 256];
            let mut i = 0;
            while i < 256 {
                tables[i] = instruction::<H, SPEC>(i as u8);
                i += 1;
            }
            tables
        };
    }
    ConstTable::<H, SPEC>::NEW
}

/// Make boxed instruction table that calls `f` closure for every instruction.
#[inline]
pub fn make_boxed_instruction_table<'a, H, FN>(
    table: &InstructionTable<H>,
    mut f: FN,
) -> BoxedInstructionTable<'a, H>
where
    H: Host + ?Sized,
    FN: FnMut(Instruction<H>) -> BoxedInstruction<'a, H>,
{
    core::array::from_fn(|i| f(table[i]))
}

/// Updates a boxed instruction with a new one.
#[inline]
pub fn update_boxed_instruction<'a, H, F>(instruction: &mut BoxedInstruction<'a, H>, f: F)
where
    H: Host + ?Sized + 'a,
    F: Fn(&DynInstruction<'a, H>, &mut Interpreter, &mut H) + 'a,
{
    // NOTE: This first allocation gets elided by the compiler.
    let prev = core::mem::replace(instruction, Box::new(|_, _| {}));
    *instruction = Box::new(move |i, h| f(&prev, i, h));
}
