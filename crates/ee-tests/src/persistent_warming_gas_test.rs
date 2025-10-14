//! Integration test for persistent warming cache gas calculation.
//!
//! This test verifies that the interpreter correctly calculates gas costs
//! based on the `is_cold` flag provided by the journal's persistent warming cache.

use revm::{
    bytecode::{opcode, Bytecode},
    context::{ContextTr, TxEnv},
    database::BenchmarkDB,
    interpreter::gas::{COLD_ACCOUNT_ACCESS_COST_ADDITIONAL, COLD_SLOAD_COST_ADDITIONAL},
    primitives::{address, Address, Bytes},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

/// Creates bytecode that performs BALANCE opcode on a target address
/// PUSH20 <address> BALANCE STOP
fn create_balance_bytecode(target: Address) -> Bytecode {
    let mut bytes = vec![opcode::PUSH20];
    bytes.extend_from_slice(target.as_slice());
    bytes.push(opcode::BALANCE);
    bytes.push(opcode::STOP);
    Bytecode::new_legacy(Bytes::from(bytes))
}

/// Creates bytecode that performs SLOAD on storage key
/// PUSH1 <key> SLOAD STOP
fn create_sload_bytecode(key: u8) -> Bytecode {
    Bytecode::new_legacy(Bytes::from(vec![
        opcode::PUSH1,
        key,
        opcode::SLOAD,
        opcode::STOP,
    ]))
}

#[test]
fn test_persistent_warming_account_access_gas() {
    // Target address to check balance of
    let target_addr = address!("0x1000000000000000000000000000000000000000");
    let bytecode = create_balance_bytecode(target_addr);

    // Test WITHOUT persistent warming (default behavior)
    {
        let mut evm = Context::mainnet()
            .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
            .build_mainnet();

        // Transaction 1
        let result1 = evm
            .transact_one(TxEnv::builder_for_bench().build_fill())
            .unwrap();
        let gas1 = result1.gas_used();

        // Transaction 2 - should also be COLD
        let result2 = evm
            .transact_one(TxEnv::builder_for_bench().nonce(1).build_fill())
            .unwrap();
        let gas2 = result2.gas_used();

        // Gas should be the same (both cold)
        assert_eq!(
            gas1, gas2,
            "Without persistent warming, both txs should use same gas (both COLD)"
        );
    }

    // Test WITH persistent warming
    {
        let mut evm = Context::mainnet()
            .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
            .build_mainnet();

        // Enable persistent warming
        evm.ctx.journal_mut().enable_persistent_warming();

        // Transaction 1 - COLD
        let result1 = evm
            .transact_one(TxEnv::builder_for_bench().build_fill())
            .unwrap();
        let gas1 = result1.gas_used();

        // Transaction 2 - WARM (persistent warming!)
        let result2 = evm
            .transact_one(TxEnv::builder_for_bench().nonce(1).build_fill())
            .unwrap();
        let gas2 = result2.gas_used();

        // Calculate savings
        let gas_saved = gas1 - gas2;
        let expected_savings = COLD_ACCOUNT_ACCESS_COST_ADDITIONAL;

        println!("\n=== Account Access Gas Test ===");
        println!("Tx1 (COLD): {} gas", gas1);
        println!("Tx2 (WARM): {} gas", gas2);
        println!("Saved:      {} gas", gas_saved);
        println!(
            "Expected:   {} gas (COLD_ACCOUNT_ACCESS_COST_ADDITIONAL)",
            expected_savings
        );

        // Verify gas savings with tolerance for base transaction costs
        assert!(
            gas_saved == expected_savings,
            "Expected {} gas savings (COLD_ACCOUNT_ACCESS_COST_ADDITIONAL), got {}",
            expected_savings,
            gas_saved
        );
    }
}

#[test]
fn test_persistent_warming_storage_access_gas() {
    let bytecode = create_sload_bytecode(1);

    // Test WITHOUT persistent warming
    {
        let mut evm = Context::mainnet()
            .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
            .build_mainnet();

        let result1 = evm
            .transact_one(TxEnv::builder_for_bench().build_fill())
            .unwrap();
        let gas1 = result1.gas_used();

        let result2 = evm
            .transact_one(TxEnv::builder_for_bench().nonce(1).build_fill())
            .unwrap();
        let gas2 = result2.gas_used();

        // Gas should be the same (both cold)
        assert_eq!(
            gas1, gas2,
            "Without persistent warming, both txs should use same gas (both COLD)"
        );
    }

    // Test WITH persistent warming
    {
        let mut evm = Context::mainnet()
            .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
            .build_mainnet();

        evm.ctx.journal_mut().enable_persistent_warming();

        // Transaction 1 - COLD
        let result1 = evm
            .transact_one(TxEnv::builder_for_bench().build_fill())
            .unwrap();
        let gas1 = result1.gas_used();

        // Transaction 2 - WARM
        let result2 = evm
            .transact_one(TxEnv::builder_for_bench().nonce(1).build_fill())
            .unwrap();
        let gas2 = result2.gas_used();

        let gas_saved = gas1 - gas2;
        let expected_savings = COLD_SLOAD_COST_ADDITIONAL;

        println!("\n=== Storage Access Gas Test ===");
        println!("Tx1 (COLD): {} gas", gas1);
        println!("Tx2 (WARM): {} gas", gas2);
        println!("Saved:      {} gas", gas_saved);
        println!(
            "Expected:   {} gas (COLD_SLOAD_COST_ADDITIONAL)",
            expected_savings
        );

        // Verify gas savings with tolerance for base transaction costs
        assert!(
            gas_saved == expected_savings,
            "Expected {} gas savings (COLD_SLOAD_COST_ADDITIONAL), got {}",
            expected_savings,
            gas_saved
        );
    }
}

#[test]
fn test_persistent_warming_combined_gas() {
    // Bytecode that does both BALANCE and SLOAD
    let target_addr = address!("0x2000000000000000000000000000000000000000");
    let mut bytes = vec![opcode::PUSH20];
    bytes.extend_from_slice(target_addr.as_slice());
    bytes.extend_from_slice(&[
        opcode::BALANCE,
        opcode::POP,
        opcode::PUSH1,
        0x05, // storage key
        opcode::SLOAD,
        opcode::POP,
        opcode::STOP,
    ]);
    let bytecode = Bytecode::new_legacy(Bytes::from(bytes));

    let mut evm = Context::mainnet()
        .with_db(BenchmarkDB::new_bytecode(bytecode))
        .build_mainnet();

    evm.ctx.journal_mut().enable_persistent_warming();

    // Transaction 1 - Both COLD
    let result1 = evm
        .transact_one(TxEnv::builder_for_bench().build_fill())
        .unwrap();
    let gas1 = result1.gas_used();

    // Transaction 2 - Both WARM
    let result2 = evm
        .transact_one(TxEnv::builder_for_bench().nonce(1).build_fill())
        .unwrap();
    let gas2 = result2.gas_used();

    let gas_saved = gas1 - gas2;
    let expected_savings = COLD_ACCOUNT_ACCESS_COST_ADDITIONAL + COLD_SLOAD_COST_ADDITIONAL;

    println!("\n=== Combined Access Gas Test ===");
    println!("Tx1 (COLD account + COLD storage): {} gas", gas1);
    println!("Tx2 (WARM account + WARM storage): {} gas", gas2);
    println!("Total saved:                        {} gas", gas_saved);
    println!(
        "Expected:                           {} gas (COLD_ACCOUNT_ACCESS_COST_ADDITIONAL + COLD_SLOAD_COST_ADDITIONAL)",
        expected_savings
    );

    // Verify combined savings
    assert!(
        gas_saved == expected_savings,
        "Expected {} gas savings (account {} + storage {}), got {}",
        expected_savings,
        COLD_ACCOUNT_ACCESS_COST_ADDITIONAL,
        COLD_SLOAD_COST_ADDITIONAL,
        gas_saved
    );

    println!(
        "✓ Persistent warming verified: {} gas saved across transactions!\n",
        gas_saved
    );
}
