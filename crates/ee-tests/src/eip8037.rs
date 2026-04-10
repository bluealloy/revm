//! EIP-8037 State Gas integration tests.
//!
//! Verifies dual-limit gas accounting where storage creation gas (state gas)
//! is tracked separately from regular gas.

use crate::TestdataConfig;
use revm::{
    bytecode::opcode,
    context::TxEnv,
    context_interface::{cfg::GasId, result::HaltReason},
    database::{BenchmarkDB, BENCH_CALLER},
    handler::{MainnetContext, MainnetEvm},
    primitives::{address, hardfork::SpecId, U256},
    state::Bytecode,
    Context, ExecuteEvm, MainBuilder, MainContext,
};
use std::path::PathBuf;

const TIP1016_TESTDATA: &str = "tests/eip8037_testdata";

fn eip8037_testdata_config() -> TestdataConfig {
    TestdataConfig {
        testdata_dir: PathBuf::from(TIP1016_TESTDATA),
    }
}

fn compare_or_save_eip8037_testdata<T>(filename: &str, output: &T)
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a> + PartialEq + std::fmt::Debug,
{
    crate::compare_or_save_testdata_with_config(filename, output, eip8037_testdata_config());
}

/// State gas costs used across all TIP-1016 tests.
const STATE_GAS_SSTORE_SET: u64 = 200_000;
const STATE_GAS_NEW_ACCOUNT: u64 = 250_000;
const STATE_GAS_CODE_DEPOSIT: u64 = 1000; // per byte
const STATE_GAS_CREATE: u64 = 330_000;

/// EIP-8037 hash cost for deployed bytecode: 6 × ceil(len / 32).
/// This is a regular gas cost only charged when EIP-8037 is enabled.
const fn hash_cost(len: usize) -> u64 {
    6 * (len as u64).div_ceil(32)
}

type MainEvm = MainnetEvm<MainnetContext<BenchmarkDB>>;

/// Builds an EVM with state gas enabled and custom gas params.
fn state_gas_evm(bytecode: Bytecode, cap: u64) -> MainEvm {
    Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.tx_gas_limit_cap = Some(cap);
            cfg.gas_params.override_gas([
                (GasId::sstore_set_state_gas(), STATE_GAS_SSTORE_SET),
                (GasId::new_account_state_gas(), STATE_GAS_NEW_ACCOUNT),
                (GasId::code_deposit_state_gas(), STATE_GAS_CODE_DEPOSIT),
                (GasId::create_state_gas(), STATE_GAS_CREATE),
            ]);
        })
        .with_db(BenchmarkDB::new_bytecode(bytecode))
        .build_mainnet()
}

/// Builds an EVM without state gas (standard behavior, no cap).
fn baseline_evm(bytecode: Bytecode) -> MainEvm {
    Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.enable_amsterdam_eip8037 = false;
            cfg.tx_gas_limit_cap = Some(u64::MAX);
        })
        .with_db(BenchmarkDB::new_bytecode(bytecode))
        .build_mainnet()
}

/// Bytecode: SSTORE(key, value); STOP
/// Stores `value` at storage slot `key`.
fn sstore_bytecode(key: u8, value: u8) -> Bytecode {
    Bytecode::new_legacy(
        [
            opcode::PUSH1,
            value, // value
            opcode::PUSH1,
            key,            // key
            opcode::SSTORE, //
            opcode::STOP,
        ]
        .into(),
    )
}

/// Bytecode: SSTORE(0, 1); SSTORE(0, 2); STOP
/// Two writes to the same slot.
fn sstore_overwrite_bytecode() -> Bytecode {
    Bytecode::new_legacy(
        [
            opcode::PUSH1,
            1, // value=1
            opcode::PUSH1,
            0,              // key=0
            opcode::SSTORE, //
            opcode::PUSH1,
            2, // value=2
            opcode::PUSH1,
            0,              // key=0
            opcode::SSTORE, //
            opcode::STOP,
        ]
        .into(),
    )
}

/// Bytecode: SSTORE(0, 1); SSTORE(1, 1); SSTORE(2, 1); STOP
/// Three new slots.
fn sstore_multi_bytecode() -> Bytecode {
    Bytecode::new_legacy(
        [
            opcode::PUSH1,
            1,
            opcode::PUSH1,
            0,
            opcode::SSTORE, //
            opcode::PUSH1,
            1,
            opcode::PUSH1,
            1,
            opcode::SSTORE, //
            opcode::PUSH1,
            1,
            opcode::PUSH1,
            2,
            opcode::SSTORE, //
            opcode::STOP,
        ]
        .into(),
    )
}

/// Bytecode: SSTORE(0, 1); SSTORE(0, 0); STOP
/// Set then clear — triggers refund but state gas persists.
fn sstore_set_then_clear_bytecode() -> Bytecode {
    Bytecode::new_legacy(
        [
            opcode::PUSH1,
            1, // value=1
            opcode::PUSH1,
            0,              // key=0
            opcode::SSTORE, //
            opcode::PUSH1,
            0, // value=0
            opcode::PUSH1,
            0,              // key=0
            opcode::SSTORE, //
            opcode::STOP,
        ]
        .into(),
    )
}

/// Init code that returns `code_len` zero bytes as runtime code.
fn return_n_bytes_init_code(code_len: u8) -> Vec<u8> {
    // PUSH1 code_len; PUSH1 0; RETURN (returns `code_len` zero bytes from memory).
    vec![opcode::PUSH1, code_len, opcode::PUSH1, 0, opcode::RETURN]
}

/// Init code that does SSTORE(0, 1) and returns 1 byte of code.
fn init_code_sstore_and_return() -> Vec<u8> {
    vec![
        // SSTORE(0, 1)
        opcode::PUSH1,
        1, // value
        opcode::PUSH1,
        0,              // key
        opcode::SSTORE, //
        // RETURN 1 byte of zero from memory
        opcode::PUSH1,
        1, // length
        opcode::PUSH1,
        0, // offset
        opcode::RETURN,
    ]
}

/// Init code that does SSTORE(0, 1) and then REVERT.
fn init_code_sstore_and_revert() -> Vec<u8> {
    vec![
        // SSTORE(0, 1)
        opcode::PUSH1,
        1, // value
        opcode::PUSH1,
        0,              // key
        opcode::SSTORE, //
        // REVERT(0, 0)
        opcode::PUSH1,
        0,
        opcode::PUSH1,
        0,
        opcode::REVERT,
    ]
}

/// Bytecode that executes CREATE with given init code (no value).
fn create_bytecode(init_code: &[u8]) -> Bytecode {
    assert!(init_code.len() < 256);
    let mut bytecode = Vec::new();
    // Store init code in memory byte by byte
    for (i, byte) in init_code.iter().enumerate() {
        bytecode.push(opcode::PUSH1);
        bytecode.push(*byte);
        bytecode.push(opcode::PUSH1);
        bytecode.push(i as u8);
        bytecode.push(opcode::MSTORE8);
    }
    // CREATE(value=0, offset=0, length)
    bytecode.push(opcode::PUSH1);
    bytecode.push(init_code.len() as u8);
    bytecode.push(opcode::PUSH1);
    bytecode.push(0);
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // value = 0
    bytecode.push(opcode::CREATE);
    bytecode.push(opcode::POP);
    bytecode.push(opcode::STOP);
    Bytecode::new_legacy(bytecode.into())
}

