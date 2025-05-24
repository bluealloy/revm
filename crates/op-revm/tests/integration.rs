mod common;

use common::compare_or_save_testdata;
use op_revm::{
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
    compare_or_save_testdata("test_deposit_tx.json", &output);
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

    compare_or_save_testdata("test_halted_deposit_tx.json", &output);
}

fn p256verify_test_tx(
) -> Context<BlockEnv, OpTransaction<TxEnv>, CfgEnv<OpSpecId>, EmptyDB, Journal<EmptyDB>, L1BlockInfo>
{
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

    compare_or_save_testdata("test_tx_call_p256verify.json", &output);
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

    compare_or_save_testdata("test_halted_tx_call_p256verify.json", &output);
}

fn bn128_pair_test_tx(
    spec: OpSpecId,
) -> Context<BlockEnv, OpTransaction<TxEnv>, CfgEnv<OpSpecId>, EmptyDB, Journal<EmptyDB>, L1BlockInfo>
{
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

    compare_or_save_testdata("test_halted_tx_call_bn128_pair_fjord.json", &output);
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

    compare_or_save_testdata("test_halted_tx_call_bn128_pair_granite.json", &output);
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_g1_add_out_of_gas.json",
        &output,
    );
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_g1_add_input_wrong_size.json",
        &output,
    );
}

fn g1_msm_test_tx(
) -> Context<BlockEnv, OpTransaction<TxEnv>, CfgEnv<OpSpecId>, EmptyDB, Journal<EmptyDB>, L1BlockInfo>
{
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_g1_msm_input_wrong_size.json",
        &output,
    );
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_g1_msm_out_of_gas.json",
        &output,
    );
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_g1_msm_wrong_input_layout.json",
        &output,
    );
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_g2_add_out_of_gas.json",
        &output,
    );
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_g2_add_input_wrong_size.json",
        &output,
    );
}

fn g2_msm_test_tx(
) -> Context<BlockEnv, OpTransaction<TxEnv>, CfgEnv<OpSpecId>, EmptyDB, Journal<EmptyDB>, L1BlockInfo>
{
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_g2_msm_input_wrong_size.json",
        &output,
    );
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_g2_msm_out_of_gas.json",
        &output,
    );
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_g2_msm_wrong_input_layout.json",
        &output,
    );
}

fn bl12_381_pairing_test_tx(
) -> Context<BlockEnv, OpTransaction<TxEnv>, CfgEnv<OpSpecId>, EmptyDB, Journal<EmptyDB>, L1BlockInfo>
{
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
    let ctx =
        bl12_381_pairing_test_tx().modify_tx_chained(|tx| tx.base.data = tx.base.data.slice(1..));

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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_pairing_input_wrong_size.json",
        &output,
    );
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_pairing_out_of_gas.json",
        &output,
    );
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_pairing_wrong_input_layout.json",
        &output,
    );
}

fn fp_to_g1_test_tx(
) -> Context<BlockEnv, OpTransaction<TxEnv>, CfgEnv<OpSpecId>, EmptyDB, Journal<EmptyDB>, L1BlockInfo>
{
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_map_fp_to_g1_out_of_gas.json",
        &output,
    );
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_map_fp_to_g1_input_wrong_size.json",
        &output,
    );
}

fn fp2_to_g2_test_tx(
) -> Context<BlockEnv, OpTransaction<TxEnv>, CfgEnv<OpSpecId>, EmptyDB, Journal<EmptyDB>, L1BlockInfo>
{
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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_map_fp2_to_g2_out_of_gas.json",
        &output,
    );
}

#[test]
fn test_halted_tx_call_bls12_381_map_fp2_to_g2_input_wrong_size() {
    let ctx = fp2_to_g2_test_tx().modify_tx_chained(|tx| tx.base.data = tx.base.data.slice(1..));

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

    compare_or_save_testdata(
        "test_halted_tx_call_bls12_381_map_fp2_to_g2_input_wrong_size.json",
        &output,
    );
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

    let ctx = Context::op().with_db(BenchmarkDB::new_bytecode(bytecode.clone()));

    let mut evm = ctx.build_op_with_inspector(LogInspector::default());

    let tx = OpTransaction {
        base: TxEnv {
            caller: BENCH_CALLER,
            kind: TxKind::Call(BENCH_TARGET),
            ..Default::default()
        },
        ..Default::default()
    };

    // Run evm.
    let output = evm.inspect_tx_finalize(tx).unwrap();

    let inspector = &evm.0.inspector;
    assert!(!inspector.logs.is_empty());

    compare_or_save_testdata("test_log_inspector.json", &output);
}
