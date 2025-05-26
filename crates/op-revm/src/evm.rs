use crate::precompiles::OpPrecompiles;
use revm::{
    context::{ContextSetters, Evm},
    context_interface::ContextTr,
    handler::{
        instructions::{EthInstructions, InstructionProvider},
        EvmTr, PrecompileProvider,
    },
    inspector::{InspectorEvmTr, JournalExt},
    interpreter::{interpreter::EthInterpreter, Interpreter, InterpreterAction, InterpreterTypes},
    Inspector,
};

pub struct OpEvm<CTX, INSP, I = EthInstructions<EthInterpreter, CTX>, P = OpPrecompiles>(
    pub Evm<CTX, INSP, I, P>,
);

impl<CTX: ContextTr, INSP> OpEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, OpPrecompiles> {
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self(Evm {
            ctx,
            inspector,
            instruction: EthInstructions::new_mainnet(),
            precompiles: OpPrecompiles::default(),
        })
    }
}

impl<CTX, INSP, I, P> OpEvm<CTX, INSP, I, P> {
    /// Consumed self and returns a new Evm type with given Inspector.
    pub fn with_inspector<OINSP>(self, inspector: OINSP) -> OpEvm<CTX, OINSP, I, P> {
        OpEvm(self.0.with_inspector(inspector))
    }

    /// Consumes self and returns a new Evm type with given Precompiles.
    pub fn with_precompiles<OP>(self, precompiles: OP) -> OpEvm<CTX, INSP, I, OP> {
        OpEvm(self.0.with_precompiles(precompiles))
    }

    /// Consumes self and returns the inner Inspector.
    pub fn into_inspector(self) -> INSP {
        self.0.into_inspector()
    }
}