/// Bytecode: SSTORE(0, 1); REVERT(0, 0)
/// Does one state gas charge (SSTORE), then explicit revert.
fn sstore_then_revert_bytecode() -> Bytecode {
    Bytecode::new_legacy(
        [
            opcode::PUSH1,
            1, // value
            opcode::PUSH1,
            0,              // key
            opcode::SSTORE, //
            // REVERT(0, 0)
            opcode::PUSH1,
            0,
            opcode::PUSH1,
            0,
            opcode::REVERT,
        ]
        .into(),
    )
}

/// Bytecode: SSTORE(0, 1); SSTORE(1, 1); REVERT(0, 0)
/// Two state gas charges (40,000 total), then explicit revert.
fn sstore_multi_then_revert_bytecode() -> Bytecode {
    Bytecode::new_legacy(
        [
            opcode::PUSH1,
            1,
            opcode::PUSH1,
            0,
            opcode::SSTORE, //
            opcode::PUSH1,
            1,
            opcode::PUSH1,
            1,
            opcode::SSTORE, //
            // REVERT(0, 0)
            opcode::PUSH1,
            0,
            opcode::PUSH1,
            0,
            opcode::REVERT,
        ]
        .into(),
    )
}

/// Bytecode that performs a CALL with value to a specific address.
#[allow(clippy::vec_init_then_push)]
fn call_with_value_bytecode(target: [u8; 20], value: U256) -> Bytecode {
    // CALL(gas, addr, value, argsOffset, argsSize, retOffset, retSize)
    let mut bytecode = Vec::new();

    // Push return size (0)
    bytecode.push(opcode::PUSH1);
    bytecode.push(0);

    // Push return offset (0)
    bytecode.push(opcode::PUSH1);
    bytecode.push(0);

    // Push args size (0)
    bytecode.push(opcode::PUSH1);
    bytecode.push(0);

    // Push args offset (0)
    bytecode.push(opcode::PUSH1);
    bytecode.push(0);

    // Push value (32 bytes)
    let value_bytes = value.to_be_bytes::<32>();
    bytecode.push(opcode::PUSH32);
    bytecode.extend_from_slice(&value_bytes);

    // Push target address (20 bytes)
    bytecode.push(opcode::PUSH20);
    bytecode.extend_from_slice(&target);

    // Push gas (use all remaining gas)
    bytecode.push(opcode::GAS);

    // Execute CALL
    bytecode.push(opcode::CALL);

    // Clean up stack
    bytecode.push(opcode::POP);

    // Stop
    bytecode.push(opcode::STOP);

    Bytecode::new_legacy(bytecode.into())
}

/// Bytecode that executes CREATE2 with given init code (no value, salt=0).
fn create2_bytecode(init_code: &[u8]) -> Bytecode {
    assert!(init_code.len() < 256);
    let mut bytecode = Vec::new();
    // Store init code in memory byte by byte
    for (i, byte) in init_code.iter().enumerate() {
        bytecode.push(opcode::PUSH1);
        bytecode.push(*byte);
        bytecode.push(opcode::PUSH1);
        bytecode.push(i as u8);
        bytecode.push(opcode::MSTORE8);
    }
    // CREATE2(value=0, offset=0, length, salt=0)
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // salt = 0
    bytecode.push(opcode::PUSH1);
    bytecode.push(init_code.len() as u8); // size
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // offset = 0
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // value = 0
    bytecode.push(opcode::CREATE2);
    bytecode.push(opcode::POP);
    bytecode.push(opcode::STOP);
    Bytecode::new_legacy(bytecode.into())
}

/// Bytecode: PUSH20 <beneficiary>; SELFDESTRUCT
fn selfdestruct_bytecode(beneficiary: [u8; 20]) -> Bytecode {
    let mut bytecode = Vec::new();
    bytecode.push(opcode::PUSH20);
    bytecode.extend_from_slice(&beneficiary);
    bytecode.push(opcode::SELFDESTRUCT);
    Bytecode::new_legacy(bytecode.into())
}

/// Bytecode that CREATEs a contract with the given child runtime code,
/// then CALLs the newly created contract.
fn call_created_child_bytecode(child_runtime: &[u8]) -> Bytecode {
    assert!(child_runtime.len() < 128, "child_runtime too large");

    // Build init code: stores child_runtime in memory byte-by-byte, then RETURN(0, len).
    let mut init_code = Vec::new();
    for (i, &byte) in child_runtime.iter().enumerate() {
        init_code.push(opcode::PUSH1);
        init_code.push(byte);
        init_code.push(opcode::PUSH1);
        init_code.push(i as u8);
        init_code.push(opcode::MSTORE8);
    }
    init_code.push(opcode::PUSH1);
    init_code.push(child_runtime.len() as u8);
    init_code.push(opcode::PUSH1);
    init_code.push(0);
    init_code.push(opcode::RETURN);

    assert!(init_code.len() < 256, "init_code too large");

    // Build main bytecode: store init_code in memory, CREATE, then CALL the result.
    let mut bytecode = Vec::new();

    // Store init code in memory byte by byte.
    for (i, &byte) in init_code.iter().enumerate() {
        bytecode.push(opcode::PUSH1);
        bytecode.push(byte);
        bytecode.push(opcode::PUSH1);
        bytecode.push(i as u8);
        bytecode.push(opcode::MSTORE8);
    }

    // CREATE(value=0, offset=0, length=init_code.len())
    bytecode.push(opcode::PUSH1);
    bytecode.push(init_code.len() as u8);
    bytecode.push(opcode::PUSH1);
    bytecode.push(0);
    bytecode.push(opcode::PUSH1);
    bytecode.push(0);
    bytecode.push(opcode::CREATE);
    // Stack: [child_addr]

    // Push 5 zeros for CALL args (retSize, retOff, argsSize, argsOff, value)
    for _ in 0..5 {
        bytecode.push(opcode::PUSH1);
        bytecode.push(0);
    }
    // Stack: [child_addr, 0, 0, 0, 0, 0]

    // SWAP5: bring child_addr to top
    bytecode.push(opcode::SWAP5);
    // Stack: [0, 0, 0, 0, 0, child_addr]

    // GAS: push remaining gas
    bytecode.push(opcode::GAS);
    // Stack: [0, 0, 0, 0, 0, child_addr, gas]

    // CALL(gas, addr, value, argsOff, argsSize, retOff, retSize)
    bytecode.push(opcode::CALL);
    bytecode.push(opcode::POP);
    bytecode.push(opcode::STOP);

    Bytecode::new_legacy(bytecode.into())
}

