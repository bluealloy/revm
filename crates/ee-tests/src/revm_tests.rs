//! Integration tests for the `revm` crate.

use crate::TestdataConfig;
use revm::{
    bytecode::opcode,
    context::{CfgEnv, ContextTr, TxEnv},
    database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET},
    primitives::{address, b256, hardfork::SpecId, Bytes, TxKind, KECCAK_EMPTY, U256},
    state::{AccountStatus, Bytecode},
    Context, ExecuteEvm, MainBuilder, MainContext,
};
use std::path::PathBuf;

// Re-export the constant for testdata directory path
const TESTS_TESTDATA: &str = "tests/revm_testdata";

fn revm_testdata_config() -> TestdataConfig {
    TestdataConfig {
        testdata_dir: PathBuf::from(TESTS_TESTDATA),
    }
}

fn compare_or_save_revm_testdata<T>(filename: &str, output: &T)
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a> + PartialEq + std::fmt::Debug,
{
    crate::compare_or_save_testdata_with_config(filename, output, revm_testdata_config());
}

const SELFDESTRUCT_BYTECODE: &[u8] = &[
    opcode::PUSH2,
    0xFF,
    0xFF,
    opcode::SELFDESTRUCT,
    opcode::STOP,
];

#[test]
fn test_selfdestruct_multi_tx() {
    let mut evm = Context::mainnet()
        .with_cfg(CfgEnv::new_with_spec(SpecId::BERLIN))
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new_legacy(
            SELFDESTRUCT_BYTECODE.into(),
        )))
        .build_mainnet();

    // trigger selfdestruct
    let result1 = evm
        .transact_one(TxEnv::builder_for_bench().build_fill())
        .unwrap();

    let destroyed_acc = evm.ctx.journal_mut().state.get_mut(&BENCH_TARGET).unwrap();

    // balance got transferred to 0x0000..00FFFF
    assert_eq!(destroyed_acc.info.balance, U256::ZERO);
    assert_eq!(destroyed_acc.info.nonce, 1);
    assert_eq!(
        destroyed_acc.info.code_hash,
        b256!("0x9125466aa9ef15459d85e7318f6d3bdc5f6978c0565bee37a8e768d7c202a67a")
    );

    // call on destroyed account. This accounts gets loaded and should contain empty code_hash afterwards.
    let result2 = evm
        .transact_one(TxEnv::builder_for_bench().nonce(1).build_fill())
        .unwrap();

    let destroyed_acc = evm.ctx.journal_mut().state.get_mut(&BENCH_TARGET).unwrap();

    assert_eq!(destroyed_acc.info.code_hash, KECCAK_EMPTY);
    assert_eq!(destroyed_acc.info.nonce, 0);
    assert_eq!(destroyed_acc.info.code, Some(Bytecode::default()));

    let output = evm.finalize();

    compare_or_save_revm_testdata(
        "test_selfdestruct_multi_tx.json",
        &(result1, result2, output),
    );
}

/// Tests multiple transactions with contract creation.
/// Verifies that created contracts persist correctly across transactions
/// and that their state is properly maintained.
#[test]
fn test_multi_tx_create() {
    let mut evm = Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::BERLIN);
            cfg.disable_nonce_check = true;
        })
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .build_mainnet();

    let result1 = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .kind(TxKind::Create)
                .data(deployment_contract(SELFDESTRUCT_BYTECODE))
                .build_fill(),
        )
        .unwrap();

    let created_address = result1.created_address().unwrap();

    let created_acc = evm
        .ctx
        .journal_mut()
        .state
        .get_mut(&created_address)
        .unwrap();

    assert_eq!(
        created_acc.status,
        AccountStatus::Created
            | AccountStatus::CreatedLocal
            | AccountStatus::Touched
            | AccountStatus::LoadedAsNotExisting
    );

    let result2 = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .nonce(1)
                .kind(TxKind::Call(created_address))
                .build_fill(),
        )
        .unwrap();

    let created_acc = evm
        .ctx
        .journal_mut()
        .state
        .get_mut(&created_address)
        .unwrap();

    // reset nonce to trigger create on same address.
    assert_eq!(
        created_acc.status,
        AccountStatus::Created
            | AccountStatus::SelfDestructed
            | AccountStatus::SelfDestructedLocal
            | AccountStatus::Touched
            | AccountStatus::LoadedAsNotExisting
    );

    // reset caller nonce
    evm.ctx
        .journal_mut()
        .state
        .get_mut(&BENCH_CALLER)
        .unwrap()
        .info
        .nonce = 0;

    // re create the contract.
    let result3 = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .nonce(0)
                .kind(TxKind::Create)
                .data(deployment_contract(SELFDESTRUCT_BYTECODE))
                .build_fill(),
        )
        .unwrap();

    let created_address_new = result3.created_address().unwrap();
    assert_eq!(created_address, created_address_new);

    let created_acc = evm
        .ctx
        .journal_mut()
        .state
        .get_mut(&created_address)
        .unwrap();

    assert_eq!(
        created_acc.status,
        AccountStatus::Created
            | AccountStatus::CreatedLocal
            | AccountStatus::Touched
            | AccountStatus::SelfDestructed
            | AccountStatus::LoadedAsNotExisting
    );
    let output = evm.finalize();

    compare_or_save_revm_testdata(
        "test_multi_tx_create.json",
        &(result1, result2, result3, output),
    );
}

