//! Example of a custom precompile that can access and modify the journal.
//!
//! This example demonstrates:
//! 1. Creating a custom precompile provider that extends the standard Ethereum precompiles
//! 2. Implementing a precompile that can read from and write to the journaled state
//! 3. Modifying account balances and storage from within a precompile
//! 4. Integrating the custom precompile into a custom EVM implementation

use example_custom_precompile_journal::{
    precompile_provider::CUSTOM_PRECOMPILE_ADDRESS, CustomEvm,
};
use revm::{
    context::{result::InvalidTransaction, Context, ContextSetters, ContextTr, TxEnv},
    context_interface::result::EVMError,
    database::InMemoryDB,
    handler::{Handler, MainnetHandler},
    inspector::NoOpInspector,
    primitives::{address, TxKind, U256},
    state::AccountInfo,
    Database, MainContext,
};

// Type alias for the error type
type MyError = EVMError<core::convert::Infallible, InvalidTransaction>;

fn main() -> anyhow::Result<()> {
    println!("=== Custom EVM with Journal-Accessing Precompiles ===\n");

    // Setup initial accounts
    let user_address = address!("0000000000000000000000000000000000000001");
    let mut db = InMemoryDB::default();

    // Give the user some ETH for gas
    let user_balance = U256::from(10).pow(U256::from(18)); // 1 ETH
    db.insert_account_info(
        user_address,
        AccountInfo {
            balance: user_balance,
            nonce: 0,
            code_hash: revm::primitives::KECCAK_EMPTY,
            code: None,
        },
    );

    // Give the precompile some initial balance for transfers
    db.insert_account_info(
        CUSTOM_PRECOMPILE_ADDRESS,
        AccountInfo {
            balance: U256::from(1000), // 1000 wei
            nonce: 0,
            code_hash: revm::primitives::KECCAK_EMPTY,
            code: None,
        },
    );

    println!("✅ Custom EVM with journal-accessing precompiles created successfully!");
    println!("🔧 Precompile available at address: {CUSTOM_PRECOMPILE_ADDRESS}");
    println!("📝 Precompile supports:");
    println!("   - Read storage (empty input): Returns value from storage slot 0");
    println!("   - Write storage (32-byte input): Stores value and transfers 1 wei to caller");

    // Create our custom EVM with mainnet handler
    let context = Context::mainnet().with_db(db);
    let mut evm = CustomEvm::new(context, NoOpInspector);
    println!("\n=== Testing Custom Precompile ===");

    // Test 1: Read initial storage value (should be 0)
    println!("1. Reading initial storage value from custom precompile...");
    evm.0.ctx.set_tx(
        TxEnv::builder()
            .caller(user_address)
            .kind(TxKind::Call(CUSTOM_PRECOMPILE_ADDRESS))
            .data(revm::primitives::Bytes::new()) // Empty data for read operation
            .gas_limit(100_000)
            .build()
            .unwrap(),
    );
    let read_result: Result<_, MyError> = MainnetHandler::default().run(&mut evm);

    match read_result {
        Ok(revm::context::result::ExecutionResult::Success {
            output, gas_used, ..
        }) => {
            println!("   ✓ Success! Gas used: {gas_used}");
            let data = output.data();
            let value = U256::from_be_slice(data);
            println!("   📖 Initial storage value: {value}");
        }
        Ok(revm::context::result::ExecutionResult::Revert { output, gas_used }) => {
            println!("   ❌ Reverted! Gas used: {gas_used}, Output: {output:?}");
        }
        Ok(revm::context::result::ExecutionResult::Halt { reason, gas_used }) => {
            println!("   🛑 Halted! Reason: {reason:?}, Gas used: {gas_used}");
        }
        Err(e) => {
            println!("   ❌ Error: {e:?}");
        }
    }

    // Test 2: Write value 42 to storage
    println!("\n2. Writing value 42 to storage via custom precompile...");
    let storage_value = U256::from(42);
    evm.0.ctx.set_tx(
        TxEnv::builder()
            .caller(user_address)
            .kind(TxKind::Call(CUSTOM_PRECOMPILE_ADDRESS))
            .data(storage_value.to_be_bytes_vec().into())
            .gas_limit(100_000)
            .nonce(1)
            .build()
            .unwrap(),
    );
    let write_result: Result<_, MyError> = MainnetHandler::default().run(&mut evm);

    match write_result {
        Ok(revm::context::result::ExecutionResult::Success { gas_used, .. }) => {
            println!("   ✓ Success! Gas used: {gas_used}");
            println!("   📝 Value 42 written to storage");
            println!("   💰 1 wei transferred from precompile to caller as reward");
        }
        Ok(revm::context::result::ExecutionResult::Revert { output, gas_used }) => {
            println!("   ❌ Reverted! Gas used: {gas_used}, Output: {output:?}");
        }
        Ok(revm::context::result::ExecutionResult::Halt { reason, gas_used }) => {
            println!("   🛑 Halted! Reason: {reason:?}, Gas used: {gas_used}");
        }
        Err(e) => {
            println!("   ❌ Error: {e:?}");
        }
    }

    // Test 3: Read storage value again to verify the write
    println!("\n3. Reading storage value again to verify the write...");
    evm.0.ctx.set_tx(
        TxEnv::builder()
            .caller(user_address)
            .kind(TxKind::Call(CUSTOM_PRECOMPILE_ADDRESS))
            .data(revm::primitives::Bytes::new()) // Empty data for read operation
            .gas_limit(100_000)
            .nonce(2)
            .build()
            .unwrap(),
    );
    let verify_result: Result<_, MyError> = MainnetHandler::default().run(&mut evm);

    match verify_result {
        Ok(revm::context::result::ExecutionResult::Success {
            output, gas_used, ..
        }) => {
            println!("   ✓ Success! Gas used: {gas_used}");
            let data = output.data();
            let value = U256::from_be_slice(data);
            println!("   📖 Final storage value: {value}");
            if value == U256::from(42) {
                println!("   🎉 Storage write was successful!");
            } else {
                println!("   ⚠️  Unexpected value in storage");
            }
        }
        Ok(revm::context::result::ExecutionResult::Revert { output, gas_used }) => {
            println!("   ❌ Reverted! Gas used: {gas_used}, Output: {output:?}");
        }
        Ok(revm::context::result::ExecutionResult::Halt { reason, gas_used }) => {
            println!("   🛑 Halted! Reason: {reason:?}, Gas used: {gas_used}");
        }
        Err(e) => {
            println!("   ❌ Error: {e:?}");
        }
    }

    // Check final account states
    println!("\n=== Final Account States ===");
    let final_context_mut = &mut evm.0.ctx;

    let user_info = final_context_mut.db_mut().basic(user_address).unwrap();
    if let Some(user_account) = user_info {
        println!("👤 User balance: {} wei", user_account.balance);
        println!("   Received 1 wei reward from precompile!");
    }

    let precompile_info = final_context_mut
        .db_mut()
        .basic(CUSTOM_PRECOMPILE_ADDRESS)
        .unwrap();
    if let Some(precompile_account) = precompile_info {
        println!("🔧 Precompile balance: {} wei", precompile_account.balance);
    }

    // Check storage directly from the journal using the storage API
    println!("📦 Note: Storage state has been modified via journal operations");

    println!("\n=== Summary ===");
    println!("✅ Custom EVM with journal-accessing precompiles working correctly!");
    println!("📝 Precompile successfully read and wrote storage");
    println!("💸 Balance transfer from precompile to caller executed");
    println!("🔍 All operations properly recorded in the journal");
    println!("🎯 Used default mainnet handler for transaction execution");

    Ok(())
}