/// Bytecode: GAS, PUSH1 0, MSTORE, PUSH1 32, PUSH1 0, RETURN
/// Executes GAS opcode, stores result in memory, returns it as 32-byte output.
fn gas_opcode_return_bytecode() -> Bytecode {
    Bytecode::new_legacy(
        [
            opcode::GAS,
            opcode::PUSH1,
            0,
            opcode::MSTORE,
            opcode::PUSH1,
            32,
            opcode::PUSH1,
            0,
            opcode::RETURN,
        ]
        .into(),
    )
}

/// Bytecode: SSTORE(0, 1); INVALID
/// One state gas charge then exceptional halt via 0xFE.
fn sstore_then_invalid_bytecode() -> Bytecode {
    Bytecode::new_legacy(
        [
            opcode::PUSH1,
            1,
            opcode::PUSH1,
            0,
            opcode::SSTORE,
            opcode::INVALID,
        ]
        .into(),
    )
}

/// Bytecode: CALL to identity precompile (0x04) with no args, then STOP.
#[allow(clippy::vec_init_then_push)]
fn call_precompile_bytecode() -> Bytecode {
    let mut bytecode = Vec::new();
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // retSize
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // retOffset
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // argsSize
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // argsOffset
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // value = 0
    bytecode.push(opcode::PUSH20);
    bytecode.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]);
    bytecode.push(opcode::GAS);
    bytecode.push(opcode::CALL);
    bytecode.push(opcode::POP);
    bytecode.push(opcode::STOP);
    Bytecode::new_legacy(bytecode.into())
}

// ---- Category 1: SSTORE State Gas ----

/// 1.1 SSTORE zero→non-zero charges sstore_set_state_gas.
#[test]
fn test_eip8037_sstore_new_slot() {
    let bytecode = sstore_bytecode(0, 1);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(baseline_result.is_success(), "Baseline should succeed");
    assert!(result.is_success(), "State gas variant should succeed");
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(
        delta, STATE_GAS_SSTORE_SET,
        "SSTORE new slot should add exactly {STATE_GAS_SSTORE_SET} state gas, got delta {delta}"
    );
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    assert_eq!(result.gas().state_gas_spent(), STATE_GAS_SSTORE_SET);
    assert_eq!(result.gas().inner_refunded(), 0);
    assert!(result.logs().is_empty());
    assert_eq!(
        result.gas().total_gas_spent() - baseline_result.gas().total_gas_spent(),
        STATE_GAS_SSTORE_SET
    );
    compare_or_save_eip8037_testdata(
        "test_eip8037_sstore_new_slot.json",
        &(baseline_result, result),
    );
}

/// 1.2 Two SSTOREs to same slot: only first charges state gas.
#[test]
fn test_eip8037_sstore_overwrite_no_state_gas() {
    let bytecode = sstore_overwrite_bytecode();

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(baseline_result.is_success());
    assert!(result.is_success());
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(
        delta, STATE_GAS_SSTORE_SET,
        "Only the first SSTORE (0->1) should charge state gas, got delta {delta}"
    );
    assert_eq!(result.gas().state_gas_spent(), STATE_GAS_SSTORE_SET);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    assert_eq!(result.gas().inner_refunded(), 0);
    assert!(result.logs().is_empty());
    compare_or_save_eip8037_testdata(
        "test_eip8037_sstore_overwrite_no_state_gas.json",
        &(baseline_result, result),
    );
}

/// 1.3 SSTORE zero→zero: no state gas.
#[test]
fn test_eip8037_sstore_zero_to_zero_no_state_gas() {
    let bytecode = sstore_bytecode(0, 0);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(baseline_result.is_success());
    assert!(result.is_success());
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, 0, "SSTORE zero→zero should add no state gas");
    assert_eq!(result.gas().state_gas_spent(), 0);
    assert_eq!(result.tx_gas_used(), baseline_gas);
    assert_eq!(result.gas().inner_refunded(), 0);
    assert_eq!(
        result.gas().total_gas_spent(),
        baseline_result.gas().total_gas_spent()
    );
    compare_or_save_eip8037_testdata(
        "test_eip8037_sstore_zero_to_zero_no_state_gas.json",
        &(baseline_result, result),
    );
}

/// 1.4 Three SSTOREs to different new slots: 3× sstore_set_state_gas.
#[test]
fn test_eip8037_sstore_multiple_new_slots() {
    let bytecode = sstore_multi_bytecode();

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(baseline_result.is_success());
    assert!(result.is_success());
    let expected = 3 * STATE_GAS_SSTORE_SET;
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(
        delta, expected,
        "3 new slots should add {expected} state gas, got delta {delta}"
    );
    assert_eq!(result.gas().state_gas_spent(), expected);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    assert_eq!(result.gas().inner_refunded(), 0);
    assert_eq!(
        result.gas().total_gas_spent() - baseline_result.gas().total_gas_spent(),
        expected
    );
    compare_or_save_eip8037_testdata(
        "test_eip8037_sstore_multiple_new_slots.json",
        &(baseline_result, result),
    );
}

// ---- Category 2: CREATE State Gas ----

/// 2.1 CREATE deploying 0-byte contract: new_account + create state gas.
#[test]
fn test_eip8037_create_empty_code() {
    let init = return_n_bytes_init_code(0);
    let bytecode = create_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let expected = STATE_GAS_CREATE;
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, expected);
    assert_eq!(result.gas().state_gas_spent(), expected);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    assert_eq!(result.tx_gas_used(), baseline_gas + expected);
    compare_or_save_eip8037_testdata(
        "test_eip8037_create_empty_code.json",
        &(baseline_result, result),
    );
}

/// 2.2 CREATE deploying 10-byte contract: new_account + create + code_deposit(10).
#[test]
fn test_eip8037_create_with_code() {
    let init = return_n_bytes_init_code(10);
    let bytecode = create_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let expected_state_gas = STATE_GAS_CREATE + STATE_GAS_CODE_DEPOSIT * 10;
    let expected_delta = expected_state_gas + hash_cost(10);
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, expected_delta);
    assert_eq!(result.gas().state_gas_spent(), expected_state_gas);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    assert_eq!(result.tx_gas_used(), baseline_gas + expected_delta);
    compare_or_save_eip8037_testdata(
        "test_eip8037_create_with_code.json",
        &(baseline_result, result),
    );
}

/// 2.3 CREATE with init code that does SSTORE + returns 1-byte code: all 4 state gas types.
#[test]
fn test_eip8037_create_with_sstore() {
    let init = init_code_sstore_and_return();
    let bytecode = create_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let expected_state_gas = STATE_GAS_CREATE + STATE_GAS_SSTORE_SET + STATE_GAS_CODE_DEPOSIT;
    let expected_delta = expected_state_gas + hash_cost(1);
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, expected_delta);
    assert_eq!(result.gas().state_gas_spent(), expected_state_gas);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    assert_eq!(result.gas().inner_refunded(), 0);
    compare_or_save_eip8037_testdata(
        "test_eip8037_create_with_sstore.json",
        &(baseline_result, result),
    );
}

