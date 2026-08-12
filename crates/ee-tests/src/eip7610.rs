//! EIP-7610 contract-creation collision tests.

use revm::{
    bytecode::opcode,
    context::TxEnv,
    context_interface::result::{ExecutionResult, HaltReason},
    database::{CacheDB, EmptyDB, BENCH_CALLER},
    primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, B256, U256},
    state::{AccountInfo, Bytecode},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

const CREATOR: Address = address!("0x000000000000000000000000000000000000c0de");

fn storage_only_target(db: &mut CacheDB<EmptyDB>, target: Address) {
    db.insert_account_info(target, AccountInfo::default());
    db.insert_account_storage(target, U256::ZERO, U256::from(1))
        .unwrap();
}

fn funded_caller(db: &mut CacheDB<EmptyDB>, nonce: u64) {
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo {
            balance: U256::from(u64::MAX),
            nonce,
            ..Default::default()
        },
    );
}

fn create_and_return(opcode: u8) -> Bytecode {
    let mut code = vec![opcode::PUSH1, 0, opcode::PUSH1, 0, opcode::PUSH1, 0];
    if opcode == opcode::CREATE2 {
        code.extend([opcode::PUSH1, 0]);
    }
    code.extend([
        opcode,
        opcode::PUSH1,
        0,
        opcode::MSTORE,
        opcode::PUSH1,
        32,
        opcode::PUSH1,
        0,
        opcode::RETURN,
    ]);
    Bytecode::new_raw(code.into())
}

fn run_opcode_create(spec: SpecId, opcode: u8, target_has_storage: bool) -> Bytes {
    let mut db = CacheDB::<EmptyDB>::default();
    funded_caller(&mut db, 0);
    db.insert_account_info(
        CREATOR,
        AccountInfo::default()
            .with_nonce(1)
            .with_code(create_and_return(opcode)),
    );

    let target = if opcode == opcode::CREATE {
        CREATOR.create(1)
    } else {
        CREATOR.create2_from_code(B256::ZERO, Bytes::new())
    };
    db.insert_account_info(target, AccountInfo::default().with_balance(U256::from(1)));
    if target_has_storage {
        storage_only_target(&mut db, target);
    }

    let mut evm = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(spec))
        .with_db(db)
        .build_mainnet();
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .kind(TxKind::Call(CREATOR))
                .gas_limit(1_000_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    result
        .output()
        .unwrap_or_else(|| panic!("unexpected create result: {result:?}"))
        .clone()
}

#[test]
fn tx_create_collides_with_storage_only_account() {
    let target = BENCH_CALLER.create(0);
    let mut db = CacheDB::<EmptyDB>::default();
    funded_caller(&mut db, 0);
    storage_only_target(&mut db, target);

    let mut evm = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SpecId::FRONTIER))
        .with_db(db)
        .build_mainnet();
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .kind(TxKind::Create)
                .gas_limit(1_000_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(matches!(
        result,
        ExecutionResult::Halt {
            reason: HaltReason::CreateCollision,
            ..
        }
    ));
}

#[test]
fn tx_create_allows_balance_only_account() {
    let target = BENCH_CALLER.create(0);
    let mut db = CacheDB::<EmptyDB>::default();
    funded_caller(&mut db, 0);
    db.insert_account_info(target, AccountInfo::default().with_balance(U256::from(1)));

    let mut evm = Context::mainnet().with_db(db).build_mainnet();
    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .kind(TxKind::Create)
                .gas_limit(1_000_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();

    assert!(result.is_success());
}

#[test]
fn create_collision_rule_is_retroactive() {
    assert_eq!(
        run_opcode_create(SpecId::BYZANTIUM, opcode::CREATE, true),
        Bytes::from(vec![0; 32])
    );
    assert_ne!(
        run_opcode_create(SpecId::BYZANTIUM, opcode::CREATE, false),
        Bytes::from(vec![0; 32])
    );
}

#[test]
fn create2_collides_with_storage_only_account() {
    assert_eq!(
        run_opcode_create(SpecId::PETERSBURG, opcode::CREATE2, true),
        Bytes::from(vec![0; 32])
    );
    assert_ne!(
        run_opcode_create(SpecId::PETERSBURG, opcode::CREATE2, false),
        Bytes::from(vec![0; 32])
    );
}

#[test]
fn balance_change_preserves_persisted_storage_collision() {
    let target = BENCH_CALLER.create(1);
    let mut db = CacheDB::<EmptyDB>::default();
    funded_caller(&mut db, 0);
    storage_only_target(&mut db, target);

    let mut evm = Context::mainnet().with_db(db).build_mainnet();
    let transfer = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .kind(TxKind::Call(target))
                .gas_limit(1_000_000)
                .value(U256::from(1))
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    assert!(transfer.is_success());

    let result = evm
        .transact_one(
            TxEnv::builder_for_bench()
                .nonce(1)
                .kind(TxKind::Create)
                .gas_limit(1_000_000)
                .gas_price(0)
                .build_fill(),
        )
        .unwrap();
    assert!(matches!(
        result,
        ExecutionResult::Halt {
            reason: HaltReason::CreateCollision,
            ..
        }
    ));
}
