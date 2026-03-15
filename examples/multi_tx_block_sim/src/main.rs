//! Multi-transaction block simulation example.
//!
//! Demonstrates how to simulate an entire block of transactions using REVM:
//! 1. Configure a realistic block environment (number, timestamp, coinbase, basefee)
//! 2. Deploy a counter contract in the first transaction
//! 3. Execute multiple transactions from different accounts that interact with the contract
//! 4. Track gas usage, state changes, and cumulative block gas across all transactions
//! 5. Show two execution strategies: per-transaction commit and batch commit
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use anyhow::bail;
use revm::{
    bytecode::opcode,
    context::{Context, ContextTr, TxEnv},
    context_interface::result::{ExecutionResult, Output},
    database::CacheDB,
    database_interface::EmptyDB,
    primitives::{address, Address, Bytes, TxKind, U256},
    state::AccountInfo,
    Database, ExecuteCommitEvm, MainBuilder, MainContext,
};

/// Counter contract bytecode:
/// - On CALL: reads slot 0, adds 1, stores back, returns new value
///
/// PUSH0 SLOAD PUSH1 1 ADD DUP1 PUSH0 SSTORE PUSH0 MSTORE PUSH1 32 PUSH0 RETURN
const COUNTER_RUNTIME: &[u8] = &[
    opcode::PUSH0,
    opcode::SLOAD,
    opcode::PUSH1,
    0x01,
    opcode::ADD,
    opcode::DUP1,
    opcode::PUSH0,
    opcode::SSTORE,
    opcode::PUSH0,
    opcode::MSTORE,
    opcode::PUSH1,
    0x20,
    opcode::PUSH0,
    opcode::RETURN,
];

/// Init code that deploys the counter runtime bytecode.
/// Copies runtime code from code to memory and returns it.
fn deployment_bytecode() -> Bytes {
    let runtime_len = COUNTER_RUNTIME.len() as u8;
    // PUSH1 <len> PUSH1 <offset> PUSH0 CODECOPY PUSH1 <len> PUSH0 RETURN
    // This init code is 10 bytes, so runtime starts at offset 10.
    let init_code: Vec<u8> = vec![
        opcode::PUSH1,
        runtime_len,
        opcode::PUSH1,
        10, // offset where runtime code starts (init code is 10 bytes)
        opcode::PUSH0,
        opcode::CODECOPY,
        opcode::PUSH1,
        runtime_len,
        opcode::PUSH0,
        opcode::RETURN,
    ];
    [init_code.as_slice(), COUNTER_RUNTIME].concat().into()
}

const ALICE: Address = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
const BOB: Address = address!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
const CHARLIE: Address = address!("0xcccccccccccccccccccccccccccccccccccccccc");
const COINBASE: Address = address!("0x000000000000000000000000000000000000c01b");