/// 2.4 CREATE2 deploying 10-byte contract: same state gas as CREATE.
#[test]
fn test_eip8037_create2_with_code() {
    let init = return_n_bytes_init_code(10);
    let bytecode = create2_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();
    assert!(baseline_result.is_success());

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let expected_state_gas = STATE_GAS_CREATE + STATE_GAS_CODE_DEPOSIT * 10;
    let expected_delta = expected_state_gas + hash_cost(10);
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, expected_delta);
    assert_eq!(result.gas().state_gas_spent(), expected_state_gas);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    // CREATE2 uses more regular gas than CREATE (hashing) but same state gas
    let create_init = return_n_bytes_init_code(10);
    let mut create_evm = state_gas_evm(create_bytecode(&create_init), u64::MAX);
    let create_result = create_evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    assert_eq!(
        result.gas().state_gas_spent(),
        create_result.gas().state_gas_spent()
    );
    compare_or_save_eip8037_testdata(
        "test_eip8037_create2_with_code.json",
        &(baseline_result, result, create_result),
    );
}

/// 2.5 CREATE deploying code where enough regular gas but insufficient total for code deposit state gas.
#[test]
fn test_eip8037_create_code_deposit_state_gas_oog() {
    let init = return_n_bytes_init_code(100);
    let bytecode = create_bytecode(&init);

    // Find baseline gas usage with ample gas.
    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    assert!(baseline_result.is_success());
    let baseline_gas = baseline_result.tx_gas_used();

    // Baseline succeeds at tight limit.
    let tight_limit = baseline_gas + 1;
    let mut baseline_tight = baseline_evm(bytecode.clone());
    let baseline_tight_result = baseline_tight
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(tight_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    assert!(baseline_tight_result.is_success());

    // With state gas at same tight limit: OOG because state gas exceeds remaining.
    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(tight_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_halt());
    match &result {
        revm::context_interface::result::ExecutionResult::Halt { reason, .. } => {
            assert!(matches!(reason, HaltReason::OutOfGas(_)));
        }
        _ => panic!("Expected Halt variant"),
    }
    assert_eq!(result.tx_gas_used(), tight_limit);
    assert_eq!(baseline_tight_result.gas().state_gas_spent(), 0);
    compare_or_save_eip8037_testdata(
        "test_eip8037_create_code_deposit_state_gas_oog.json",
        &(baseline_result, baseline_tight_result, result),
    );
}

// ---- Category 3: CALL State Gas ----

/// 3.1 CALL with value to non-existent account: new_account_state_gas.
#[test]
fn test_eip8037_call_new_account() {
    let target = address!("0xd000000000000000000000000000000000000001");
    let bytecode = call_with_value_bytecode(target.into_array(), U256::from(1));

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    assert_eq!(result.gas().state_gas_spent(), STATE_GAS_NEW_ACCOUNT);
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, STATE_GAS_NEW_ACCOUNT);
    assert_eq!(
        result.gas().total_gas_spent(),
        baseline_result.gas().total_gas_spent() + STATE_GAS_NEW_ACCOUNT
    );
    compare_or_save_eip8037_testdata(
        "test_eip8037_call_new_account.json",
        &(baseline_result, result),
    );
}

/// 3.2 CALL with value to existing account: no state gas.
#[test]
fn test_eip8037_call_existing_account() {
    let bytecode = call_with_value_bytecode(BENCH_CALLER.into_array(), U256::from(1));

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    assert_eq!(result.gas().state_gas_spent(), 0);
    assert_eq!(result.tx_gas_used(), baseline_gas);
    assert_eq!(
        result.gas().total_gas_spent(),
        baseline_result.gas().total_gas_spent()
    );
    compare_or_save_eip8037_testdata(
        "test_eip8037_call_existing_account.json",
        &(baseline_result, result),
    );
}

/// 3.3 SELFDESTRUCT sending balance to non-existent account: new_account_state_gas.
#[test]
fn test_eip8037_selfdestruct_new_account() {
    let beneficiary = address!("0xd000000000000000000000000000000000000002");
    let bytecode = selfdestruct_bytecode(beneficiary.into_array());

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    assert_eq!(result.gas().state_gas_spent(), STATE_GAS_NEW_ACCOUNT);
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, STATE_GAS_NEW_ACCOUNT);
    assert_eq!(
        result.gas().total_gas_spent(),
        baseline_result.gas().total_gas_spent() + STATE_GAS_NEW_ACCOUNT
    );
    compare_or_save_eip8037_testdata(
        "test_eip8037_selfdestruct_new_account.json",
        &(baseline_result, result),
    );
}

/// 3.4 SELFDESTRUCT sending balance to existing account: no state gas.
#[test]
fn test_eip8037_selfdestruct_existing_account() {
    let bytecode = selfdestruct_bytecode(BENCH_CALLER.into_array());

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    assert_eq!(result.gas().state_gas_spent(), 0);
    assert_eq!(result.tx_gas_used(), baseline_gas);
    assert_eq!(
        result.gas().total_gas_spent(),
        baseline_result.gas().total_gas_spent()
    );
    assert_eq!(result.tx_gas_used(), baseline_gas);
    compare_or_save_eip8037_testdata(
        "test_eip8037_selfdestruct_existing_account.json",
        &(baseline_result, result),
    );
}

// ---- Category 4: Regular Gas Cap Enforcement ----

/// 4.1 Tight regular gas cap causes OOG.
#[test]
fn test_eip8037_regular_gas_cap_causes_oog() {
    let bytecode = sstore_bytecode(0, 1);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(100_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    assert!(baseline_result.is_success());

    let mut evm = state_gas_evm(bytecode, 30_000);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(30_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_halt());
    match &result {
        revm::context_interface::result::ExecutionResult::Halt { reason, .. } => {
            assert!(matches!(reason, HaltReason::OutOfGas(_)));
        }
        _ => panic!("Expected Halt variant"),
    }
    assert_eq!(result.tx_gas_used(), 30_000);
    assert_eq!(result.gas().total_gas_spent(), 30_000);
    compare_or_save_eip8037_testdata(
        "test_eip8037_regular_gas_cap_causes_oog.json",
        &(baseline_result, result),
    );
}

/// 4.2 Adequate regular gas cap: success.
#[test]
fn test_eip8037_regular_gas_cap_sufficient() {
    let bytecode = sstore_bytecode(0, 1);

    let gas_limit = 500_000u64;

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();
    assert!(baseline_result.is_success());

    let mut evm = state_gas_evm(bytecode, gas_limit);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success());
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, STATE_GAS_SSTORE_SET);
    assert_eq!(result.gas().state_gas_spent(), STATE_GAS_SSTORE_SET);
    assert_eq!(result.tx_gas_used(), baseline_gas + STATE_GAS_SSTORE_SET);
    compare_or_save_eip8037_testdata(
        "test_eip8037_regular_gas_cap_sufficient.json",
        &(baseline_result, result),
    );
}

