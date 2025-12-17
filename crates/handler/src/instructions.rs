use auto_impl::auto_impl;
use context::SetSpecTr;
use interpreter::{
    instructions::{instruction_table_gas_changes_spec, InstructionTable},
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
}

/// Ethereum instruction contains list of mainnet instructions that is used for Interpreter execution.
#[derive(Debug)]
pub struct EthInstructions<WIRE: InterpreterTypes, HOST: ?Sized> {
    /// Table containing instruction implementations indexed by opcode.
    pub instruction_table: Box<InstructionTable<WIRE, HOST>>,
    /// Spec that is used to set gas costs for instructions.
    pub spec: SpecId,
}

impl<WIRE, HOST: Host + ?Sized> Clone for EthInstructions<WIRE, HOST>
where
    WIRE: InterpreterTypes,
{
    fn clone(&self) -> Self {
        Self {
            instruction_table: self.instruction_table.clone(),
            spec: self.spec,
        }
    }
}

impl<WIRE, HOST, SPEC: Into<SpecId> + Clone> SetSpecTr<SPEC> for EthInstructions<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host,
{
    #[inline]
    fn set_spec(&mut self, spec: SPEC) {
        let spec = spec.into();
        if spec == self.spec {
            return;
        }
        self.spec = spec;
        self.instruction_table = Box::new(instruction_table_gas_changes_spec(spec));
    }
}

impl<WIRE, HOST> EthInstructions<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host,
{
    /// Returns `EthInstructions` with mainnet spec.
    pub fn new_mainnet() -> Self {
        let spec = SpecId::default();
        Self::new(instruction_table_gas_changes_spec(spec), spec)
    }

    /// Returns `EthInstructions` with mainnet spec.
    pub fn new_mainnet_with_spec(spec: SpecId) -> Self {
        Self::new(instruction_table_gas_changes_spec(spec), spec)
    }

    /// Returns a new instance of `EthInstructions` with custom instruction table.
    #[inline]
    pub fn new(base_table: InstructionTable<WIRE, HOST>, spec: SpecId) -> Self {
        Self {
            instruction_table: Box::new(base_table),
            spec,
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