/// Creates deployment bytecode for a contract.
/// Prepends the initialization code that will deploy the provided runtime bytecode.
fn deployment_contract(bytes: &[u8]) -> Bytes {
    assert!(bytes.len() < 256);
    let len = bytes.len();
    let ret = &[
        opcode::PUSH1,
        len as u8,
        opcode::PUSH1,
        12,
        opcode::PUSH1,
        0,
        // Copy code to memory.
        opcode::CODECOPY,
        opcode::PUSH1,
        len as u8,
        opcode::PUSH1,
        0,
        // Return copied code.
        opcode::RETURN,
    ];

    [ret, bytes].concat().into()
}

#[test]
fn test_frame_stack_index() {
    let mut evm = Context::mainnet()
        .with_cfg(CfgEnv::new_with_spec(SpecId::BERLIN))
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new_legacy(
            SELFDESTRUCT_BYTECODE.into(),
        )))
        .build_mainnet();

    // transfer to other account
    let result1 = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .to(address!("0xc000000000000000000000000000000000000000"))
                .build_fill(),
        )
        .unwrap();

    assert_eq!(evm.frame_stack.index(), None);
    compare_or_save_revm_testdata("test_frame_stack_index.json", &result1);
}

#[test]
#[cfg(feature = "optional_balance_check")]
fn test_disable_balance_check() {
    use revm::database::BENCH_CALLER_BALANCE;
    const RETURN_CALLER_BALANCE_BYTECODE: &[u8] = &[
        opcode::CALLER,
        opcode::BALANCE,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];

    let mut evm = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.disable_balance_check = true)
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new_legacy(
            RETURN_CALLER_BALANCE_BYTECODE.into(),
        )))
        .build_mainnet();

    // Construct tx so that effective cost is more than caller balance.
    let gas_price = 1;
    let gas_limit = 100_000;
    // Make sure value doesn't consume all balance since we want to validate that all effective
    // cost is deducted.
    let tx_value = BENCH_CALLER_BALANCE - U256::from(1);

    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_price(gas_price)
                .gas_limit(gas_limit)
                .value(tx_value)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success());

    let returned_balance = U256::from_be_slice(result.output().unwrap().as_ref());
    let expected_balance = U256::ZERO;
    assert_eq!(returned_balance, expected_balance);
}

// ============================================================================
// EIP-7708: ETH transfers emit a log
// ============================================================================

use revm::primitives::eip7708::{
    ETH_TRANSFER_LOG_ADDRESS, ETH_TRANSFER_LOG_TOPIC, SELFDESTRUCT_LOG_TOPIC,
};
use revm::primitives::B256;

/// Test EIP-7708 transfer log emission for transaction value transfer
#[test]
fn test_eip7708_transfer_log_tx_value() {
    let recipient = address!("0xc000000000000000000000000000000000000001");

    let mut evm = Context::mainnet()
        .with_cfg(CfgEnv::new_with_spec(SpecId::AMSTERDAM))
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .build_mainnet();

    let tx_value = U256::from(1_000_000_000_000_000u128); // 0.001 ETH (within balance)

    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .to(recipient)
                .value(tx_value)
                .gas_limit(100_000)
                .gas_price(0) // Zero gas price to avoid balance issues
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success());

    // Verify that a transfer log was emitted
    let logs = result.logs();
    assert_eq!(logs.len(), 1, "Expected 1 transfer log");

    let log = &logs[0];
    assert_eq!(log.address, ETH_TRANSFER_LOG_ADDRESS);
    assert_eq!(log.data.topics().len(), 3);
    assert_eq!(log.data.topics()[0], ETH_TRANSFER_LOG_TOPIC);
    assert_eq!(
        log.data.topics()[1],
        B256::left_padding_from(BENCH_CALLER.as_slice())
    );
    assert_eq!(
        log.data.topics()[2],
        B256::left_padding_from(recipient.as_slice())
    );
    assert_eq!(log.data.data.as_ref(), &tx_value.to_be_bytes::<32>());
}

