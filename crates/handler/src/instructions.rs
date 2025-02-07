use auto_impl::auto_impl;
use context_interface::ContextTrait;
use interpreter::{
    table::{make_instruction_table, InstructionTable},
    Host, Interpreter, InterpreterAction, InterpreterTypes,
};
use std::rc::Rc;

/// Stores instructions for EVM.
#[auto_impl(&, Arc, Rc)]
pub trait InstructionProvider {
    type Context;
    type InterpreterTypes: InterpreterTypes;
    type Output;

    fn instruction_table(&self) -> &InstructionTable<Self::InterpreterTypes, Self::Context>;
}

pub struct EthInstructions<WIRE: InterpreterTypes, HOST> {
    pub instruction_table: Rc<InstructionTable<WIRE, HOST>>,
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
    pub fn new_mainnet() -> Self {
        Self::new(make_instruction_table::<WIRE, HOST>())
    }

    pub fn new(base_table: InstructionTable<WIRE, HOST>) -> Self {
        Self {
            instruction_table: Rc::new(base_table),
        }
    }
}

pub trait ContextInspectRun {
    type InterpreterTypes: InterpreterTypes;
    type Context: ContextTrait + Host;

    fn run_context(
        &mut self,
        interpretere: Interpreter<Self::InterpreterTypes>,
        instructions: &InstructionTable<Self::InterpreterTypes, Self::Context>,
    );
}

impl<IT, CTX> InstructionProvider for EthInstructions<IT, CTX>
where
    IT: InterpreterTypes,
    CTX: Host,
{
    type InterpreterTypes = IT;
    type Context = CTX;
    /// TODO Interpreter action could be tied to InterpreterTypes so we can
    /// set custom actions from instructions.
    type Output = InterpreterAction;

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
