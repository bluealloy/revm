//! Minimal example: transfer ETH between two accounts in an in-memory EVM.
//!
//! Shows how to:
//! 1. Seed balances in a [`CacheDB`]
//! 2. Build a value-transfer [`TxEnv`]
//! 3. Commit execution and read resulting balances
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use anyhow::{bail, Context as _};
use revm::{
    context::{Context, ContextTr, TxEnv},
    context_interface::result::ExecutionResult,
    database::CacheDB,
    database_interface::EmptyDB,
    primitives::{address, TxKind, U256},
    state::AccountInfo,
    Database, ExecuteCommitEvm, MainBuilder, MainContext,
};

fn main() -> anyhow::Result<()> {
    let alice = address!("0x1000000000000000000000000000000000000001");
    let bob = address!("0x2000000000000000000000000000000000000002");

    let initial = U256::from(10).pow(U256::from(18)); // 1 ETH
    let send_amount = U256::from(10).pow(U256::from(17)); // 0.1 ETH

    let mut db = CacheDB::<EmptyDB>::default();
    db.insert_account_info(
        alice,
        AccountInfo {
            balance: initial,
            nonce: 0,
            ..Default::default()
        },
    );
    db.insert_account_info(bob, AccountInfo::default());

    let ctx = Context::mainnet().with_db(db);
    let mut evm = ctx.build_mainnet();

    println!("Alice balance before: {initial} wei");
    println!("Bob balance before:   0 wei");
    println!("Sending {send_amount} wei from Alice -> Bob");

    let result = evm.transact_commit(
        TxEnv::builder()
            .caller(alice)
            .kind(TxKind::Call(bob))
            .value(send_amount)
            .gas_limit(21_000)
            .build()
            .unwrap(),
    )?;

    let ExecutionResult::Success { gas, .. } = result else {
        bail!("transfer failed: {result:#?}");
    };

    let alice_after = evm
        .db_mut()
        .basic(alice)?
        .context("alice account missing after transfer")?;
    let bob_after = evm
        .db_mut()
        .basic(bob)?
        .context("bob account missing after transfer")?;

    println!("Gas used:             {}", gas.tx_gas_used());
    println!("Alice balance after:  {} wei", alice_after.balance);
    println!("Bob balance after:    {} wei", bob_after.balance);

    assert_eq!(
        bob_after.balance, send_amount,
        "bob should receive the full transfer value"
    );
    // With default zero gas price, only value moves; gas_used is still recorded.
    assert_eq!(
        alice_after.balance,
        initial - send_amount,
        "with default zero gas price, alice pays only the transfer value"
    );

    println!("Transfer succeeded.");
    Ok(())
}
