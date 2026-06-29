//! EIP-2780 integration tests.
//!
//! Verifies the decomposed intrinsic gas model (`TX_BASE_COST` + to-based +
//! value-based) and the top-level execution charges (state gas for empty
//! recipient with value, extra cold access for EIP-7702-delegated recipients).
//!
//! Self-transfers (`tx.to == sender`) are special-cased per execution-specs:
//! they pay only `TX_BASE_COST` with no `to`- or `value`-based charge. The
//! precompile zero-charge edge case from the draft is not implemented, so
//! precompile transfers are charged as regular recipients.

use revm::{
    context::TxEnv,
    database::{BenchmarkDB, BENCH_CALLER},
    handler::{MainnetContext, MainnetEvm},
    primitives::{
        address, eip2780, eip8037, eip8037::CPSB_GLAMSTERDAM, eip8038, hardfork::SpecId, TxKind,
        U256,
    },
    state::Bytecode,
    Context, ExecuteEvm, MainBuilder, MainContext,
};

type MainEvm = MainnetEvm<MainnetContext<BenchmarkDB>>;

const TX_GAS_LIMIT: u64 = 1_000_000;

/// Pre-EIP-2780 legacy intrinsic base.
const LEGACY_BASE: u64 = 21_000;

/// State gas for new-account creation under Glamsterdam CPSB (120 × 1530 = 183_600).
const STATE_BYTES_PER_NEW_ACCOUNT: u64 = eip8037::NEW_ACCOUNT_BYTES * CPSB_GLAMSTERDAM;

/// Builds an EVM at AMSTERDAM with EIP-2780 enabled.
fn evm() -> MainEvm {
    Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.tx_gas_limit_cap = Some(u64::MAX);
        })
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .build_mainnet()
}

/// Builds an EVM at AMSTERDAM with EIP-2780 disabled (legacy 21k base).
fn evm_no_eip2780() -> MainEvm {
    Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.enable_amsterdam_eip2780 = false;
            cfg.tx_gas_limit_cap = Some(u64::MAX);
        })
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .build_mainnet()
}

fn tx(kind: TxKind, value: U256) -> TxEnv {
    TxEnv::builder_for_bench()
        .kind(kind)
        .value(value)
        .gas_price(0)
        .gas_limit(TX_GAS_LIMIT)
        .build_fill()
}

fn run(evm: &mut MainEvm, kind: TxKind, value: U256) -> revm::context_interface::result::ResultGas {
    *evm.transact_one(tx(kind, value)).unwrap().gas()
}

/// Helper: regular gas spent (`total_gas_spent` minus the state-gas portion).
fn regular_spent(gas: &revm::context_interface::result::ResultGas) -> u64 {
    gas.total_gas_spent()
        .saturating_sub(gas.state_gas_spent_final())
}

#[test]
fn test_eip2780_self_transfer() {
    let mut evm = evm();
    let gas = run(&mut evm, TxKind::Call(BENCH_CALLER), U256::ZERO);
    // Self-transfer pays only TX_BASE_COST (no `to`- or `value`-based charge).
    assert_eq!(gas.total_gas_spent(), eip2780::TX_BASE_COST);
    assert_eq!(gas.state_gas_spent_final(), 0);
}

#[test]
fn test_eip2780_call_eoa_no_value() {
    let mut evm = evm();
    // Cold EOA (not a precompile, not self).
    let to = address!("0x00000000000000000000000000000000000000aa");
    let gas = run(&mut evm, TxKind::Call(to), U256::ZERO);
    // Intrinsic: TX_BASE_COST + COLD_ACCOUNT_ACCESS = 14_900.
    assert_eq!(
        gas.total_gas_spent(),
        eip2780::TX_BASE_COST + eip8038::COLD_ACCOUNT_ACCESS
    );
    assert_eq!(gas.state_gas_spent_final(), 0);
}

#[test]
fn test_eip2780_call_empty_with_value() {
    let mut evm = evm();
    let to = address!("0x00000000000000000000000000000000000000ab");
    let gas = run(&mut evm, TxKind::Call(to), U256::from(1u64));
    // Intrinsic regular: TX_BASE_COST + COLD_ACCOUNT_ACCESS + TRANSFER_LOG_COST + TX_VALUE_COST = 21_000.
    // The ACCOUNT_WRITE for creating the empty recipient is not charged as regular
    // gas (only the NEW_ACCOUNT state gas is). Top-level state gas: 183_600.
    let expected_regular = eip2780::TX_BASE_COST
        + eip8038::COLD_ACCOUNT_ACCESS
        + eip2780::TRANSFER_LOG_COST
        + eip2780::TX_VALUE_COST;
    assert_eq!(regular_spent(&gas), expected_regular);
    assert_eq!(gas.state_gas_spent_final(), STATE_BYTES_PER_NEW_ACCOUNT);
}

#[test]
fn test_eip2780_create_no_value() {
    let mut evm = evm();
    let gas = run(&mut evm, TxKind::Create, U256::ZERO);
    // Intrinsic regular: TX_BASE_COST + CREATE_ACCESS = 21_600.
    // Intrinsic state gas: STATE_BYTES_PER_NEW_ACCOUNT × CPSB = 183_600.
    let expected_regular = eip2780::TX_BASE_COST + eip8038::CREATE_ACCESS;
    assert_eq!(regular_spent(&gas), expected_regular);
    assert_eq!(gas.state_gas_spent_final(), STATE_BYTES_PER_NEW_ACCOUNT);
}

#[test]
fn test_eip2780_create_with_value() {
    let mut evm = evm();
    let gas = run(&mut evm, TxKind::Create, U256::from(1u64));
    // Intrinsic regular: TX_BASE_COST + CREATE_ACCESS + TRANSFER_LOG_COST.
    // Intrinsic state gas: STATE_BYTES_PER_NEW_ACCOUNT × CPSB.
    let expected_regular =
        eip2780::TX_BASE_COST + eip8038::CREATE_ACCESS + eip2780::TRANSFER_LOG_COST;
    assert_eq!(regular_spent(&gas), expected_regular);
    assert_eq!(gas.state_gas_spent_final(), STATE_BYTES_PER_NEW_ACCOUNT);
}

#[test]
fn test_eip2780_legacy_base_when_disabled() {
    // With EIP-2780 disabled, the legacy 21,000 stipend applies.
    let mut evm = evm_no_eip2780();
    let to = address!("0x00000000000000000000000000000000000000aa");
    let gas = run(&mut evm, TxKind::Call(to), U256::ZERO);
    assert_eq!(gas.total_gas_spent(), LEGACY_BASE);
}
