use revm::{
    context::{setters::ContextSetters, EvmData},
    context_interface::ContextGetters,
    handler::{
        handler::EvmTypesTrait,
        instructions::{EthInstructions, InstructionExecutor},
    },
    interpreter::{interpreter::EthInterpreter, Host, Interpreter, InterpreterAction},
};

use crate::handler::precompiles::OpPrecompileProvider;

pub struct OpEvm<CTX, INSP, I> {
    pub data: EvmData<CTX, INSP>,
    pub enabled_inspection: bool,
    pub instruction: I,
    pub precompiles: OpPrecompileProvider<CTX>,
}

impl<CTX: Host, INSP> OpEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>> {
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self {
            data: EvmData { ctx, inspector },
            enabled_inspection: false,
            instruction: EthInstructions::new_mainnet(),
            precompiles: OpPrecompileProvider::default(),
        }
    }
}

impl<CTX: ContextSetters, INSP, I> ContextSetters for OpEvm<CTX, INSP, I> {
    type Tx = <CTX as ContextSetters>::Tx;
    type Block = <CTX as ContextSetters>::Block;

    fn set_tx(&mut self, tx: Self::Tx) {
        self.data.ctx.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.data.ctx.set_block(block);
    }
}

impl<CTX, INSP, I> EvmTypesTrait for OpEvm<CTX, INSP, I>
where
    CTX: ContextGetters,
    I: InstructionExecutor<Context = CTX, Output = InterpreterAction>,
{
    type Context = CTX;
    type Inspector = INSP;
    type Instructions = I;
    type Precompiles = OpPrecompileProvider<Self::Context>;

    fn run_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionExecutor>::InterpreterTypes,
        >,
    ) -> <Self::Instructions as InstructionExecutor>::Output {
        let (ctx, instructions) = self.ctx_instructions();
        interpreter.run_plain(instructions.plain_instruction_table(), ctx)
        // TODO if self.enabled_inspection {}
    }

    fn enable_inspection(&mut self, enable: bool) {
        self.enabled_inspection = enable;
    }

    fn ctx(&mut self) -> &mut Self::Context {
        &mut self.data.ctx
    }

    fn ctx_ref(&self) -> &Self::Context {
        &self.data.ctx
    }

    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        (&mut self.data.ctx, &mut self.data.inspector)
    }

    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions) {
        (&mut self.data.ctx, &mut self.instruction)
    }

    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles) {
        (&mut self.data.ctx, &mut self.precompiles)
    }
}
