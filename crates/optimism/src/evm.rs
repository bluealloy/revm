use revm::{
    context::{setters::ContextSetters, Evm, EvmData},
    context_interface::ContextTr,
    handler::{
        handler::EvmTr,
        instructions::{EthInstructions, InstructionProvider},
    },
    interpreter::{interpreter::EthInterpreter, Host, Interpreter, InterpreterAction},
};

use crate::handler::precompiles::OpPrecompileProvider;

pub struct OpEvm<CTX, INSP, I = EthInstructions<EthInterpreter, CTX>, P = OpPrecompileProvider<CTX>>(
    pub Evm<CTX, INSP, I, P>,
);

impl<CTX: Host, INSP>
    OpEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, OpPrecompileProvider<CTX>>
{
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self(Evm {
            data: EvmData { ctx, inspector },
            instruction: EthInstructions::new_mainnet(),
            precompiles: OpPrecompileProvider::default(),
        })
    }
}

impl<CTX: ContextSetters, INSP, I, P> ContextSetters for OpEvm<CTX, INSP, I, P> {
    type Tx = <CTX as ContextSetters>::Tx;
    type Block = <CTX as ContextSetters>::Block;

    fn set_tx(&mut self, tx: Self::Tx) {
        self.0.data.ctx.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.0.data.ctx.set_block(block);
    }
}

impl<CTX, INSP, I, P> EvmTr for OpEvm<CTX, INSP, I, P>
where
    CTX: ContextTr,
    I: InstructionProvider<Context = CTX, Output = InterpreterAction>,
{
    type Context = CTX;
    type Instructions = I;
    type Precompiles = P;

    fn run_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <Self::Instructions as InstructionProvider>::Output {
        let context = &mut self.0.data.ctx;
        let instructions = &mut self.0.instruction;
        interpreter.run_plain(instructions.instruction_table(), context)
    }

    fn ctx(&mut self) -> &mut Self::Context {
        &mut self.0.data.ctx
    }

    fn ctx_ref(&self) -> &Self::Context {
        &self.0.data.ctx
    }

    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions) {
        (&mut self.0.data.ctx, &mut self.0.instruction)
    }

    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles) {
        (&mut self.0.data.ctx, &mut self.0.precompiles)
    }
}