/// 4.3 Remaining gas insufficient after state gas deduction.
#[test]
fn test_eip8037_state_gas_oog_remaining() {
    let bytecode = sstore_bytecode(0, 1);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(50_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    assert!(baseline_result.is_success());
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(50_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_halt());
    match &result {
        revm::context_interface::result::ExecutionResult::Halt { reason, .. } => {
            assert!(matches!(reason, HaltReason::OutOfGas(_)));
        }
        _ => panic!("Expected Halt variant"),
    }
    assert_eq!(result.tx_gas_used(), 50_000);
    assert!(baseline_gas < 50_000);
    assert!(baseline_gas + STATE_GAS_SSTORE_SET > 50_000);
    compare_or_save_eip8037_testdata(
        "test_eip8037_state_gas_oog_remaining.json",
        &(baseline_result, result),
    );
}

/// 4.4 tx_gas_limit_cap is NOT enforced as a hard cap when state gas reservoir covers it.
#[test]
fn test_eip8037_tx_limit_cap_not_enforced_with_state_gas() {
    let bytecode = sstore_bytecode(0, 1);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(500_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    assert!(baseline_result.is_success());
    let baseline_gas = baseline_result.tx_gas_used();

    // gas_limit=500k, cap=50k: reservoir covers state gas.
    let mut evm = state_gas_evm(bytecode, 50_000);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(500_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success());
    assert!(result.tx_gas_used() > 50_000, "gas_used exceeds cap");
    assert_eq!(result.gas().state_gas_spent(), STATE_GAS_SSTORE_SET);
    let delta = result.tx_gas_used() - baseline_gas;
    // EIP-8037: tx_gas_used = tx.gas - gas_left - state_gas_left
    // Unused reservoir gas (including new account state gas that wasn't consumed)
    // is not counted as gas used.
    assert_eq!(delta, STATE_GAS_SSTORE_SET);
    compare_or_save_eip8037_testdata(
        "test_eip8037_tx_limit_cap_not_enforced_with_state_gas.json",
        &(baseline_result, result),
    );
}

/// 4.5 Block gas limit is still enforced even with state gas enabled.
#[test]
fn test_eip8037_block_gas_limit_enforced_with_state_gas() {
    use revm::context_interface::result::{EVMError, InvalidTransaction};

    let bytecode = sstore_bytecode(0, 1);

    // tx gas_limit(100k) > block gas_limit(10k) -> validation error.
    let mut evm = Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.tx_gas_limit_cap = Some(u64::MAX);
            cfg.gas_params.override_gas([
                (GasId::sstore_set_state_gas(), STATE_GAS_SSTORE_SET),
                (GasId::new_account_state_gas(), STATE_GAS_NEW_ACCOUNT),
                (GasId::code_deposit_state_gas(), STATE_GAS_CODE_DEPOSIT),
                (GasId::create_state_gas(), STATE_GAS_CREATE),
            ]);
        })
        .modify_block_chained(|block| {
            block.gas_limit = 10_000;
        })
        .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
        .build_mainnet();

    let err = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(100_000)
                .gas_price(0)
                .build_fill(),
        )
        .expect_err("Expected validation error when tx gas_limit exceeds block gas_limit");

    assert!(matches!(&err, EVMError::Transaction(_)));
    match &err {
        EVMError::Transaction(InvalidTransaction::CallerGasLimitMoreThanBlock) => {}
        other => panic!("Expected CallerGasLimitMoreThanBlock, got {other:?}"),
    }

    // Also verify without state gas - same error.
    let mut evm_no_state = Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.tx_gas_limit_cap = Some(u64::MAX);
        })
        .modify_block_chained(|block| {
            block.gas_limit = 10_000;
        })
        .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
        .build_mainnet();

    let err_no_state = evm_no_state
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(100_000)
                .gas_price(0)
                .build_fill(),
        )
        .expect_err("Expected validation error without state gas too");

    match &err_no_state {
        EVMError::Transaction(InvalidTransaction::CallerGasLimitMoreThanBlock) => {}
        other => panic!("Expected CallerGasLimitMoreThanBlock, got {other:?}"),
    }

    // Tx fitting within block limit should succeed.
    let mut evm_fits = state_gas_evm(bytecode, u64::MAX);
    let result_fits = evm_fits
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    assert!(result_fits.is_success());
    compare_or_save_eip8037_testdata(
        "test_eip8037_block_gas_limit_enforced_with_state_gas.json",
        &result_fits,
    );
}

// ---- Category 5: State Gas Propagation ----

/// 5.1 CREATE child's state gas propagates to parent on success.
#[test]
fn test_eip8037_create_child_propagates() {
    let init = init_code_sstore_and_return();
    let bytecode = create_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    let expected_state_gas = STATE_GAS_CREATE + STATE_GAS_SSTORE_SET + STATE_GAS_CODE_DEPOSIT;
    let expected_delta = expected_state_gas + hash_cost(1);

    assert!(result.is_success());
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, expected_delta);
    assert_eq!(result.gas().state_gas_spent(), expected_state_gas);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    compare_or_save_eip8037_testdata(
        "test_eip8037_create_child_propagates.json",
        &(baseline_result, result),
    );
}

/// 5.2 Reverted CREATE: child's SSTORE state gas is refunded on revert.
#[test]
fn test_eip8037_reverted_create_child() {
    let init = init_code_sstore_and_revert();
    let bytecode = create_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    // On child revert, state gas is returned to parent's reservoir (matching Python spec).
    // Only CREATE state gas contributes to the delta (SSTORE state gas is refunded).
    let expected_delta = STATE_GAS_CREATE;
    let parent_state_gas = STATE_GAS_CREATE;

    assert!(result.is_success());
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, expected_delta);
    // state_gas_spent reflects only parent's state gas (child's SSTORE state gas refunded on revert).
    assert_eq!(result.gas().state_gas_spent(), parent_state_gas);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    compare_or_save_eip8037_testdata(
        "test_eip8037_reverted_create_child.json",
        &(baseline_result, result),
    );
}

/// 5.3 CALL to contract that does SSTORE(0,1). Child's state_gas_spent propagates on success.
#[test]
fn test_eip8037_call_child_sstore_propagates() {
    let child_runtime: Vec<u8> = vec![
        opcode::PUSH1,
        0x01,
        opcode::PUSH1,
        0x00,
        opcode::SSTORE,
        opcode::STOP,
    ];
    let bytecode = call_created_child_bytecode(&child_runtime);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    let code_deposit_gas = STATE_GAS_CODE_DEPOSIT * child_runtime.len() as u64;
    let expected_state_gas = STATE_GAS_CREATE + code_deposit_gas + STATE_GAS_SSTORE_SET;
    let expected_delta = expected_state_gas + hash_cost(child_runtime.len());

    assert!(result.is_success());
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, expected_delta);
    assert_eq!(result.gas().state_gas_spent(), expected_state_gas);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    compare_or_save_eip8037_testdata(
        "test_eip8037_call_child_sstore_propagates.json",
        &(baseline_result, result),
    );
}

