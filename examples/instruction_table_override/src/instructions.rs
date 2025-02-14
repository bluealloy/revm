use revm::{
    handler::instructions::InstructionExecutor,
    interpreter::{
        gas,
        interpreter_types::{LoopControl, StackTrait},
        popn_top,
        table::{make_instruction_table, InstructionTable},
        Host, Interpreter, InterpreterAction, InterpreterTypes,
    },
    primitives::U256,
};
use std::rc::Rc;

const CLZ: u8 = 0x5f;

pub struct CustomInstructionExecutor<WIRE: InterpreterTypes, HOST> {
    instruction_table: Rc<InstructionTable<WIRE, HOST>>,
}

impl<WIRE, HOST> Clone for CustomInstructionExecutor<WIRE, HOST>
where
    WIRE: InterpreterTypes,
{
    fn clone(&self) -> Self {
        Self {
            instruction_table: self.instruction_table.clone(),
        }
    }
}

impl<WIRE, HOST> CustomInstructionExecutor<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    WIRE::Stack: StackTrait,
    HOST: Host,
{
    pub fn new() -> Self {
        let mut table = make_instruction_table::<WIRE, HOST>();
        table[CLZ as usize] = |interpreter: &mut Interpreter<WIRE>, _host: &mut HOST| {
            revm::interpreter::gas!(interpreter, gas::VERYLOW);
            popn_top!([], value, interpreter);
            let value: &mut U256 = value;
            let leading_zeros = value.leading_zeros();
            *value = U256::from(leading_zeros);
        };
        Self {
            instruction_table: Rc::new(table),
        }
    }
}

impl<IT, CTX> InstructionExecutor for CustomInstructionExecutor<IT, CTX>
where
    IT: InterpreterTypes,
    CTX: Host,
{
    type InterpreterTypes = IT;
    type CTX = CTX;
    type Output = InterpreterAction;

    fn run(
        &mut self,
        context: &mut Self::CTX,
        interpreter: &mut Interpreter<Self::InterpreterTypes>,
    ) -> Self::Output {
        interpreter.run_plain(self.instruction_table.as_ref(), context)
    }
}

impl<WIRE, HOST> Default for CustomInstructionExecutor<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host,
{
    fn default() -> Self {
        Self::new()
    }
}
