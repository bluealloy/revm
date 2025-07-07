//! Integration tests for the `op-revm` crate.
mod common;

use common::compare_or_save_testdata;
use context::ContextTr;
use database::BENCH_CALLER;
use primitives::{address, b256, hardfork::SpecId, Bytes, TxKind, KECCAK_EMPTY};
use revm::{
    bytecode::opcode,
    context::TxEnv,
    database::{BenchmarkDB, BENCH_TARGET},
    primitives::U256,
    state::Bytecode,
    Context, ExecuteEvm, MainBuilder, MainContext,
};
use state::AccountStatus;

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
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::BERLIN)
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

    compare_or_save_testdata(
        "test_selfdestruct_multi_tx.json",
        (result1, result2, output),
    );
}

/// Tests multiple transactions with contract creation.
/// Verifies that created contracts persist correctly across transactions
/// and that their state is properly maintained.
#[test]
pub fn test_multi_tx_create() {
    let mut evm = Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.spec = SpecId::BERLIN;
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

    compare_or_save_testdata(
        "test_multi_tx_create.json",
        (result1, result2, result3, output),
    );
}

/// Creates deployment bytecode for a contract.
/// Prepends the initialization code that will deploy the provided runtime bytecode.
pub fn deployment_contract(bytes: &[u8]) -> Bytes {
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
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::BERLIN)
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
    compare_or_save_testdata("test_frame_stack_index.json", result1);
}

#[test]
#[cfg(feature = "optional_balance_check")]
fn test_disable_balance_check_deduct() {
    use database::BENCH_CALLER_BALANCE;

    const RETURN_CALLER_BALANCE_BYTECODE: &[u8] = &[
        opcode::CALLER,
        opcode::BALANCE,
        opcode::PUSH1, 0x00,
        opcode::MSTORE,
        opcode::PUSH1, 0x20,
        opcode::PUSH1, 0x00,
        opcode::RETURN,
    ];

    let mut evm = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.disable_balance_check = true)
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new_legacy(
            RETURN_CALLER_BALANCE_BYTECODE.into(),
        )))
        .build_mainnet();

    let gas_limit = 100_000;

    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .gas_price(1)
                .gas_limit(gas_limit)
                .build_fill()
        )
        .unwrap();

    assert!(result.is_success());

    let returned_balance = U256::from_be_slice(result.output().unwrap().as_ref());
    let expected_balance = BENCH_CALLER_BALANCE - U256::from(gas_limit);
    assert_eq!(returned_balance, expected_balance);
}
