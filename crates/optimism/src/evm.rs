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

#[cfg(test)]
mod tests {
    use crate::{
        transaction::deposit::DEPOSIT_TRANSACTION_TYPE, DefaultOp, OpBuilder, OpHaltReason,
        OpSpecId,
    };
    use database::{BenchmarkDB, BENCH_CALLER, BENCH_CALLER_BALANCE, BENCH_TARGET};
    use precompile::Address;
    use revm::{
        bytecode::opcode,
        context::result::ExecutionResult,
        primitives::{TxKind, U256},
        state::Bytecode,
        Context, ExecuteEvm,
    };

    #[test]
    fn test_deposit_tx() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.enveloped_tx = None;
                tx.deposit.mint = Some(100);
                tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::HOLOCENE);

        let mut evm = ctx.build_op();

        let output = evm.transact_previous().unwrap();

        // balance should be 100
        assert_eq!(
            output
                .state
                .get(&Address::default())
                .map(|a| a.info.balance),
            Some(U256::from(100))
        );
    }

    #[test]
    fn test_halted_deposit_tx() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.enveloped_tx = None;
                tx.deposit.mint = Some(100);
                tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
                tx.base.caller = BENCH_CALLER;
                tx.base.kind = TxKind::Call(BENCH_TARGET);
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::HOLOCENE)
            .with_db(BenchmarkDB::new_bytecode(Bytecode::new_legacy(
                [opcode::POP].into(),
            )));

        // POP would return a halt.
        let mut evm = ctx.build_op();

        let output = evm.transact_previous().unwrap();

        // balance should be 100 + previous balance
        assert_eq!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::FailedDeposit,
                gas_used: 30_000_000
            }
        );
        assert_eq!(
            output.state.get(&BENCH_CALLER).map(|a| a.info.balance),
            Some(U256::from(100) + BENCH_CALLER_BALANCE)
        );
    }
}
