use context::ContextTrait;
use interpreter::{
    table::{make_instruction_table, InstructionTable},
    Host, Interpreter, InterpreterAction, InterpreterTypes,
};
use std::rc::Rc;

// TODO rename to Instructions. It should store the instructions on
// plan and inspect execution.
pub trait InstructionExecutor {
    type Context;
    type InterpreterTypes: InterpreterTypes;
    type Output;

    fn plain_instruction_table(&self) -> &InstructionTable<Self::InterpreterTypes, Self::Context>;

    fn inspector_instruction_table(
        &self,
    ) -> &InstructionTable<Self::InterpreterTypes, Self::Context>;
}

pub struct EthInstructions<WIRE: InterpreterTypes, HOST> {
    pub instruction_table: Rc<InstructionTable<WIRE, HOST>>,
    pub inspector_table: Rc<InstructionTable<WIRE, HOST>>,
    pub inspection_enabled: bool,
}

pub trait InstructionExecutorGetter {
    type InstructionExecutor: InstructionExecutor;

    fn executor(&mut self) -> &mut Self::InstructionExecutor;
}

impl<WIRE, HOST> Clone for EthInstructions<WIRE, HOST>
where
    WIRE: InterpreterTypes,
{
    fn clone(&self) -> Self {
        Self {
            instruction_table: self.instruction_table.clone(),
            inspector_table: self.inspector_table.clone(),
            inspection_enabled: false,
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
        // TODO make a wrapper for inspector calls.
        let inspector_table = base_table.clone();
        Self {
            instruction_table: Rc::new(base_table),
            inspector_table: Rc::new(inspector_table),
            inspection_enabled: false,
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

impl<IT, CTX> InstructionExecutor for EthInstructions<IT, CTX>
where
    IT: InterpreterTypes,
    CTX: Host,
{
    type InterpreterTypes = IT;
    type Context = CTX;
    /// TODO Interpreter action could be tied to InterpreterTypes so we can
    /// set custom actions from instructions.
    type Output = InterpreterAction;

    fn plain_instruction_table(&self) -> &InstructionTable<Self::InterpreterTypes, Self::Context> {
        &self.instruction_table
    }

    fn inspector_instruction_table(
        &self,
    ) -> &InstructionTable<Self::InterpreterTypes, Self::Context> {
        &self.inspector_table
    }
}

/*

Frame< Inspector:


*/

impl<WIRE, HOST> Default for EthInstructions<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host,
{
    fn default() -> Self {
        Self::new_mainnet()
    }
}