/// 5.4 CALL to contract that does SSTORE(0,1) then REVERT. Child's state gas is refunded.
#[test]
fn test_eip8037_call_child_sstore_reverts() {
    let child_runtime: Vec<u8> = vec![
        opcode::PUSH1,
        0x01,
        opcode::PUSH1,
        0x00,
        opcode::SSTORE,
        opcode::PUSH1,
        0x00,
        opcode::PUSH1,
        0x00,
        opcode::REVERT,
    ];
    let bytecode = call_created_child_bytecode(&child_runtime);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap()
        .result;
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap()
        .result;

    let code_deposit_gas = STATE_GAS_CODE_DEPOSIT * child_runtime.len() as u64;
    let create_state_gas = STATE_GAS_CREATE + code_deposit_gas;

    assert!(result.is_success());
    // state_gas_spent reflects only CREATE costs (child SSTORE refunded on revert).
    assert_eq!(result.gas().state_gas_spent(), create_state_gas);
    // On child revert, state changes are rolled back and state gas is returned
    // to the parent's reservoir (matching Python spec's incorporate_child_on_error).
    // So only CREATE state gas and hash cost contribute to the delta.
    let expected_delta = create_state_gas + hash_cost(child_runtime.len());
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, expected_delta);
    compare_or_save_eip8037_testdata(
        "test_eip8037_call_child_sstore_reverts.json",
        &(baseline_result, result),
    );
}

/// 5.5 Multi-level nesting: CALL -> CREATE -> SSTORE. State gas propagates through frames.
#[test]
fn test_eip8037_nested_call_create_sstore() {
    let child_runtime: Vec<u8> = vec![
        opcode::PUSH1,
        0x01,
        opcode::PUSH1,
        0x00,
        opcode::SSTORE,
        opcode::STOP,
    ];
    let bytecode = call_created_child_bytecode(&child_runtime);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    let code_deposit_gas = STATE_GAS_CODE_DEPOSIT * child_runtime.len() as u64;
    let expected_state_gas = STATE_GAS_CREATE + code_deposit_gas + STATE_GAS_SSTORE_SET;
    let expected_delta = expected_state_gas + hash_cost(child_runtime.len());

    assert!(result.is_success());
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, expected_delta);
    assert_eq!(result.gas().state_gas_spent(), expected_state_gas);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);

    // Cross-validate: CREATE-only run (no CALL child SSTORE).
    let init = return_n_bytes_init_code(child_runtime.len() as u8);
    let mut create_evm = state_gas_evm(create_bytecode(&init), u64::MAX);
    let create_result = create_evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    assert!(create_result.is_success());
    let sstore_portion = result.gas().state_gas_spent() - create_result.gas().state_gas_spent();
    assert_eq!(sstore_portion, STATE_GAS_SSTORE_SET);
    compare_or_save_eip8037_testdata(
        "test_eip8037_nested_call_create_sstore.json",
        &(baseline_result, result, create_result),
    );
}

// ---- Category 6: Interactions ----

/// 6.1 SSTORE 0→1 (state gas), then 1→0 (refund). Refund does NOT undo state gas.
#[test]
fn test_eip8037_sstore_set_then_clear_refund() {
    let bytecode = sstore_set_then_clear_bytecode();

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    // State gas increases spent by exactly STATE_GAS_SSTORE_SET.
    assert_eq!(result.gas().state_gas_spent(), STATE_GAS_SSTORE_SET);
    let spent_delta = result.gas().total_gas_spent() - baseline_result.gas().total_gas_spent();
    assert_eq!(spent_delta, STATE_GAS_SSTORE_SET);
    // Refund does NOT undo state gas — gas_used is higher than baseline.
    assert!(result.tx_gas_used() > baseline_gas);
    assert!(result.gas().total_gas_spent() > baseline_result.gas().total_gas_spent());
    compare_or_save_eip8037_testdata(
        "test_eip8037_sstore_set_then_clear_refund.json",
        &(baseline_result, result),
    );
}

/// 6.2 State gas does not reduce regular gas budget.
#[test]
fn test_eip8037_state_gas_does_not_reduce_regular_gas() {
    let bytecode = sstore_bytecode(0, 1);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    assert!(baseline_result.is_success());
    let baseline_gas = baseline_result.tx_gas_used();

    let tight_cap = baseline_gas + STATE_GAS_SSTORE_SET + 1;
    let mut evm = state_gas_evm(bytecode, tight_cap);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(tight_cap)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success());
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, STATE_GAS_SSTORE_SET);
    assert_eq!(result.gas().state_gas_spent(), STATE_GAS_SSTORE_SET);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    compare_or_save_eip8037_testdata(
        "test_eip8037_state_gas_does_not_reduce_regular_gas.json",
        &(baseline_result, result),
    );
}

/// 6.3 GAS opcode returns remaining (excludes reservoir).
#[test]
fn test_eip8037_gas_opcode_excludes_reservoir() {
    let bytecode = gas_opcode_return_bytecode();
    let gas_limit: u64 = 100_000;

    // Baseline: no state gas, no reservoir.
    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    assert!(baseline_result.is_success());
    let baseline_output = baseline_result.output().unwrap();
    let baseline_gas_value = U256::from_be_slice(baseline_output.as_ref());

    // With state gas: cap=50k creates a reservoir.
    let cap = 50_000u64;
    let mut evm = state_gas_evm(bytecode, cap);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success());
    let output = result.output().unwrap();
    assert_eq!(output.len(), 32);
    let gas_opcode_value = U256::from_be_slice(output.as_ref());
    let intrinsic = 21_000u64; // Standard intrinsic gas
    let execution_gas = gas_limit - intrinsic;
    assert!(gas_opcode_value < U256::from(execution_gas));
    // GAS opcode with state gas should be less than baseline (reservoir excluded).
    assert!(gas_opcode_value < baseline_gas_value);
    // Difference should equal reservoir size.
    let regular_budget = cap - intrinsic;
    let expected_reservoir = execution_gas - regular_budget;
    let gas_diff = baseline_gas_value - gas_opcode_value;
    assert_eq!(gas_diff, U256::from(expected_reservoir));
    compare_or_save_eip8037_testdata(
        "test_eip8037_gas_opcode_excludes_reservoir.json",
        &(baseline_result, result),
    );
}

/// 6.4 INVALID opcode after SSTORE: spend_all() zeroes remaining but preserves reservoir.
///
/// On HALT, state gas spent is restored to the reservoir (state changes rolled back),
/// so tx_gas_used is reduced by the state gas amount compared to the baseline.
#[test]
fn test_eip8037_spend_all_preserves_reservoir() {
    let bytecode = sstore_then_invalid_bytecode();
    let gas_limit: u64 = 500_000;

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(baseline_result.is_halt());
    assert_eq!(baseline_result.tx_gas_used(), gas_limit);

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_halt());
    match &result {
        revm::context_interface::result::ExecutionResult::Halt { reason, .. } => {
            assert!(matches!(reason, HaltReason::InvalidFEOpcode));
        }
        _ => panic!("Expected Halt variant"),
    }
    // On halt, state gas is refunded via reservoir, so tx_gas_used is reduced.
    assert_eq!(
        result.tx_gas_used(),
        gas_limit - STATE_GAS_SSTORE_SET,
        "Halt refunds state gas via reservoir"
    );
    // state_gas_spent is zeroed on halt (state changes rolled back).
    assert_eq!(result.gas().state_gas_spent(), 0);
    compare_or_save_eip8037_testdata(
        "test_eip8037_spend_all_preserves_reservoir.json",
        &(baseline_result, result),
    );
}

