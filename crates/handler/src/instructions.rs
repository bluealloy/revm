use interpreter::{
    table::{make_instruction_table, InstructionTable},
    Host, Interpreter, InterpreterAction, InterpreterTypes,
};
use std::rc::Rc;

pub trait InstructionExecutor {
    type CTX;
    type InterpreterTypes: InterpreterTypes;
    type Output;

    fn run(
        &mut self,
        context: &mut Self::CTX,
        interpreter: &mut Interpreter<Self::InterpreterTypes>,
    ) -> Self::Output;
}

pub struct EthInstructionExecutor<WIRE: InterpreterTypes, HOST> {
    pub instruction_table: Rc<InstructionTable<WIRE, HOST>>,
}

pub trait InstructionExecutorGetter {
    type InstructionExecutor: InstructionExecutor;

    fn executor(&mut self) -> &mut Self::InstructionExecutor;
}

impl<WIRE, HOST> Clone for EthInstructionExecutor<WIRE, HOST>
where
    WIRE: InterpreterTypes,
{
    fn clone(&self) -> Self {
        Self {
            instruction_table: self.instruction_table.clone(),
        }
    }
}

impl<WIRE, HOST> EthInstructionExecutor<WIRE, HOST>
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

impl<IT, CTX> InstructionExecutor for EthInstructionExecutor<IT, CTX>
where
    IT: InterpreterTypes,
    CTX: Host,
{
    type InterpreterTypes = IT;
    type CTX = CTX;
    /// TODO Interpreter action could be tied to InterpreterTypes so we can
    /// set custom actions from instructions.
    type Output = InterpreterAction;

    fn run(
        &mut self,
        context: &mut Self::CTX,
        interpreter: &mut Interpreter<Self::InterpreterTypes>,
    ) -> Self::Output {
        interpreter.run_plain(self.instruction_table.as_ref(), context)
    }
}

impl<WIRE, HOST> Default for EthInstructionExecutor<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host,
{
    fn default() -> Self {
        Self::new_mainnet()
    }
}
