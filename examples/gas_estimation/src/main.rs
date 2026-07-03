//! Example that estimates the gas limit of a transaction with a binary search,
//! the same way `eth_estimateGas` does: probe non-committing executions with
//! different gas limits until the smallest succeeding limit is found.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use anyhow::bail;
use revm::{
    bytecode::opcode,
    context::{Context, TxEnv},
    context_interface::result::{ExecutionResult, Output},
    database::CacheDB,
    database_interface::EmptyDB,
    primitives::{Bytes, TxKind},
    ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext,
};

/// The minimum amount of gas a transaction can consume.
const BASE_TX_GAS: u64 = 21_000;

/// Upper bound for the search: the per-transaction gas cap introduced by
/// EIP-7825 (2^24), enforced on the latest hardfork.
const GAS_LIMIT_CEILING: u64 = 16_777_216;

/// Copy the runtime bytecode to memory and return it.
const INIT_CODE: &[u8] = &[
    opcode::PUSH1,
    0x05, // runtime length
    opcode::PUSH1,
    0x0a, // runtime offset in the init code
    opcode::PUSH0,
    opcode::CODECOPY,
    opcode::PUSH1,
    0x05,
    opcode::PUSH0,
    opcode::RETURN,
];

/// Store 42 at slot zero, so calls pay for a cold `SSTORE`.
const RUNTIME_BYTECODE: &[u8] = &[
    opcode::PUSH1,
    0x2a,
    opcode::PUSH0,
    opcode::SSTORE,
    opcode::STOP,
];

fn main() -> anyhow::Result<()> {
    let ctx = Context::mainnet().with_db(CacheDB::<EmptyDB>::default());
    let mut evm = ctx.build_mainnet();

    // Deploy the contract we want to estimate a call against.
    let bytecode: Bytes = [INIT_CODE, RUNTIME_BYTECODE].concat().into();
    let deploy_tx = evm.transact_commit(
        TxEnv::builder()
            .kind(TxKind::Create)
            .data(bytecode)
            .build()
            .unwrap(),
    )?;
    let ExecutionResult::Success {
        output: Output::Create(_, Some(address)),
        ..
    } = deploy_tx
    else {
        bail!("Failed to create contract: {deploy_tx:#?}");
    };
    println!("Created contract at {address}");

    // Runs a call with the given gas limit without committing state, so every
    // probe starts from the same state.
    let mut execute_with_gas_limit = |gas_limit: u64| -> anyhow::Result<ExecutionResult> {
        Ok(evm
            .transact(
                TxEnv::builder()
                    .kind(TxKind::Call(address))
                    .gas_limit(gas_limit)
                    .nonce(1)
                    .build()
                    .unwrap(),
            )?
            .result)
    };

    // A reference run with plenty of gas tells us how much the call consumes.
    let reference = execute_with_gas_limit(GAS_LIMIT_CEILING)?;
    if !reference.is_success() {
        bail!("Reference call failed: {reference:#?}");
    }
    let gas_used = reference.tx_gas_used();

    // Binary search the smallest gas limit the call still succeeds with.
    let (mut lo, mut hi) = (BASE_TX_GAS, GAS_LIMIT_CEILING);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if execute_with_gas_limit(mid)?.is_success() {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }

    println!("gas used by the call:       {gas_used}");
    println!("estimated gas limit:        {hi}");
    println!("difference (EIP-150 floor): {}", hi - gas_used);

    // The estimate must cover the actual consumption, but it can exceed it:
    // the 63/64 rule reserves gas in the caller frame, so the smallest
    // working limit is not necessarily equal to the gas consumed.
    assert!(hi >= gas_used);
    Ok(())
}