/// 6.5 state_gas_spent field in ResultGas.
#[test]
fn test_eip8037_state_gas_spent_in_result() {
    let bytecode = sstore_bytecode(0, 1);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(baseline_result.is_success());
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    assert_eq!(result.gas().state_gas_spent(), STATE_GAS_SSTORE_SET);
    let spent_delta = result.gas().total_gas_spent() - baseline_result.gas().total_gas_spent();
    assert_eq!(spent_delta, STATE_GAS_SSTORE_SET);
    let gas_used_delta = result.tx_gas_used() - baseline_result.tx_gas_used();
    assert_eq!(gas_used_delta, STATE_GAS_SSTORE_SET);
    compare_or_save_eip8037_testdata(
        "test_eip8037_state_gas_spent_in_result.json",
        &(baseline_result, result),
    );
}

/// 6.6 CALL to precompile: precompile gas is regular, not state gas.
#[test]
fn test_eip8037_precompile_no_state_gas() {
    let bytecode = call_precompile_bytecode();

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();
    assert!(baseline_result.is_success());

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, 0);
    assert_eq!(result.gas().state_gas_spent(), 0);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    assert_eq!(
        result.gas().total_gas_spent(),
        baseline_result.gas().total_gas_spent()
    );
    compare_or_save_eip8037_testdata(
        "test_eip8037_precompile_no_state_gas.json",
        &(baseline_result, result),
    );
}

// ---- Category 7: Reservoir Refill ----
//
// The reservoir refill logic (handler_reservoir_refill) is invoked on HALT or REVERT:
// `new_reservoir = reservoir + max(0, state_gas_spent - reservoir)`
//
// This accounts for state gas that had to be drawn from regular gas.
// On OK: reservoir stays unchanged (no refill needed).
// On REVERT: remaining is reimbursed, then refill applied.
// On HALT: remaining is NOT reimbursed, refill applied to final gas accounting.

/// 7.1 REVERT with state_gas < reservoir.
///
/// On REVERT, state gas is restored to the reservoir (state changes rolled back),
/// so tx_gas_used matches the baseline (no extra state gas charged).
#[test]
fn test_eip8037_reservoir_refill_revert_state_gas_less() {
    let bytecode = sstore_then_revert_bytecode();
    let gas_limit = 500_000u64;

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(!result.is_success() && !result.is_halt(), "Expected REVERT");
    // On revert, state gas is refunded via reservoir, so no delta vs baseline.
    assert_eq!(result.tx_gas_used(), baseline_gas);
    // state_gas_spent is zeroed on revert (state changes rolled back).
    assert_eq!(result.gas().state_gas_spent(), 0);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    assert!(
        result.tx_gas_used() < gas_limit,
        "REVERT reimburses remaining"
    );
    assert_eq!(result.gas().inner_refunded(), 0);
    assert!(result.logs().is_empty());
    compare_or_save_eip8037_testdata(
        "test_eip8037_reservoir_refill_revert_state_gas_less.json",
        &(baseline_result, result),
    );
}

/// 7.2 REVERT with state_gas > reservoir (2x SSTORE).
///
/// On REVERT, all state gas is restored to the reservoir (state changes rolled back),
/// so tx_gas_used matches the baseline (no extra state gas charged).
#[test]
fn test_eip8037_reservoir_refill_revert_state_gas_more() {
    let bytecode = sstore_multi_then_revert_bytecode();
    let gas_limit = 1_000_000u64;

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(!result.is_success() && !result.is_halt(), "Expected REVERT");
    // On revert, state gas is refunded via reservoir, so no delta vs baseline.
    assert_eq!(result.tx_gas_used(), baseline_gas);
    // state_gas_spent is zeroed on revert (state changes rolled back).
    assert_eq!(result.gas().state_gas_spent(), 0);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    assert!(result.tx_gas_used() < gas_limit);
    assert_eq!(result.gas().inner_refunded(), 0);
    assert!(result.logs().is_empty());
    compare_or_save_eip8037_testdata(
        "test_eip8037_reservoir_refill_revert_state_gas_more.json",
        &(baseline_result, result),
    );
}

/// 7.3 HALT (OOG) with tight gas limit.
#[test]
fn test_eip8037_reservoir_refill_halt_state_gas_less() {
    let bytecode = sstore_bytecode(0, 1);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(100_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    assert!(baseline_result.is_success());

    let gas_limit = 25_000u64;
    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_halt());
    match &result {
        revm::context_interface::result::ExecutionResult::Halt { reason, .. } => {
            assert!(matches!(reason, HaltReason::OutOfGas(_)));
        }
        _ => panic!("Expected Halt variant"),
    }
    assert_eq!(result.tx_gas_used(), gas_limit);
    assert_eq!(result.gas().total_gas_spent(), gas_limit);
    assert_eq!(result.gas().inner_refunded(), 0);
    assert!(result.logs().is_empty());
    compare_or_save_eip8037_testdata(
        "test_eip8037_reservoir_refill_halt_state_gas_less.json",
        &(baseline_result, result),
    );
}

/// 7.4 HALT (OOG) with tight cap forcing regular gas exhaustion.
#[test]
fn test_eip8037_reservoir_refill_halt_state_gas_more() {
    let bytecode = sstore_multi_bytecode();
    let gas_limit = 100_000u64;

    let mut evm = state_gas_evm(bytecode, 30_000);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_halt());
    match &result {
        revm::context_interface::result::ExecutionResult::Halt { reason, .. } => {
            assert!(matches!(reason, HaltReason::OutOfGas(_)));
        }
        _ => panic!("Expected Halt variant"),
    }
    // EIP-8037: tx_gas_used = tx.gas - gas_left - state_gas_left
    // On halt, gas_left=0 but state_gas_left (reservoir) may be non-zero.
    // Unused reservoir gas is refunded, so tx_gas_used < gas_limit.
    assert!(result.tx_gas_used() < gas_limit);
    assert_eq!(result.gas().inner_refunded(), 0);
    assert!(result.logs().is_empty());
    compare_or_save_eip8037_testdata(
        "test_eip8037_reservoir_refill_halt_state_gas_more.json",
        &result,
    );
}

/// 7.5 HALT vs REVERT: gas accounting difference.
#[test]
fn test_eip8037_reservoir_refill_halt_vs_revert_difference() {
    let bytecode = sstore_then_revert_bytecode();

    // HALT path: tight gas limit causes OOG.
    let halt_gas_limit = 25_000u64;
    let mut evm_halt = state_gas_evm(bytecode.clone(), u64::MAX);
    let result_halt = evm_halt
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(halt_gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    // REVERT path: ample gas allows full execution.
    let revert_gas_limit = 500_000u64;
    let mut evm_revert = state_gas_evm(bytecode, u64::MAX);
    let result_revert = evm_revert
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(revert_gas_limit)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result_halt.is_halt());
    assert!(!result_revert.is_success() && !result_revert.is_halt());
    // On halt with tight gas, tx_gas_used = halt_gas_limit (no state gas to refund
    // since OOG happened before SSTORE could execute).
    assert_eq!(result_halt.tx_gas_used(), halt_gas_limit);
    assert!(result_revert.tx_gas_used() < revert_gas_limit);
    // On revert/halt, state_gas_spent is zeroed (state changes rolled back).
    assert_eq!(result_revert.gas().state_gas_spent(), 0);
    assert_eq!(result_halt.gas().state_gas_spent(), 0);
    assert_eq!(result_halt.gas().inner_refunded(), 0);
    assert_eq!(result_revert.gas().inner_refunded(), 0);
    compare_or_save_eip8037_testdata(
        "test_eip8037_reservoir_refill_halt_vs_revert_difference.json",
        &(result_halt, result_revert),
    );
}

