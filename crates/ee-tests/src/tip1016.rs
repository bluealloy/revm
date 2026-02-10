//! TIP-1016 State Gas integration tests.
//!
//! Verifies dual-limit gas accounting where storage creation gas (state gas)
//! is tracked separately from regular gas.

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

/// State gas costs used across all TIP-1016 tests.
const STATE_GAS_SSTORE_SET: u64 = 20_000;
const STATE_GAS_NEW_ACCOUNT: u64 = 25_000;
const STATE_GAS_CODE_DEPOSIT: u64 = 200; // per byte
const STATE_GAS_CREATE: u64 = 32_000;

type MainEvm = MainnetEvm<MainnetContext<BenchmarkDB>>;

/// Builds an EVM with state gas enabled and custom gas params.
fn state_gas_evm(bytecode: Bytecode, cap: u64) -> MainEvm {
    Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.enable_state_gas = true;
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

/// Init code that does SSTORE(0, 1) and then REVERTs.
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

// ---- Category 1: SSTORE State Gas ----

/// 1.1 SSTORE zero→non-zero charges sstore_set_state_gas.
#[test]
fn test_tip1016_sstore_new_slot() {
    let bytecode = sstore_bytecode(0, 1);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(
        delta, STATE_GAS_SSTORE_SET,
        "SSTORE new slot should add exactly {STATE_GAS_SSTORE_SET} state gas, got delta {delta}"
    );
}

/// 1.2 Two SSTOREs to same slot: only first charges state gas.
#[test]
fn test_tip1016_sstore_overwrite_no_state_gas() {
    let bytecode = sstore_overwrite_bytecode();

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(
        delta, STATE_GAS_SSTORE_SET,
        "Only the first SSTORE (0->1) should charge state gas, got delta {delta}"
    );
}

/// 1.3 SSTORE zero→zero: no state gas.
#[test]
fn test_tip1016_sstore_zero_to_zero_no_state_gas() {
    let bytecode = sstore_bytecode(0, 0);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(
        delta, 0,
        "SSTORE zero→zero should add no state gas, got delta {delta}"
    );
}

/// 1.4 Three SSTOREs to different new slots: 3× sstore_set_state_gas.
#[test]
fn test_tip1016_sstore_multiple_new_slots() {
    let bytecode = sstore_multi_bytecode();

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let expected = 3 * STATE_GAS_SSTORE_SET;
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(
        delta, expected,
        "3 new slots should add {expected} state gas, got delta {delta}"
    );
}

// ---- Category 2: CREATE State Gas ----

/// 2.1 CREATE deploying 0-byte contract: new_account + create state gas.
#[test]
fn test_tip1016_create_empty_code() {
    let init = return_n_bytes_init_code(0);
    let bytecode = create_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let expected = STATE_GAS_NEW_ACCOUNT + STATE_GAS_CREATE;
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(
        delta, expected,
        "CREATE empty code should add {expected} state gas (25k+32k), got delta {delta}"
    );
}

/// 2.2 CREATE deploying 10-byte contract: new_account + create + code_deposit(10).
#[test]
fn test_tip1016_create_with_code() {
    let init = return_n_bytes_init_code(10);
    let bytecode = create_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let expected = STATE_GAS_NEW_ACCOUNT + STATE_GAS_CREATE + STATE_GAS_CODE_DEPOSIT * 10;
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(
        delta, expected,
        "CREATE with 10-byte code should add {expected} state gas (25k+32k+2k), got delta {delta}"
    );
}

/// 2.3 CREATE with init code that does SSTORE + returns 1-byte code: all 4 state gas types.
#[test]
fn test_tip1016_create_with_sstore() {
    let init = init_code_sstore_and_return();
    let bytecode = create_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let expected =
        STATE_GAS_NEW_ACCOUNT + STATE_GAS_CREATE + STATE_GAS_SSTORE_SET + STATE_GAS_CODE_DEPOSIT; // 1 byte
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(
        delta, expected,
        "CREATE+SSTORE+1byte code should add {expected} state gas, got delta {delta}"
    );
}

// ---- Category 3: CALL State Gas ----

/// 3.1 CALL with value to non-existent account: new_account_state_gas.
#[test]
fn test_tip1016_call_new_account() {
    let target = address!("0xd000000000000000000000000000000000000001");
    let bytecode = call_with_value_bytecode(target.into_array(), U256::from(1));

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(
        delta, STATE_GAS_NEW_ACCOUNT,
        "CALL to new account should add {STATE_GAS_NEW_ACCOUNT} state gas, got delta {delta}"
    );
}

/// 3.2 CALL with value to existing account: no state gas.
#[test]
fn test_tip1016_call_existing_account() {
    let bytecode = call_with_value_bytecode(BENCH_CALLER.into_array(), U256::from(1));

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(
        delta, 0,
        "CALL to existing account should add no state gas, got delta {delta}"
    );
}

// ---- Category 4: Regular Gas Cap Enforcement ----

/// 4.1 Tight regular gas cap causes OOG.
/// With gas_limit=cap=30,000, regular gas budget = cap - initial_gas(21,000) = 9,000.
/// The SSTORE needs ~22,106 regular gas beyond intrinsic, so 9,000 is insufficient.
#[test]
fn test_tip1016_regular_gas_cap_causes_oog() {
    let bytecode = sstore_bytecode(0, 1);

    // Baseline with 100k succeeds.
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

    // State gas with gas_limit=cap=30,000 → regular gas = 30k - 21k = 9k (insufficient).
    let mut evm = state_gas_evm(bytecode, 30_000);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(30_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(
        result.is_halt(),
        "Expected OOG halt with tight regular gas cap, got success with gas_used={}",
        result.gas_used()
    );
    match &result {
        revm::context_interface::result::ExecutionResult::Halt { reason, .. } => {
            assert!(
                matches!(reason, HaltReason::OutOfGas(_)),
                "Expected OutOfGas halt, got {reason:?}"
            );
        }
        _ => panic!("Expected Halt variant"),
    }
}

/// 4.2 Adequate regular gas cap: success.
#[test]
fn test_tip1016_regular_gas_cap_sufficient() {
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
    let baseline_gas = baseline_result.gas_used();
    assert!(baseline_result.is_success());

    // cap=100,000 → regular gas = 100,000 - 20,000 = 80,000. Plenty.
    let mut evm = state_gas_evm(bytecode, 100_000);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(100_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success());
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(delta, STATE_GAS_SSTORE_SET);
}

/// 4.3 Remaining gas insufficient after state gas deduction.
/// gas_limit=50,000 is enough for execution (~43,106) but NOT for execution + state gas (63,106).
#[test]
fn test_tip1016_state_gas_oog_remaining() {
    let bytecode = sstore_bytecode(0, 1);

    // Baseline with 50,000 gas succeeds.
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

    // With state gas: gas_limit=50,000, cap=u64::MAX (no regular gas constraint).
    // Execution needs ~43,106 + 20,000 state gas = ~63,106 > 50,000 → OOG.
    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(50_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(
        result.is_halt(),
        "Expected OOG when remaining can't cover state gas, got success with gas_used={}",
        result.gas_used()
    );
}

// ---- Category 5: State Gas Propagation ----

/// 5.1 CREATE child's state gas propagates to parent on success.
#[test]
fn test_tip1016_create_child_propagates() {
    // Same as test 2.3 — CREATE with init code that SSTOREs and returns 1-byte code.
    // All 4 state gas types (new_account + create + sstore + code_deposit) propagate.
    let init = init_code_sstore_and_return();
    let bytecode = create_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let expected =
        STATE_GAS_NEW_ACCOUNT + STATE_GAS_CREATE + STATE_GAS_SSTORE_SET + STATE_GAS_CODE_DEPOSIT;
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(
        delta, expected,
        "All child state gas should propagate to parent, got delta {delta}"
    );
}

/// 5.2 Reverted CREATE: child's SSTORE state gas is NOT propagated to parent's state_gas counter,
/// but the gas consumed by the child for state gas is not returned on revert (it's lost).
/// The parent's own state gas (new_account + create) was charged before the child ran.
#[test]
fn test_tip1016_reverted_create_child() {
    let init = init_code_sstore_and_revert();
    let bytecode = create_bytecode(&init);

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    let delta = result.gas_used() - baseline_gas;
    // Parent charged new_account(25k) + create(32k) = 57k state gas before child ran.
    // Child's SSTORE charged 20k state gas on child's gas (reduced child remaining).
    // On revert: child remaining is returned to parent, but the 20k spent on state gas
    // in the child is NOT returned. So total gas_used delta = 57k + 20k = 77k.
    let expected = STATE_GAS_NEW_ACCOUNT + STATE_GAS_CREATE + STATE_GAS_SSTORE_SET;
    assert_eq!(
        delta, expected,
        "Reverted CREATE: parent state gas + child's lost state gas, got delta {delta}"
    );
}

// ---- Category 6: Interactions ----

/// 6.1 SSTORE 0→1 (state gas), then 1→0 (refund). Refund does NOT undo state gas.
#[test]
fn test_tip1016_sstore_set_then_clear_refund() {
    let bytecode = sstore_set_then_clear_bytecode();

    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    let baseline_gas = baseline_result.gas_used();

    let mut evm = state_gas_evm(bytecode, u64::MAX);
    let result = evm
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();

    assert!(result.is_success());
    // State gas increases total `spent`, which raises the refund cap (spent/5 in London+).
    // Raw refund exceeds the cap, so both baseline and state gas use the cap.
    // This means state gas effectively increases the refund by state_gas/5.
    // Net delta = state_gas - state_gas/5 = state_gas * 4/5
    let delta = result.gas_used() - baseline_gas;
    let expected = STATE_GAS_SSTORE_SET * 4 / 5; // 16,000
    assert_eq!(
        delta, expected,
        "State gas persists but raises refund cap: net delta = state_gas*4/5, got {delta}"
    );
}

/// 6.2 State gas does not reduce regular gas budget.
/// With gas_limit=cap=50,000, regular gas = cap - intrinsic(21,000) = 29,000.
/// SSTORE regular gas ~22,106 fits in budget. State gas (20,000) is separate.
/// Total gas_used = ~43,106 + 20,000 = ~63,106 which exceeds gas_limit=50,000.
/// But this shows the *regular gas* check passes — the OOG happens on `remaining`, not regular gas.
/// So we use gas_limit=100,000 and cap=100,000 to have enough remaining too.
#[test]
fn test_tip1016_state_gas_does_not_reduce_regular_gas() {
    let bytecode = sstore_bytecode(0, 1);

    // Baseline succeeds.
    let mut baseline = baseline_evm(bytecode.clone());
    let baseline_result = baseline
        .transact_one(TxEnv::builder_for_bench().gas_price(0).build_fill())
        .unwrap();
    assert!(baseline_result.is_success());
    let baseline_gas = baseline_result.gas_used();

    // cap = baseline_gas + STATE_GAS_SSTORE_SET + 1 (just enough for execution + state).
    // This ensures regular gas budget = cap - intrinsic is tight but sufficient,
    // proving state gas is not subtracted from the regular gas budget.
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

    assert!(
        result.is_success(),
        "Should succeed: state gas doesn't consume regular gas budget. Gas used: {}",
        result.gas_used()
    );
    let delta = result.gas_used() - baseline_gas;
    assert_eq!(delta, STATE_GAS_SSTORE_SET);
}
