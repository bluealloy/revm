use crate::precompiles::OpPrecompiles;
use revm::{
    context::{ContextSetters, Evm, EvmData},
    context_interface::ContextTr,
    handler::{
        instructions::{EthInstructions, InstructionProvider},
        EvmTr,
    },
    inspector::{InspectorEvmTr, JournalExt},
    interpreter::{
        interpreter::EthInterpreter, Host, Interpreter, InterpreterAction, InterpreterTypes,
    },
    Inspector,
};

pub struct OpEvm<CTX, INSP, I = EthInstructions<EthInterpreter, CTX>, P = OpPrecompiles>(
    pub Evm<CTX, INSP, I, P>,
);

impl<CTX: Host, INSP> OpEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, OpPrecompiles> {
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self(Evm {
            data: EvmData { ctx, inspector },
            instruction: EthInstructions::new_mainnet(),
            precompiles: OpPrecompiles::default(),
        })
    }
}

impl<CTX, INSP, I, P> InspectorEvmTr for OpEvm<CTX, INSP, I, P>
where
    CTX: ContextTr<Journal: JournalExt> + ContextSetters,
    I: InstructionProvider<
        Context = CTX,
        InterpreterTypes: InterpreterTypes<Output = InterpreterAction>,
    >,
    INSP: Inspector<CTX, I::InterpreterTypes>,
{
    type Inspector = INSP;

    fn inspector(&mut self) -> &mut Self::Inspector {
        &mut self.0.data.inspector
    }

    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        (&mut self.0.data.ctx, &mut self.0.data.inspector)
    }

    fn run_inspect_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output
    {
        self.0.run_inspect_interpreter(interpreter)
    }
}

impl<CTX, INSP, I, P> EvmTr for OpEvm<CTX, INSP, I, P>
where
    CTX: ContextTr,
    I: InstructionProvider<
        Context = CTX,
        InterpreterTypes: InterpreterTypes<Output = InterpreterAction>,
    >,
{
    type Context = CTX;
    type Instructions = I;
    type Precompiles = P;

    fn run_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output
    {
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
        precompiles::bn128_pair::GRANITE_MAX_INPUT_SIZE,
        transaction::deposit::DEPOSIT_TRANSACTION_TYPE, DefaultOp, OpBuilder, OpHaltReason,
        OpSpecId,
    };
    use revm::{
        bytecode::opcode,
        context::result::{ExecutionResult, OutOfGasError},
        context_interface::result::HaltReason,
        database::{BenchmarkDB, BENCH_CALLER, BENCH_CALLER_BALANCE, BENCH_TARGET},
        precompile::{bn128, u64_to_address},
        primitives::{Address, Bytes, TxKind, U256},
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

        let output = evm.replay().unwrap();

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

        let output = evm.replay().unwrap();

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

    #[test]
    fn test_tx_call_p256verify() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(u64_to_address(256));
                tx.base.gas_limit = 24_450; // P256VERIFY base is 3450
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::FJORD);

        let mut evm = ctx.build_op();

        let output = evm.replay().unwrap();

        // assert successful call to P256VERIFY
        assert!(output.result.is_success());
    }

    #[test]
    fn test_halted_tx_call_p256verify() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(u64_to_address(256));
                tx.base.gas_limit = 24_449; // 1 gas low
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::FJORD);

        let mut evm = ctx.build_op();

        let output = evm.replay().unwrap();

        // assert out of gas for P256VERIFY
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::OutOfGas(OutOfGasError::Precompile)),
                ..
            }
        ));
    }

    #[test]
    fn test_halted_tx_call_bn128_pair_fjord() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bn128::pair::ADDRESS);
                tx.base.data = Bytes::from([1; GRANITE_MAX_INPUT_SIZE + 2].to_vec());
                tx.base.gas_limit = 19_969_000; // gas needed by bn128::pair for input len
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::FJORD);

        let mut evm = ctx.build_op();

        let output = evm.replay().unwrap();

        // assert out of gas
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::OutOfGas(OutOfGasError::Precompile)),
                ..
            }
        ));
    }

    #[test]
    fn test_halted_tx_call_bn128_pair_granite() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bn128::pair::ADDRESS);
                tx.base.data = Bytes::from([1; GRANITE_MAX_INPUT_SIZE + 2].to_vec());
                tx.base.gas_limit = 19_969_000; // gas needed by bn128::pair for input len
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::GRANITE);

        let mut evm = ctx.build_op();

        let output = evm.replay().unwrap();

        // assert bails early because input size too big
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::PrecompileError),
                ..
            }
        ));
    }
}