/// Test that no transfer log is emitted for zero value transfer
#[test]
fn test_eip7708_no_log_for_zero_value() {
    let recipient = address!("0xc000000000000000000000000000000000000001");

    let mut evm = Context::mainnet()
        .with_cfg(CfgEnv::new_with_spec(SpecId::AMSTERDAM))
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .build_mainnet();

    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .to(recipient)
                .value(U256::ZERO)
                .gas_limit(100_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success());

    // No logs should be emitted for zero value transfer
    let logs = result.logs();
    assert_eq!(logs.len(), 0, "Expected no logs for zero value transfer");
}

/// Test that no transfer log is emitted before AMSTERDAM
#[test]
fn test_eip7708_no_log_before_amsterdam() {
    let recipient = address!("0xc000000000000000000000000000000000000001");

    let mut evm = Context::mainnet()
        .with_cfg(CfgEnv::new_with_spec(SpecId::OSAKA)) // Before AMSTERDAM
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .build_mainnet();

    let tx_value = U256::from(1_000_000_000_000_000u128); // 0.001 ETH (within balance)

    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .to(recipient)
                .value(tx_value)
                .gas_limit(100_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success());

    // No logs should be emitted before AMSTERDAM
    let logs = result.logs();
    assert_eq!(logs.len(), 0, "Expected no logs before AMSTERDAM");
}

/// Bytecode that selfdestructs to the caller (different address)
const SELFDESTRUCT_TO_CALLER_BYTECODE: &[u8] = &[
    opcode::CALLER, // Push caller address
    opcode::SELFDESTRUCT,
    opcode::STOP,
];

/// Test EIP-7708 transfer log emission for selfdestruct to different address
#[test]
fn test_eip7708_selfdestruct_to_different_address() {
    let mut evm = Context::mainnet()
        .with_cfg(CfgEnv::new_with_spec(SpecId::AMSTERDAM))
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new_legacy(
            SELFDESTRUCT_TO_CALLER_BYTECODE.into(),
        )))
        .build_mainnet();

    // Execute selfdestruct - the contract at BENCH_TARGET will selfdestruct to BENCH_CALLER
    // After Cancun (and Amsterdam), SELFDESTRUCT only destroys if created in same tx,
    // but it still transfers balance, so we should see a transfer log.
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(100_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success());

    // There should be two logs:
    // 1. Transfer log for initial tx value transfer from BENCH_CALLER to BENCH_TARGET
    // 2. Transfer log for selfdestruct balance transfer from BENCH_TARGET to BENCH_CALLER
    let logs = result.logs();

    // Find the selfdestruct transfer log (from BENCH_TARGET to BENCH_CALLER)
    let selfdestruct_log = logs.iter().find(|log| {
        log.data.topics().len() == 3
            && log.data.topics()[0] == ETH_TRANSFER_LOG_TOPIC
            && log.data.topics()[1] == B256::left_padding_from(BENCH_TARGET.as_slice())
    });

    assert!(
        selfdestruct_log.is_some(),
        "Expected selfdestruct transfer log"
    );
    let log = selfdestruct_log.unwrap();
    assert_eq!(log.address, ETH_TRANSFER_LOG_ADDRESS);
    assert_eq!(
        log.data.topics()[2],
        B256::left_padding_from(BENCH_CALLER.as_slice())
    );
}

/// Init code that selfdestructs to itself during construction
/// This triggers the selfdestruct-to-self scenario where a newly created
/// contract (is_created_locally = true) selfdestructs to itself.
const SELFDESTRUCT_TO_SELF_INIT_CODE: &[u8] = &[
    opcode::ADDRESS, // Push contract's own address
    opcode::SELFDESTRUCT,
];

/// Test EIP-7708 selfdestruct-to-self log emission
/// This test creates a contract with value that selfdestructs to itself during construction,
/// which should emit a SelfBalanceLog since the contract is created in the same tx.
#[test]
fn test_eip7708_selfdestruct_to_self() {
    let mut evm = Context::mainnet()
        .with_cfg(CfgEnv::new_with_spec(SpecId::AMSTERDAM))
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .build_mainnet();

    let create_value = U256::from(1_000_000u128); // 1M wei

    // Create a contract with value that selfdestructs to itself during init
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .kind(TxKind::Create)
                .data(SELFDESTRUCT_TO_SELF_INIT_CODE.into())
                .value(create_value)
                .gas_limit(100_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success(), "Transaction should succeed");

    // Find the selfdestruct-to-self log
    let logs = result.logs();
    let selfdestruct_to_self_log = logs
        .iter()
        .find(|log| log.data.topics().len() == 2 && log.data.topics()[0] == SELFDESTRUCT_LOG_TOPIC);

    assert!(
        selfdestruct_to_self_log.is_some(),
        "Expected selfdestruct-to-self log, got logs: {:?}",
        logs
    );
    let log = selfdestruct_to_self_log.unwrap();
    assert_eq!(log.address, ETH_TRANSFER_LOG_ADDRESS);
    // The log data should contain the create value
    assert_eq!(log.data.data.as_ref(), &create_value.to_be_bytes::<32>());
}

