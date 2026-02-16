use criterion::Criterion;
use revm::{
    bytecode::opcode,
    context::TxEnv,
    database::{InMemoryDB, BENCH_CALLER, BENCH_TARGET},
    primitives::{address, Address, TxKind, U256},
    state::{AccountInfo, Bytecode},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

const SUBCALL_TARGET_A: Address = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
const SUBCALL_TARGET_B: Address = address!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");

/// Constructs bytecode that loops 1000 times, each iteration doing a CALL to `target`
/// with the given `value` (0 or 1 wei).
fn make_loop_call_bytecode(target: Address, value: u8) -> Bytecode {
    let mut code = Vec::new();

    // PUSH2 0x03E8 (1000) — loop counter
    code.push(opcode::PUSH2);
    code.push(0x03);
    code.push(0xE8);

    // JUMPDEST — loop_start at offset 3
    code.push(opcode::JUMPDEST);

    // Set up CALL args: retSize, retOffset, argsSize, argsOffset, value, target, gas
    // PUSH1 0x00 — retSize
    code.push(opcode::PUSH1);
    code.push(0x00);
    // PUSH1 0x00 — retOffset
    code.push(opcode::PUSH1);
    code.push(0x00);
    // PUSH1 0x00 — argsSize
    code.push(opcode::PUSH1);
    code.push(0x00);
    // PUSH1 0x00 — argsOffset
    code.push(opcode::PUSH1);
    code.push(0x00);
    // PUSH1 <value> — value
    code.push(opcode::PUSH1);
    code.push(value);
    // PUSH20 <target> — target address
    code.push(opcode::PUSH20);
    code.extend_from_slice(target.as_slice());
    // GAS — forward all remaining gas
    code.push(opcode::GAS);
    // CALL
    code.push(opcode::CALL);
    // POP — discard success/failure
    code.push(opcode::POP);

    // Decrement counter: PUSH1 1, SWAP1, SUB
    code.push(opcode::PUSH1);
    code.push(0x01);
    code.push(opcode::SWAP1);
    code.push(opcode::SUB);

    // DUP1 — duplicate counter for JUMPI check
    code.push(opcode::DUP1);

    // PUSH1 0x03 — jump target (JUMPDEST offset)
    code.push(opcode::PUSH1);
    code.push(0x03);

    // JUMPI — jump back if counter != 0
    code.push(opcode::JUMPI);

    // POP — clean up remaining counter (0)
    code.push(opcode::POP);

    // STOP
    code.push(opcode::STOP);

    Bytecode::new_raw(code.into())
}

/// Minimal contract that just STOPs.
fn make_stop_bytecode() -> Bytecode {
    Bytecode::new_raw([opcode::STOP].into())
}

/// Constructs bytecode that does a single CALL (no value) to `target`, then STOPs.
fn make_subcall_bytecode(target: Address) -> Bytecode {
    let mut code = Vec::new();

    // PUSH1 0x00 — retSize
    code.push(opcode::PUSH1);
    code.push(0x00);
    // PUSH1 0x00 — retOffset
    code.push(opcode::PUSH1);
    code.push(0x00);
    // PUSH1 0x00 — argsSize
    code.push(opcode::PUSH1);
    code.push(0x00);
    // PUSH1 0x00 — argsOffset
    code.push(opcode::PUSH1);
    code.push(0x00);
    // PUSH1 0x00 — value (no transfer)
    code.push(opcode::PUSH1);
    code.push(0x00);
    // PUSH20 <target>
    code.push(opcode::PUSH20);
    code.extend_from_slice(target.as_slice());
    // GAS
    code.push(opcode::GAS);
    // CALL
    code.push(opcode::CALL);
    // POP
    code.push(opcode::POP);
    // STOP
    code.push(opcode::STOP);

    Bytecode::new_raw(code.into())
}

pub fn run(criterion: &mut Criterion) {
    // Variant 1: 1000 subcalls each transferring 1 wei
    {
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo {
                balance: U256::from(u128::MAX),
                ..Default::default()
            },
        );
        db.insert_account_info(
            BENCH_TARGET,
            AccountInfo {
                balance: U256::from(u128::MAX),
                code: Some(make_loop_call_bytecode(SUBCALL_TARGET_A, 1)),
                ..Default::default()
            },
        );
        db.insert_account_info(
            SUBCALL_TARGET_A,
            AccountInfo {
                code: Some(make_stop_bytecode()),
                ..Default::default()
            },
        );

        let mut evm = Context::mainnet()
            .with_db(db)
            .modify_cfg_chained(|c| {
                c.disable_nonce_check = true;
                c.tx_gas_limit_cap = Some(u64::MAX);
            })
            .build_mainnet();

        let tx = TxEnv::builder()
            .caller(BENCH_CALLER)
            .kind(TxKind::Call(BENCH_TARGET))
            .gas_limit(u64::MAX)
            .build()
            .unwrap();

        criterion.bench_function("subcall_1000_transfer_1wei", |b| {
            b.iter_batched(
                || tx.clone(),
                |input| evm.transact_one(input).unwrap(),
                criterion::BatchSize::SmallInput,
            );
        });
    }

    // Variant 2: 1000 subcalls with no value transfer (same account)
    {
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo {
                balance: U256::from(u128::MAX),
                ..Default::default()
            },
        );
        db.insert_account_info(
            BENCH_TARGET,
            AccountInfo {
                code: Some(make_loop_call_bytecode(SUBCALL_TARGET_A, 0)),
                ..Default::default()
            },
        );
        db.insert_account_info(
            SUBCALL_TARGET_A,
            AccountInfo {
                code: Some(make_stop_bytecode()),
                ..Default::default()
            },
        );

        let mut evm = Context::mainnet()
            .with_db(db)
            .modify_cfg_chained(|c| {
                c.disable_nonce_check = true;
                c.tx_gas_limit_cap = Some(u64::MAX);
            })
            .build_mainnet();

        let tx = TxEnv::builder()
            .caller(BENCH_CALLER)
            .kind(TxKind::Call(BENCH_TARGET))
            .gas_limit(u64::MAX)
            .build()
            .unwrap();

        criterion.bench_function("subcall_1000_same_account", |b| {
            b.iter_batched(
                || tx.clone(),
                |input| evm.transact_one(input).unwrap(),
                criterion::BatchSize::SmallInput,
            );
        });
    }

    // Variant 3: 1000 subcalls where each target does another subcall (nested)
    {
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo {
                balance: U256::from(u128::MAX),
                ..Default::default()
            },
        );
        db.insert_account_info(
            BENCH_TARGET,
            AccountInfo {
                code: Some(make_loop_call_bytecode(SUBCALL_TARGET_A, 0)),
                ..Default::default()
            },
        );
        db.insert_account_info(
            SUBCALL_TARGET_A,
            AccountInfo {
                code: Some(make_subcall_bytecode(SUBCALL_TARGET_B)),
                ..Default::default()
            },
        );
        db.insert_account_info(
            SUBCALL_TARGET_B,
            AccountInfo {
                code: Some(make_stop_bytecode()),
                ..Default::default()
            },
        );

        let mut evm = Context::mainnet()
            .with_db(db)
            .modify_cfg_chained(|c| {
                c.disable_nonce_check = true;
                c.tx_gas_limit_cap = Some(u64::MAX);
            })
            .build_mainnet();

        let tx = TxEnv::builder()
            .caller(BENCH_CALLER)
            .kind(TxKind::Call(BENCH_TARGET))
            .gas_limit(u64::MAX)
            .build()
            .unwrap();

        criterion.bench_function("subcall_1000_nested", |b| {
            b.iter_batched(
                || tx.clone(),
                |input| evm.transact_one(input).unwrap(),
                criterion::BatchSize::SmallInput,
            );
        });
    }
}
