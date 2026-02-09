//! Block Access List (BAL) example demonstrating how to:
//! 1. Build a BAL by executing transactions and capturing state changes
//! 2. Use the BAL to re-execute the same transactions with pre-computed state
//!
//! BAL (EIP-7928) optimizes block execution by pre-computing state access patterns,
//! allowing parallel or optimized re-execution using the captured state versions.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::sync::Arc;

use revm::{
    bytecode::opcode,
    context::{Context, ContextTr, TxEnv},
    context_interface::result::ExecutionResult,
    database::State,
    primitives::{address, keccak256, Bytes, TxKind, U256},
    ExecuteCommitEvm, MainBuilder, MainContext,
};

fn main() -> anyhow::Result<()> {
    println!("=== Block Access List (BAL) Example ===\n");

    // Simple counter contract bytecode:
    // - Reads current value from slot 0
    // - Adds 1 to it
    // - Stores back to slot 0
    // - Returns the new value
    //
    // Bytecode: PUSH0 SLOAD PUSH1 1 ADD DUP1 PUSH0 SSTORE PUSH0 MSTORE PUSH1 32 PUSH0 RETURN
    let counter_contract: Bytes = [
        opcode::PUSH0, // Push slot 0
        opcode::SLOAD, // Load current value from slot 0
        opcode::PUSH1,
        0x01,           // Push 1
        opcode::ADD,    // Add 1 to current value
        opcode::DUP1,   // Duplicate for return
        opcode::PUSH0,  // Push slot 0
        opcode::SSTORE, // Store incremented value
        opcode::PUSH0,  // Push memory offset 0
        opcode::MSTORE, // Store result in memory
        opcode::PUSH1,
        0x20,           // Push 32 (return size)
        opcode::PUSH0,  // Push 0 (return offset)
        opcode::RETURN, // Return 32 bytes from memory
    ]
    .into();

    let caller = address!("0x1000000000000000000000000000000000000001");
    let contract_address = address!("0x2000000000000000000000000000000000000002");

    // ========================================
    // PHASE 1: Build the BAL
    // ========================================
    println!("--- Phase 1: Building BAL ---");

    let mut state_for_building = State::builder().with_bal_builder().build();

    // Give caller some balance
    state_for_building.insert_account(
        caller,
        revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u128),
            nonce: 0,
            ..Default::default()
        },
    );

    // Deploy counter contract
    let bytecode = revm::bytecode::Bytecode::new_raw(counter_contract.clone());
    state_for_building.insert_account(
        contract_address,
        revm::state::AccountInfo {
            code_hash: keccak256(&counter_contract),
            code: Some(bytecode),
            ..Default::default()
        },
    );

    let ctx = Context::mainnet().with_db(&mut state_for_building);
    let mut evm = ctx.build_mainnet();

    // Execute transaction 1: increment counter (0 -> 1)
    evm.db_mut().bump_bal_index(); // BAL index 1 for first tx
    let tx1 = TxEnv::builder()
        .caller(caller)
        .kind(TxKind::Call(contract_address))
        .gas_limit(100_000)
        .nonce(0)
        .build()
        .unwrap();

    let result1 = evm.transact_commit(tx1.clone())?;
    match &result1 {
        ExecutionResult::Success { gas, output, .. } => {
            println!(
                "  TX 1: Counter incremented (0 -> 1), gas used: {}",
                gas.gas_used
            );
            if let revm::context_interface::result::Output::Call(bytes) = output {
                let value = U256::from_be_slice(bytes);
                println!("         Returned value: {value}");
            }
        }
        _ => anyhow::bail!("TX 1 failed: {result1:?}"),
    }

    // Execute transaction 2: increment counter again (1 -> 2)
    evm.db_mut().bump_bal_index(); // BAL index 2 for second tx
    let tx2 = TxEnv::builder()
        .caller(caller)
        .kind(TxKind::Call(contract_address))
        .gas_limit(100_000)
        .nonce(1)
        .build()
        .unwrap();

    let result2 = evm.transact_commit(tx2.clone())?;
    match &result2 {
        ExecutionResult::Success { gas, output, .. } => {
            println!(
                "  TX 2: Counter incremented (1 -> 2), gas used: {}",
                gas.gas_used
            );
            if let revm::context_interface::result::Output::Call(bytes) = output {
                let value = U256::from_be_slice(bytes);
                println!("         Returned value: {value}");
            }
        }
        _ => anyhow::bail!("TX 2 failed: {result2:?}"),
    }

    // Extract the built BAL
    let bal = evm.db_mut().take_built_bal().expect("BAL should be built");

    println!("\n  Built BAL with {} accounts tracked", bal.accounts.len());
    println!();

    // Print the BAL structure showing state changes
    bal.pretty_print();

    // ========================================
    // PHASE 2: Use the BAL for re-execution
    // ========================================
    println!("\n--- Phase 2: Re-executing with BAL ---");

    // Create a new state with the BAL for reading
    let bal_arc = Arc::new(bal);
    let mut state_with_bal = State::builder().with_bal(bal_arc).build();

    // Re-insert initial state (simulating fresh execution context)
    state_with_bal.insert_account(
        caller,
        revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u128),
            nonce: 0,
            ..Default::default()
        },
    );
    let bytecode2 = revm::bytecode::Bytecode::new_raw(counter_contract.clone());
    state_with_bal.insert_account(
        contract_address,
        revm::state::AccountInfo {
            code_hash: keccak256(&counter_contract),
            code: Some(bytecode2),
            ..Default::default()
        },
    );

    let ctx2 = Context::mainnet().with_db(&mut state_with_bal);
    let mut evm2 = ctx2.build_mainnet();

    // Re-execute transaction 1 using BAL
    // The BAL provides the storage value written by TX 1 so subsequent
    // reads in TX 2 can be resolved from the BAL instead of computing
    evm2.db_mut().bump_bal_index(); // BAL index 1
    let result1_replay = evm2.transact_commit(tx1)?;
    match &result1_replay {
        ExecutionResult::Success { gas, output, .. } => {
            println!("  TX 1 replayed with BAL, gas used: {}", gas.gas_used);
            if let revm::context_interface::result::Output::Call(bytes) = output {
                let value = U256::from_be_slice(bytes);
                println!("         Returned value: {value}");
            }
        }
        _ => anyhow::bail!("TX 1 replay failed: {result1_replay:?}"),
    }

    // Re-execute transaction 2 using BAL
    evm2.db_mut().bump_bal_index(); // BAL index 2
    let result2_replay = evm2.transact_commit(tx2)?;
    match &result2_replay {
        ExecutionResult::Success { gas, output, .. } => {
            println!("  TX 2 replayed with BAL, gas used: {}", gas.gas_used);
            if let revm::context_interface::result::Output::Call(bytes) = output {
                let value = U256::from_be_slice(bytes);
                println!("         Returned value: {value}");
            }
        }
        _ => anyhow::bail!("TX 2 replay failed: {result2_replay:?}"),
    }

    println!("\n=== BAL Example Complete ===");

    Ok(())
}