/// Bytecode that performs a CALL with value to a specific address
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

/// Test EIP-7708 transfer log emission for CALL with value
#[test]
fn test_eip7708_call_with_value() {
    let call_target = address!("0xd000000000000000000000000000000000000001");
    let call_value = U256::from(1_000_000u128); // Small value (1M wei)

    let bytecode = call_with_value_bytecode(call_target.into_array(), call_value);

    let mut evm = Context::mainnet()
        .with_cfg(CfgEnv::new_with_spec(SpecId::AMSTERDAM))
        .with_db(BenchmarkDB::new_bytecode(bytecode))
        .build_mainnet();

    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(200_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success(), "Transaction should succeed");

    // There should be transfer logs:
    // Transfer from BENCH_TARGET to call_target (CALL value transfer)
    let logs = result.logs();

    // Find the CALL transfer log (from BENCH_TARGET to call_target)
    let call_log = logs.iter().find(|log| {
        log.data.topics().len() == 3
            && log.data.topics()[0] == ETH_TRANSFER_LOG_TOPIC
            && log.data.topics()[1] == B256::left_padding_from(BENCH_TARGET.as_slice())
            && log.data.topics()[2] == B256::left_padding_from(call_target.as_slice())
    });

    assert!(
        call_log.is_some(),
        "Expected CALL transfer log, got logs: {:?}",
        logs
    );
    let log = call_log.unwrap();
    assert_eq!(log.address, ETH_TRANSFER_LOG_ADDRESS);
    assert_eq!(log.data.data.as_ref(), &call_value.to_be_bytes::<32>());
}

/// Bytecode that creates a contract with initial value
#[allow(clippy::vec_init_then_push)]
fn create_with_value_bytecode(init_code: &[u8], value: U256) -> Bytecode {
    // CREATE(value, offset, length)
    let mut bytecode = Vec::new();

    // First, store init_code in memory
    // PUSH init_code bytes
    for (i, byte) in init_code.iter().enumerate() {
        bytecode.push(opcode::PUSH1);
        bytecode.push(*byte);
        bytecode.push(opcode::PUSH1);
        bytecode.push(i as u8);
        bytecode.push(opcode::MSTORE8);
    }

    // Push length
    bytecode.push(opcode::PUSH1);
    bytecode.push(init_code.len() as u8);

    // Push offset (0)
    bytecode.push(opcode::PUSH1);
    bytecode.push(0);

    // Push value (32 bytes)
    let value_bytes = value.to_be_bytes::<32>();
    bytecode.push(opcode::PUSH32);
    bytecode.extend_from_slice(&value_bytes);

    // Execute CREATE
    bytecode.push(opcode::CREATE);

    // Clean up stack
    bytecode.push(opcode::POP);

    // Stop
    bytecode.push(opcode::STOP);

    Bytecode::new_legacy(bytecode.into())
}

/// Simple init code that just returns nothing (creates empty contract)
const SIMPLE_INIT_CODE: &[u8] = &[
    opcode::PUSH1,
    0, // length
    opcode::PUSH1,
    0, // offset
    opcode::RETURN,
];

/// Test EIP-7708 transfer log emission for CREATE with value
#[test]
fn test_eip7708_create_with_value() {
    let create_value = U256::from(1_000_000u128); // Small value (1M wei)

    let bytecode = create_with_value_bytecode(SIMPLE_INIT_CODE, create_value);

    let mut evm = Context::mainnet()
        .with_cfg(CfgEnv::new_with_spec(SpecId::AMSTERDAM))
        .with_db(BenchmarkDB::new_bytecode(bytecode))
        .build_mainnet();

    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_limit(200_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success(), "Transaction should succeed");

    // There should be transfer logs:
    // Transfer from BENCH_TARGET to created address (CREATE value transfer)
    let logs = result.logs();

    // Find the CREATE transfer log (from BENCH_TARGET to some address)
    let create_log = logs.iter().find(|log| {
        log.data.topics().len() == 3
            && log.data.topics()[0] == ETH_TRANSFER_LOG_TOPIC
            && log.data.topics()[1] == B256::left_padding_from(BENCH_TARGET.as_slice())
            && log.data.topics()[2] != B256::left_padding_from(BENCH_CALLER.as_slice())
    });

    assert!(
        create_log.is_some(),
        "Expected CREATE transfer log, got logs: {:?}",
        logs
    );
    let log = create_log.unwrap();
    assert_eq!(log.address, ETH_TRANSFER_LOG_ADDRESS);
    assert_eq!(log.data.data.as_ref(), &create_value.to_be_bytes::<32>());
}
