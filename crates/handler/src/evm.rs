use crate::{instructions::InstructionProvider, PrecompileProvider};
use auto_impl::auto_impl;
use context::{ContextTr, Evm};
use interpreter::{Interpreter, InterpreterAction, InterpreterTypes};

/// A trait that integrates context, instruction set, and precompiles to create an EVM struct.
///
/// In addition to execution capabilities, this trait provides getter methods for its component fields.
#[auto_impl(&mut, Box)]
pub trait EvmTr {
    /// The context type that implements ContextTr to provide access to execution state
    type Context: ContextTr;
    /// The instruction set type that implements InstructionProvider to define available operations
    type Instructions: InstructionProvider;
    /// The type containing the available precompiled contracts
    type Precompiles: PrecompileProvider<Self::Context>;

    /// Executes the interpreter loop for the given interpreter instance.
    /// Returns either a completion status or the next interpreter action to take.
    fn run_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output;

    /// Returns a mutable reference to the execution context
    fn ctx(&mut self) -> &mut Self::Context;

    /// Returns an immutable reference to the execution context
    fn ctx_ref(&self) -> &Self::Context;

    /// Returns mutable references to both the context and instruction set.
    /// This enables atomic access to both components when needed.
    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions);

    /// Returns mutable references to both the context and precompiles.
    /// This enables atomic access to both components when needed.
    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles);
}

impl<CTX, INSP, I, P> EvmTr for Evm<CTX, INSP, I, P>
where
    CTX: ContextTr,
    I: InstructionProvider<
        Context = CTX,
        InterpreterTypes: InterpreterTypes<Output = InterpreterAction>,
    >,
    P: PrecompileProvider<CTX>,
{
    type Context = CTX;
    type Instructions = I;
    type Precompiles = P;

    #[inline]
    fn run_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output
    {
        let context = &mut self.ctx;
        let instructions = &mut self.instruction;
        interpreter.run_plain(instructions.instruction_table(), context)
    }
    #[inline]
    fn ctx(&mut self) -> &mut Self::Context {
        &mut self.ctx
    }

    #[inline]
    fn ctx_ref(&self) -> &Self::Context {
        &self.ctx
    }

    #[inline]
    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions) {
        (&mut self.ctx, &mut self.instruction)
    }

    #[inline]
    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles) {
        (&mut self.ctx, &mut self.precompiles)
    }
}
