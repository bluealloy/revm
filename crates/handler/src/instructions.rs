use auto_impl::auto_impl;
use interpreter::{
    instructions::{instruction_table, InstructionTable},
    Host, Instruction, InterpreterTypes,
};
use std::boxed::Box;

/// Stores instructions for EVM.
#[auto_impl(&, Arc, Rc)]
pub trait InstructionProvider {
    /// Context type.
    type Context;
    /// Interpreter types.
    type InterpreterTypes: InterpreterTypes;

    /// Returns the instruction table that is used by EvmTr to execute instructions.
    fn instruction_table(&self) -> &InstructionTable<Self::InterpreterTypes, Self::Context>;
}

/// Ethereum instruction contains list of mainnet instructions that is used for Interpreter execution.
pub struct EthInstructions<WIRE: InterpreterTypes, HOST> {
    pub instruction_table: Box<InstructionTable<WIRE, HOST>>,
}

impl<WIRE, HOST> Clone for EthInstructions<WIRE, HOST>
where
    WIRE: InterpreterTypes,
{
    fn clone(&self) -> Self {
        Self {
            instruction_table: self.instruction_table.clone(),
        }
    }
}

impl<WIRE, HOST> EthInstructions<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host,
{
    /// Returns `EthInstructions` with mainnet spec.
    pub fn new_mainnet() -> Self {
        Self::new(instruction_table::<WIRE, HOST>())
    }

    /// Rerurns new `EthInstructions` with custom instruction table.
    pub fn new(base_table: InstructionTable<WIRE, HOST>) -> Self {
        Self {
            instruction_table: Box::new(base_table),
        }
    }

    /// Inserts a new instruction into the instruction table.s
    pub fn insert_instruction(&mut self, opcode: u8, instruction: Instruction<WIRE, HOST>) {
        self.instruction_table[opcode as usize] = instruction;
    }
}

impl<IT, CTX> InstructionProvider for EthInstructions<IT, CTX>
where
    IT: InterpreterTypes,
    CTX: Host,
{
    type InterpreterTypes = IT;
    type Context = CTX;

    fn instruction_table(&self) -> &InstructionTable<Self::InterpreterTypes, Self::Context> {
        &self.instruction_table
    }
}

impl<WIRE, HOST> Default for EthInstructions<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host,
{
    fn default() -> Self {
        Self::new_mainnet()
    }
}