// ---- Gap Tests: Categories A-E from eip8037.md ----

/// Bytecode: CALL(gas, addr, value=0, 0, 0, 0, 0); POP; STOP
/// CALL without value transfer to given address.
#[allow(clippy::vec_init_then_push)]
fn call_no_value_bytecode(target: [u8; 20]) -> Bytecode {
    let mut bytecode = Vec::new();
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // retSize
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // retOffset
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // argsSize
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // argsOffset
    bytecode.push(opcode::PUSH1);
    bytecode.push(0); // value = 0
    bytecode.push(opcode::PUSH20);
    bytecode.extend_from_slice(&target);
    bytecode.push(opcode::GAS);
    bytecode.push(opcode::CALL);
    bytecode.push(opcode::POP);
    bytecode.push(opcode::STOP);
    Bytecode::new_legacy(bytecode.into())
}

/// Bytecode that CREATEs a child, CALLs it (child reverts), then does SSTORE(0,1) itself.
/// Tests that child's reverted state gas doesn't affect parent's own state gas accounting.
fn create_call_revert_then_sstore_bytecode(child_runtime: &[u8]) -> Bytecode {
    assert!(child_runtime.len() < 128, "child_runtime too large");

    // Build init code for child deployment.
    let mut init_code = Vec::new();
    for (i, &byte) in child_runtime.iter().enumerate() {
        init_code.push(opcode::PUSH1);
        init_code.push(byte);
        init_code.push(opcode::PUSH1);
        init_code.push(i as u8);
        init_code.push(opcode::MSTORE8);
    }
    init_code.push(opcode::PUSH1);
    init_code.push(child_runtime.len() as u8);
    init_code.push(opcode::PUSH1);
    init_code.push(0);
    init_code.push(opcode::RETURN);

    assert!(init_code.len() < 256, "init_code too large");

    let mut bytecode = Vec::new();

    // Store init code in memory.
    for (i, &byte) in init_code.iter().enumerate() {
        bytecode.push(opcode::PUSH1);
        bytecode.push(byte);
        bytecode.push(opcode::PUSH1);
        bytecode.push(i as u8);
        bytecode.push(opcode::MSTORE8);
    }

    // CREATE(value=0, offset=0, length)
    bytecode.push(opcode::PUSH1);
    bytecode.push(init_code.len() as u8);
    bytecode.push(opcode::PUSH1);
    bytecode.push(0);
    bytecode.push(opcode::PUSH1);
    bytecode.push(0);
    bytecode.push(opcode::CREATE);
    // Stack: [child_addr]

    // CALL child (value=0) — child will SSTORE then REVERT
    for _ in 0..5 {
        bytecode.push(opcode::PUSH1);
        bytecode.push(0);
    }
    bytecode.push(opcode::SWAP5);
    bytecode.push(opcode::GAS);
    bytecode.push(opcode::CALL);
    bytecode.push(opcode::POP); // discard call result

    // Parent does SSTORE(0, 1) — this state gas should be kept
    bytecode.push(opcode::PUSH1);
    bytecode.push(1);
    bytecode.push(opcode::PUSH1);
    bytecode.push(0);
    bytecode.push(opcode::SSTORE);

    bytecode.push(opcode::STOP);
    Bytecode::new_legacy(bytecode.into())
}

/// C.1/E.5: CALL to new empty account WITHOUT value transfer — no state gas.
#[test]
fn test_eip8037_call_new_account_no_value() {
    let target = address!("0xd000000000000000000000000000000000000099");
    let bytecode = call_no_value_bytecode(target.into_array());

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    // No value transfer → no new_account_state_gas even for empty account.
    assert_eq!(result.gas().state_gas_spent(), 0);
    assert_eq!(result.tx_gas_used(), baseline_gas);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    compare_or_save_eip8037_testdata(
        "test_eip8037_call_new_account_no_value.json",
        &(baseline_result, result),
    );
}

/// E.3: Large code deployment — CREATE deploying 200-byte contract.
/// Verifies code_deposit_state_gas scales correctly with code size.
#[test]
fn test_eip8037_create_large_code() {
    let init = return_n_bytes_init_code(200);
    let bytecode = create_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let expected_state_gas = STATE_GAS_CREATE + STATE_GAS_CODE_DEPOSIT * 200;
    let expected_delta = expected_state_gas + hash_cost(200);
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, expected_delta);
    assert_eq!(result.gas().state_gas_spent(), expected_state_gas);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    // Verify code deposit portion is significant
    let code_deposit_portion = STATE_GAS_CODE_DEPOSIT * 200;
    assert_eq!(code_deposit_portion, 200_000);
    assert_eq!(result.tx_gas_used(), baseline_gas + expected_delta);
    compare_or_save_eip8037_testdata(
        "test_eip8037_create_large_code.json",
        &(baseline_result, result),
    );
}

/// B.2: Parent CREATEs child, CALLs child (child SSTORE + REVERT), then parent SSTORE.
/// Verifies child's reverted state gas is refunded while parent's own state gas accumulates.
#[test]
fn test_eip8037_parent_sstore_after_child_revert() {
    // Child runtime: SSTORE(0,1); REVERT(0,0)
    let child_runtime: Vec<u8> = vec![
        opcode::PUSH1,
        0x01,
        opcode::PUSH1,
        0x00,
        opcode::SSTORE,
        opcode::PUSH1,
        0x00,
        opcode::PUSH1,
        0x00,
        opcode::REVERT,
    ];
    let bytecode = create_call_revert_then_sstore_bytecode(&child_runtime);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.tx_gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    let code_deposit_gas = STATE_GAS_CODE_DEPOSIT * child_runtime.len() as u64;
    // Parent's CREATE state gas + code deposit + parent's own SSTORE.
    // Child's SSTORE state gas (200k) is refunded on revert.
    let expected_state_gas = STATE_GAS_CREATE + code_deposit_gas + STATE_GAS_SSTORE_SET;
    let expected_delta = expected_state_gas + hash_cost(child_runtime.len());

    assert!(result.is_success());
    let delta = result.tx_gas_used() - baseline_gas;
    assert_eq!(delta, expected_delta);
    assert_eq!(result.gas().state_gas_spent(), expected_state_gas);
    assert_eq!(baseline_result.gas().state_gas_spent(), 0);
    compare_or_save_eip8037_testdata(
        "test_eip8037_parent_sstore_after_child_revert.json",
        &(baseline_result, result),
    );
}