impl<CTX, INSP, I, P> InspectorEvmTr for OpEvm<CTX, INSP, I, P>
where
    CTX: ContextTr<Journal: JournalExt> + ContextSetters,
    I: InstructionProvider<
        Context = CTX,
        InterpreterTypes: InterpreterTypes<Output = InterpreterAction>,
    >,
    P: PrecompileProvider<CTX>,
    INSP: Inspector<CTX, I::InterpreterTypes>,
{
    type Inspector = INSP;

    fn inspector(&mut self) -> &mut Self::Inspector {
        &mut self.0.inspector
    }

    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        (&mut self.0.ctx, &mut self.0.inspector)
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
    P: PrecompileProvider<CTX>,
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
        let context = &mut self.0.ctx;
        let instructions = &mut self.0.instruction;
        interpreter.run_plain(instructions.instruction_table(), context)
    }

    fn ctx(&mut self) -> &mut Self::Context {
        &mut self.0.ctx
    }

    fn ctx_ref(&self) -> &Self::Context {
        &self.0.ctx
    }

    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions) {
        (&mut self.0.ctx, &mut self.0.instruction)
    }

    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles) {
        (&mut self.0.ctx, &mut self.0.precompiles)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        precompiles::bn128_pair::GRANITE_MAX_INPUT_SIZE,
        transaction::deposit::DEPOSIT_TRANSACTION_TYPE, DefaultOp, L1BlockInfo, OpBuilder,
        OpHaltReason, OpSpecId, OpTransaction,
    };
    use revm::{
        bytecode::opcode,
        context::{
            result::{ExecutionResult, OutOfGasError},
            BlockEnv, CfgEnv, TxEnv,
        },
        context_interface::result::HaltReason,
        database::{BenchmarkDB, EmptyDB, BENCH_CALLER, BENCH_CALLER_BALANCE, BENCH_TARGET},
        interpreter::{
            gas::{calculate_initial_tx_gas, InitialAndFloorGas},
            Interpreter, InterpreterTypes,
        },
        precompile::{bls12_381_const, bls12_381_utils, bn128, secp256r1, u64_to_address},
        primitives::{Address, Bytes, Log, TxKind, U256},
        state::Bytecode,
        Context, ExecuteEvm, InspectEvm, Inspector, Journal,
    };
    use std::vec::Vec;

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

    fn p256verify_test_tx() -> Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpecId>,
        EmptyDB,
        Journal<EmptyDB>,
        L1BlockInfo,
    > {
        const SPEC_ID: OpSpecId = OpSpecId::FJORD;

        let InitialAndFloorGas { initial_gas, .. } =
            calculate_initial_tx_gas(SPEC_ID.into(), &[], false, 0, 0, 0);

        Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(u64_to_address(secp256r1::P256VERIFY_ADDRESS));
                tx.base.gas_limit = initial_gas + secp256r1::P256VERIFY_BASE_GAS_FEE;
            })
            .modify_cfg_chained(|cfg| cfg.spec = SPEC_ID)
    }

    #[test]
    fn test_tx_call_p256verify() {
        let ctx = p256verify_test_tx();

        let mut evm = ctx.build_op();
        let output = evm.replay().unwrap();

        // assert successful call to P256VERIFY
        assert!(output.result.is_success());
    }

    #[test]
    fn test_halted_tx_call_p256verify() {
        let ctx = p256verify_test_tx().modify_tx_chained(|tx| tx.base.gas_limit -= 1);

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

    fn bn128_pair_test_tx(
        spec: OpSpecId,
    ) -> Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpecId>,
        EmptyDB,
        Journal<EmptyDB>,
        L1BlockInfo,
    > {
        let input = Bytes::from([1; GRANITE_MAX_INPUT_SIZE + 2]);
        let InitialAndFloorGas { initial_gas, .. } =
            calculate_initial_tx_gas(spec.into(), &input[..], false, 0, 0, 0);

        Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bn128::pair::ADDRESS);
                tx.base.data = input;
                tx.base.gas_limit = initial_gas;
            })
            .modify_cfg_chained(|cfg| cfg.spec = spec)
    }

    #[test]
    fn test_halted_tx_call_bn128_pair_fjord() {
        let ctx = bn128_pair_test_tx(OpSpecId::FJORD);

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
        let ctx = bn128_pair_test_tx(OpSpecId::GRANITE);

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

    #[test]
    fn test_halted_tx_call_bls12_381_g1_add_out_of_gas() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bls12_381_const::G1_ADD_ADDRESS);
                tx.base.gas_limit = 21_000 + bls12_381_const::G1_ADD_BASE_GAS_FEE - 1;
            })
            .modify_chain_chained(|l1_block| {
                l1_block.operator_fee_constant = Some(U256::ZERO);
                l1_block.operator_fee_scalar = Some(U256::ZERO)
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::ISTHMUS);

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
    fn test_halted_tx_call_bls12_381_g1_add_input_wrong_size() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bls12_381_const::G1_ADD_ADDRESS);
                tx.base.gas_limit = 21_000 + bls12_381_const::G1_ADD_BASE_GAS_FEE;
            })
            .modify_chain_chained(|l1_block| {
                l1_block.operator_fee_constant = Some(U256::ZERO);
                l1_block.operator_fee_scalar = Some(U256::ZERO)
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::ISTHMUS);

        let mut evm = ctx.build_op();
        let output = evm.replay().unwrap();

        // assert fails post gas check, because input is wrong size
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::PrecompileError),
                ..
            }
        ));
    }

    fn g1_msm_test_tx() -> Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpecId>,
        EmptyDB,
        Journal<EmptyDB>,
        L1BlockInfo,
    > {
        const SPEC_ID: OpSpecId = OpSpecId::ISTHMUS;

        let input = Bytes::from([1; bls12_381_const::G1_MSM_INPUT_LENGTH]);
        let InitialAndFloorGas { initial_gas, .. } =
            calculate_initial_tx_gas(SPEC_ID.into(), &input[..], false, 0, 0, 0);
        let gs1_msm_gas = bls12_381_utils::msm_required_gas(
            1,
            &bls12_381_const::DISCOUNT_TABLE_G1_MSM,
            bls12_381_const::G1_MSM_BASE_GAS_FEE,
        );

        Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bls12_381_const::G1_MSM_ADDRESS);
                tx.base.data = input;
                tx.base.gas_limit = initial_gas + gs1_msm_gas;
            })
            .modify_chain_chained(|l1_block| {
                l1_block.operator_fee_constant = Some(U256::ZERO);
                l1_block.operator_fee_scalar = Some(U256::ZERO)
            })
            .modify_cfg_chained(|cfg| cfg.spec = SPEC_ID)
    }

    #[test]
    fn test_halted_tx_call_bls12_381_g1_msm_input_wrong_size() {
        let ctx = g1_msm_test_tx().modify_tx_chained(|tx| tx.base.data = tx.base.data.slice(1..));

        let mut evm = ctx.build_op();
        let output = evm.replay().unwrap();

        // assert fails pre gas check, because input is wrong size
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::PrecompileError),
                ..
            }
        ));
    }

    #[test]
    fn test_halted_tx_call_bls12_381_g1_msm_out_of_gas() {
        let ctx = g1_msm_test_tx().modify_tx_chained(|tx| tx.base.gas_limit -= 1);

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
    fn test_halted_tx_call_bls12_381_g1_msm_wrong_input_layout() {
        let ctx = g1_msm_test_tx();

        let mut evm = ctx.build_op();
        let output = evm.replay().unwrap();

        // assert fails post gas check, because input is wrong layout
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::PrecompileError),
                ..
            }
        ));
    }

    #[test]
    fn test_halted_tx_call_bls12_381_g2_add_out_of_gas() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bls12_381_const::G2_ADD_ADDRESS);
                tx.base.gas_limit = 21_000 + bls12_381_const::G2_ADD_BASE_GAS_FEE - 1;
            })
            .modify_chain_chained(|l1_block| {
                l1_block.operator_fee_constant = Some(U256::ZERO);
                l1_block.operator_fee_scalar = Some(U256::ZERO)
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::ISTHMUS);

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
    fn test_halted_tx_call_bls12_381_g2_add_input_wrong_size() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bls12_381_const::G2_ADD_ADDRESS);
                tx.base.gas_limit = 21_000 + bls12_381_const::G2_ADD_BASE_GAS_FEE;
            })
            .modify_chain_chained(|l1_block| {
                l1_block.operator_fee_constant = Some(U256::ZERO);
                l1_block.operator_fee_scalar = Some(U256::ZERO)
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::ISTHMUS);

        let mut evm = ctx.build_op();

        let output = evm.replay().unwrap();

        // assert fails post gas check, because input is wrong size
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::PrecompileError),
                ..
            }
        ));
    }

    fn g2_msm_test_tx() -> Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpecId>,
        EmptyDB,
        Journal<EmptyDB>,
        L1BlockInfo,
    > {
        const SPEC_ID: OpSpecId = OpSpecId::ISTHMUS;

        let input = Bytes::from([1; bls12_381_const::G2_MSM_INPUT_LENGTH]);
        let InitialAndFloorGas { initial_gas, .. } =
            calculate_initial_tx_gas(SPEC_ID.into(), &input[..], false, 0, 0, 0);
        let gs2_msm_gas = bls12_381_utils::msm_required_gas(
            1,
            &bls12_381_const::DISCOUNT_TABLE_G2_MSM,
            bls12_381_const::G2_MSM_BASE_GAS_FEE,
        );

        Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bls12_381_const::G2_MSM_ADDRESS);
                tx.base.data = input;
                tx.base.gas_limit = initial_gas + gs2_msm_gas;
            })
            .modify_chain_chained(|l1_block| {
                l1_block.operator_fee_constant = Some(U256::ZERO);
                l1_block.operator_fee_scalar = Some(U256::ZERO)
            })
            .modify_cfg_chained(|cfg| cfg.spec = SPEC_ID)
    }

    #[test]
    fn test_halted_tx_call_bls12_381_g2_msm_input_wrong_size() {
        let ctx = g2_msm_test_tx().modify_tx_chained(|tx| tx.base.data = tx.base.data.slice(1..));

        let mut evm = ctx.build_op();
        let output = evm.replay().unwrap();

        // assert fails pre gas check, because input is wrong size
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::PrecompileError),
                ..
            }
        ));
    }

    #[test]
    fn test_halted_tx_call_bls12_381_g2_msm_out_of_gas() {
        let ctx = g2_msm_test_tx().modify_tx_chained(|tx| tx.base.gas_limit -= 1);

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
    fn test_halted_tx_call_bls12_381_g2_msm_wrong_input_layout() {
        let ctx = g2_msm_test_tx();

        let mut evm = ctx.build_op();
        let output = evm.replay().unwrap();

        // assert fails post gas check, because input is wrong layout
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::PrecompileError),
                ..
            }
        ));
    }

    fn bl12_381_pairing_test_tx() -> Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpecId>,
        EmptyDB,
        Journal<EmptyDB>,
        L1BlockInfo,
    > {
        const SPEC_ID: OpSpecId = OpSpecId::ISTHMUS;

        let input = Bytes::from([1; bls12_381_const::PAIRING_INPUT_LENGTH]);
        let InitialAndFloorGas { initial_gas, .. } =
            calculate_initial_tx_gas(SPEC_ID.into(), &input[..], false, 0, 0, 0);

        let pairing_gas: u64 =
            bls12_381_const::PAIRING_MULTIPLIER_BASE + bls12_381_const::PAIRING_OFFSET_BASE;

        Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bls12_381_const::PAIRING_ADDRESS);
                tx.base.data = input;
                tx.base.gas_limit = initial_gas + pairing_gas;
            })
            .modify_chain_chained(|l1_block| {
                l1_block.operator_fee_constant = Some(U256::ZERO);
                l1_block.operator_fee_scalar = Some(U256::ZERO)
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::ISTHMUS)
    }

    #[test]
    fn test_halted_tx_call_bls12_381_pairing_input_wrong_size() {
        let ctx = bl12_381_pairing_test_tx()
            .modify_tx_chained(|tx| tx.base.data = tx.base.data.slice(1..));

        let mut evm = ctx.build_op();
        let output = evm.replay().unwrap();

        // assert fails pre gas check, because input is wrong size
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::PrecompileError),
                ..
            }
        ));
    }

    #[test]
    fn test_halted_tx_call_bls12_381_pairing_out_of_gas() {
        let ctx = bl12_381_pairing_test_tx().modify_tx_chained(|tx| tx.base.gas_limit -= 1);

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
    fn test_tx_call_bls12_381_pairing_wrong_input_layout() {
        let ctx = bl12_381_pairing_test_tx();

        let mut evm = ctx.build_op();
        let output = evm.replay().unwrap();

        // assert fails post gas check, because input is wrong layout
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::PrecompileError),
                ..
            }
        ));
    }

    fn fp_to_g1_test_tx() -> Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpecId>,
        EmptyDB,
        Journal<EmptyDB>,
        L1BlockInfo,
    > {
        const SPEC_ID: OpSpecId = OpSpecId::ISTHMUS;

        let input = Bytes::from([1; bls12_381_const::PADDED_FP_LENGTH]);
        let InitialAndFloorGas { initial_gas, .. } =
            calculate_initial_tx_gas(SPEC_ID.into(), &input[..], false, 0, 0, 0);

        Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bls12_381_const::MAP_FP_TO_G1_ADDRESS);
                tx.base.data = input;
                tx.base.gas_limit = initial_gas + bls12_381_const::MAP_FP_TO_G1_BASE_GAS_FEE;
            })
            .modify_chain_chained(|l1_block| {
                l1_block.operator_fee_constant = Some(U256::ZERO);
                l1_block.operator_fee_scalar = Some(U256::ZERO)
            })
            .modify_cfg_chained(|cfg| cfg.spec = SPEC_ID)
    }

    #[test]
    fn test_halted_tx_call_bls12_381_map_fp_to_g1_out_of_gas() {
        let ctx = fp_to_g1_test_tx().modify_tx_chained(|tx| tx.base.gas_limit -= 1);

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
    fn test_halted_tx_call_bls12_381_map_fp_to_g1_input_wrong_size() {
        let ctx = fp_to_g1_test_tx().modify_tx_chained(|tx| tx.base.data = tx.base.data.slice(1..));

        let mut evm = ctx.build_op();
        let output = evm.replay().unwrap();

        // assert fails post gas check, because input is wrong size
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::PrecompileError),
                ..
            }
        ));
    }

    fn fp2_to_g2_test_tx() -> Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpecId>,
        EmptyDB,
        Journal<EmptyDB>,
        L1BlockInfo,
    > {
        const SPEC_ID: OpSpecId = OpSpecId::ISTHMUS;

        let input = Bytes::from([1; bls12_381_const::PADDED_FP2_LENGTH]);
        let InitialAndFloorGas { initial_gas, .. } =
            calculate_initial_tx_gas(SPEC_ID.into(), &input[..], false, 0, 0, 0);

        Context::op()
            .modify_tx_chained(|tx| {
                tx.base.kind = TxKind::Call(bls12_381_const::MAP_FP2_TO_G2_ADDRESS);
                tx.base.data = input;
                tx.base.gas_limit = initial_gas + bls12_381_const::MAP_FP2_TO_G2_BASE_GAS_FEE;
            })
            .modify_chain_chained(|l1_block| {
                l1_block.operator_fee_constant = Some(U256::ZERO);
                l1_block.operator_fee_scalar = Some(U256::ZERO)
            })
            .modify_cfg_chained(|cfg| cfg.spec = SPEC_ID)
    }

    #[test]
    fn test_halted_tx_call_bls12_381_map_fp2_to_g2_out_of_gas() {
        let ctx = fp2_to_g2_test_tx().modify_tx_chained(|tx| tx.base.gas_limit -= 1);

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
    fn test_halted_tx_call_bls12_381_map_fp2_to_g2_input_wrong_size() {
        let ctx =
            fp2_to_g2_test_tx().modify_tx_chained(|tx| tx.base.data = tx.base.data.slice(1..));

        let mut evm = ctx.build_op();
        let output = evm.replay().unwrap();

        // assert fails post gas check, because input is wrong size
        assert!(matches!(
            output.result,
            ExecutionResult::Halt {
                reason: OpHaltReason::Base(HaltReason::PrecompileError),
                ..
            }
        ));
    }

    #[derive(Default, Debug)]
    struct LogInspector {
        logs: Vec<Log>,
    }

    impl<CTX, INTR: InterpreterTypes> Inspector<CTX, INTR> for LogInspector {
        fn log(&mut self, _interp: &mut Interpreter<INTR>, _context: &mut CTX, log: Log) {
            self.logs.push(log)
        }
    }

    #[test]
    fn test_log_inspector() {
        // simple yul contract emits a log in constructor

        /*object "Contract" {
            code {
                log0(0, 0)
            }
        }*/

        let contract_data: Bytes = Bytes::from([
            opcode::PUSH1,
            0x00,
            opcode::DUP1,
            opcode::LOG0,
            opcode::STOP,
        ]);
        let bytecode = Bytecode::new_raw(contract_data);

        let ctx = Context::op()
            .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
            .modify_tx_chained(|tx| {
                tx.base.caller = BENCH_CALLER;
                tx.base.kind = TxKind::Call(BENCH_TARGET);
            });

        let mut evm = ctx.build_op_with_inspector(LogInspector::default());

        // Run evm.
        let _ = evm.inspect_replay().unwrap();

        let inspector = &evm.0.inspector;
        assert!(!inspector.logs.is_empty());
    }
}
