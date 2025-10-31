use auto_impl::auto_impl;
use interpreter::{
    instructions::{instruction_table, instruction_table_gas_changes_spec, InstructionTable},
    Host, Instruction, InterpreterTypes,
};
use primitives::hardfork::SpecId;
use std::boxed::Box;

/// Stores instructions for EVM.
#[auto_impl(&mut, Box)]
pub trait InstructionProvider {
    /// Context type.
    type Context;
    /// Interpreter types.
    type InterpreterTypes: InterpreterTypes;

    /// Returns the instruction table that is used by EvmTr to execute instructions.
    fn instruction_table(&self) -> &InstructionTable<Self::InterpreterTypes, Self::Context>;

    /// Sets the spec id.
    fn set_spec_id(&mut self, spec_id: SpecId);
}

/// Ethereum instruction contains list of mainnet instructions that is used for Interpreter execution.
#[derive(Debug)]
pub struct EthInstructions<WIRE: InterpreterTypes, HOST: ?Sized> {
    /// Table containing instruction implementations indexed by opcode.
    pub instruction_table: Box<InstructionTable<WIRE, HOST>>,
}

impl<WIRE, HOST: Host + ?Sized> Clone for EthInstructions<WIRE, HOST>
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

    /// Returns a new instance of `EthInstructions` with custom instruction table.
    #[inline]
    pub fn new(base_table: InstructionTable<WIRE, HOST>) -> Self {
        Self {
            instruction_table: Box::new(base_table),
        }
    }

    /// Inserts a new instruction into the instruction table.
    #[inline]
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

    fn set_spec_id(&mut self, spec_id: SpecId) {
        self.instruction_table = Box::new(instruction_table_gas_changes_spec(spec_id));
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
