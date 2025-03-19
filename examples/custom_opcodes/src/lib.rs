//! Custom opcodes example
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use revm::{
    context::{ContextTr, Evm},
    handler::instructions::{EthInstructions, InstructionProvider},
    interpreter::{interpreter::EthInterpreter, InstructionTable},
};

pub struct MyOpcodeEvm<CTX, INSP, P>(Evm<CTX, INSP, MyInstructions<CTX>, P>);

pub struct MyInstructions<CTX>(pub EthInstructions<EthInterpreter, CTX>);

impl<CTX: ContextTr> InstructionProvider for MyInstructions<CTX> {
    type Context = CTX;
    type InterpreterTypes = EthInterpreter;

    fn instruction_table(&self) -> &InstructionTable<Self::InterpreterTypes, Self::Context> {
        self.0.instruction_table()
    }
}