fn main() -> anyhow::Result<()> {
    println!("=== Multi-Transaction Block Simulation ===\n");

    // -------------------------------------------------------
    // Phase 1: Per-transaction commit (transact_commit)
    // Each transaction is finalized and committed individually.
    // State is visible to the next transaction immediately.
    // -------------------------------------------------------
    println!("--- Strategy 1: Per-transaction commit ---\n");

    let mut cache_db = CacheDB::new(EmptyDB::default());

    // Fund accounts with 10 ETH each
    let initial_balance = U256::from(10_000_000_000_000_000_000u128); // 10 ETH
    for addr in [ALICE, BOB, CHARLIE] {
        cache_db.insert_account_info(
            addr,
            AccountInfo {
                balance: initial_balance,
                ..Default::default()
            },
        );
    }

    // Build EVM with a realistic block environment
    let mut evm = Context::mainnet()
        .with_db(cache_db)
        .modify_block_chained(|block| {
            block.number = U256::from(18_000_000);
            block.timestamp = U256::from(1_700_000_000);
            block.beneficiary = COINBASE;
            block.gas_limit = 30_000_000;
            block.basefee = 10; // 10 wei basefee
        })
        .build_mainnet();

    let mut block_gas_used: u64 = 0;

    // TX 1: Alice deploys the counter contract
    println!("TX 1: Alice deploys counter contract");
    let deploy_tx = TxEnv::builder()
        .caller(ALICE)
        .kind(TxKind::Create)
        .data(deployment_bytecode())
        .gas_limit(100_000)
        .gas_price(10)
        .nonce(0)
        .build()?;

    let result = evm.transact_commit(deploy_tx)?;
    let contract_address = match &result {
        ExecutionResult::Success {
            output: Output::Create(_, Some(addr)),
            gas,
            ..
        } => {
            let used = gas.used();
            block_gas_used += used;
            println!("  Contract deployed at: {addr}");
            println!("  Gas used: {used}");
            *addr
        }
        _ => bail!("Deploy failed: {result:#?}"),
    };

    // TX 2: Alice increments counter (0 -> 1)
    println!("\nTX 2: Alice increments counter");
    let result = evm.transact_commit(
        TxEnv::builder()
            .caller(ALICE)
            .kind(TxKind::Call(contract_address))
            .gas_limit(100_000)
            .gas_price(10)
            .nonce(1)
            .build()?,
    )?;
    print_call_result(&result, &mut block_gas_used);

    // TX 3: Bob increments counter (1 -> 2)
    println!("\nTX 3: Bob increments counter");
    let result = evm.transact_commit(
        TxEnv::builder()
            .caller(BOB)
            .kind(TxKind::Call(contract_address))
            .gas_limit(100_000)
            .gas_price(10)
            .nonce(0)
            .build()?,
    )?;
    print_call_result(&result, &mut block_gas_used);

    // TX 4: Charlie increments counter (2 -> 3)
    println!("\nTX 4: Charlie increments counter");
    let result = evm.transact_commit(
        TxEnv::builder()
            .caller(CHARLIE)
            .kind(TxKind::Call(contract_address))
            .gas_limit(100_000)
            .gas_price(10)
            .nonce(0)
            .build()?,
    )?;
    print_call_result(&result, &mut block_gas_used);

    // TX 5: Alice sends 1 ETH to Bob (simple value transfer)
    println!("\nTX 5: Alice sends 1 ETH to Bob");
    let result = evm.transact_commit(
        TxEnv::builder()
            .caller(ALICE)
            .kind(TxKind::Call(BOB))
            .value(U256::from(1_000_000_000_000_000_000u128)) // 1 ETH
            .gas_limit(21_000)
            .gas_price(10)
            .nonce(2)
            .build()?,
    )?;
    match &result {
        ExecutionResult::Success { gas, .. } => {
            let used = gas.used();
            block_gas_used += used;
            println!("  Transfer successful, gas used: {used}");
        }
        _ => bail!("Transfer failed: {result:#?}"),
    }

    println!("\n  Block gas used (total): {block_gas_used}");

    // Read final account balances directly from the database
    println!("\n--- Final Balances ---");
    for (name, addr) in [
        ("Alice", ALICE),
        ("Bob", BOB),
        ("Charlie", CHARLIE),
        ("Coinbase", COINBASE),
    ] {
        if let Some(info) = evm.db_mut().basic(addr)? {
            println!("  {name} ({addr}): {}", format_eth(info.balance));
        }
    }

    // -------------------------------------------------------
    // Phase 2: Batch commit (transact_many_commit)
    // All transactions are executed and committed in one call.
    // State still accumulates across transactions in the batch.
    // -------------------------------------------------------
    println!("\n--- Strategy 2: Batch commit (transact_many_commit) ---\n");

    let mut cache_db = CacheDB::new(EmptyDB::default());
    for addr in [ALICE, BOB] {
        cache_db.insert_account_info(
            addr,
            AccountInfo {
                balance: initial_balance,
                ..Default::default()
            },
        );
    }

    let mut evm = Context::mainnet()
        .with_db(cache_db)
        .modify_block_chained(|block| {
            block.number = U256::from(18_000_001);
            block.timestamp = U256::from(1_700_000_012); // 12 seconds later
            block.beneficiary = COINBASE;
            block.gas_limit = 30_000_000;
            block.basefee = 10;
        })
        .build_mainnet();

    // Batch: Alice sends ETH to Bob in 3 transactions
    let txs = vec![
        TxEnv::builder()
            .caller(ALICE)
            .kind(TxKind::Call(BOB))
            .value(U256::from(1_000_000_000_000_000_000u128))
            .gas_limit(21_000)
            .gas_price(10)
            .nonce(0)
            .build()?,
        TxEnv::builder()
            .caller(ALICE)
            .kind(TxKind::Call(BOB))
            .value(U256::from(2_000_000_000_000_000_000u128))
            .gas_limit(21_000)
            .gas_price(10)
            .nonce(1)
            .build()?,
        TxEnv::builder()
            .caller(ALICE)
            .kind(TxKind::Call(BOB))
            .value(U256::from(500_000_000_000_000_000u128))
            .gas_limit(21_000)
            .gas_price(10)
            .nonce(2)
            .build()?,
    ];

    let results = evm.transact_many_commit(txs.into_iter())?;

    let mut batch_gas = 0u64;
    for (i, result) in results.iter().enumerate() {
        match result {
            ExecutionResult::Success { gas, .. } => {
                let used = gas.used();
                batch_gas += used;
                println!("  TX {}: success, gas used: {used}", i + 1);
            }
            _ => println!("  TX {}: failed", i + 1),
        }
    }
    println!("  Batch gas used (total): {batch_gas}");

    println!("\n--- Final Balances ---");
    for (name, addr) in [("Alice", ALICE), ("Bob", BOB)] {
        if let Some(info) = evm.db_mut().basic(addr)? {
            println!("  {name} ({addr}): {}", format_eth(info.balance));
        }
    }

    println!("\n=== Block Simulation Complete ===");
    Ok(())
}

/// Print the result of a counter increment call.
fn print_call_result(result: &ExecutionResult, block_gas_used: &mut u64) {
    match result {
        ExecutionResult::Success {
            output: Output::Call(bytes),
            gas,
            ..
        } => {
            let value = U256::from_be_slice(bytes);
            let used = gas.used();
            *block_gas_used += used;
            println!("  Counter value: {value}, gas used: {used}");
        }
        _ => println!("  Call failed: {result:#?}"),
    }
}

/// Format a U256 wei amount as a human-readable ETH string.
fn format_eth(wei: U256) -> String {
    let eth = wei / U256::from(1_000_000_000_000_000_000u128);
    let remainder = wei % U256::from(1_000_000_000_000_000_000u128);
    if remainder.is_zero() {
        format!("{eth} ETH")
    } else {
        format!("{eth} ETH + {remainder} wei")
    }
}
