use auto_impl::auto_impl;
use interpreter::{
    instructions::{gas_table_spec, GasTable, InstructionTable},
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

    /// Returns the gas table for static gas costs.
    fn gas_table(&self) -> &GasTable;
}

/// Ethereum instruction contains list of mainnet instructions that is used for Interpreter execution.
#[derive(Debug)]
pub struct EthInstructions<WIRE: InterpreterTypes, HOST: ?Sized> {
    /// Spec that is used to set gas costs for instructions.
    pub spec: SpecId,
    inner: Box<EthInstructionsInner<WIRE, HOST>>,
}

#[derive(Debug)]
struct EthInstructionsInner<WIRE: InterpreterTypes, HOST: ?Sized> {
    /// Table containing instruction implementations indexed by opcode.
    instruction_table: InstructionTable<WIRE, HOST>,
    /// Static gas cost table indexed by opcode.
    gas_table: GasTable,
}

impl<WIRE, HOST: Host + ?Sized> Clone for EthInstructions<WIRE, HOST>
where
    WIRE: InterpreterTypes,
{
    fn clone(&self) -> Self {
        Self {
            spec: self.spec,
            inner: self.inner.clone(),
        }
    }
}

impl<WIRE, HOST: Host + ?Sized> Clone for EthInstructionsInner<WIRE, HOST>
where
    WIRE: InterpreterTypes,
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<WIRE, HOST: Host + ?Sized> Copy for EthInstructionsInner<WIRE, HOST> where
    WIRE: InterpreterTypes
{
}

impl<WIRE, HOST> EthInstructions<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host,
{
    /// Returns `EthInstructions` with mainnet spec.
    #[deprecated(since = "0.2.0", note = "use new_mainnet_with_spec instead")]
    pub fn new_mainnet() -> Self {
        let spec = SpecId::default();
        Self::new_mainnet_with_spec(spec)
    }

    /// Returns `EthInstructions` with mainnet spec.
    pub fn new_mainnet_with_spec(spec: SpecId) -> Self {
        Self::new(interpreter::instruction_table(), gas_table_spec(spec), spec)
    }

    /// Returns a new instance of `EthInstructions` with custom instruction and gas tables.
    pub fn new(
        instruction_table: InstructionTable<WIRE, HOST>,
        gas_table: GasTable,
        spec: SpecId,
    ) -> Self {
        Self {
            spec,
            inner: Box::new(EthInstructionsInner {
                instruction_table,
                gas_table,
            }),
        }
    }

    /// Inserts a new instruction into the instruction table.
    #[inline]
    pub fn insert_instruction(
        &mut self,
        opcode: u8,
        instruction: Instruction<WIRE, HOST>,
        gas: u16,
    ) {
        self.inner.instruction_table[opcode as usize] = instruction;
        self.inner.gas_table[opcode as usize] = gas;
    }

    /// Inserts a new gas cost into the gas table.
    #[inline]
    pub fn insert_gas(&mut self, opcode: u8, gas: u16) {
        self.inner.gas_table[opcode as usize] = gas;
    }

    /// Returns a reference to the instruction table.
    #[inline]
    pub fn instruction_table(&self) -> &InstructionTable<WIRE, HOST> {
        &self.inner.instruction_table
    }

    /// Returns a mutable reference to the instruction table.
    #[inline]
    pub fn instruction_table_mut(&mut self) -> &mut InstructionTable<WIRE, HOST> {
        &mut self.inner.instruction_table
    }

    /// Returns a reference to the gas table.
    #[inline]
    pub fn gas_table(&self) -> &GasTable {
        &self.inner.gas_table
    }

    /// Returns a mutable reference to the gas table.
    #[inline]
    pub fn gas_table_mut(&mut self) -> &mut GasTable {
        &mut self.inner.gas_table
    }
}

impl<IT, CTX> InstructionProvider for EthInstructions<IT, CTX>
where
    IT: InterpreterTypes,
    CTX: Host,
{
    type InterpreterTypes = IT;
    type Context = CTX;

    #[inline]
    fn instruction_table(&self) -> &InstructionTable<Self::InterpreterTypes, Self::Context> {
        self.instruction_table()
    }

    #[inline]
    fn gas_table(&self) -> &GasTable {
        self.gas_table()
    }
}
