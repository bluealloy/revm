//! EIP-2780 integration tests.
//!
//! Verifies the decomposed intrinsic gas model (`TX_BASE_COST` + to-based +
//! value-based) and the top-level execution charges (state gas for empty
//! recipient with value, extra cold access for EIP-7702-delegated recipients).
//!
//! Asserts are based on the gas table in the EIP. Where execution gas is
//! involved (precompile base cost, etc.) the asserts compare deltas against
//! a control run with EIP-2780 disabled, so the test focuses on the EIP's
//! incremental contribution.

use revm::{
    context::TxEnv,
    context_interface::transaction::{AccessList, AccessListItem},
    database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET},
    handler::{MainnetContext, MainnetEvm},
    primitives::{
        self as primitives, address, eip2780, eip8037, eip8037::CPSB_GLAMSTERDAM, eip8038,
        hardfork::SpecId, TxKind, U256,
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

/// Builds an EVM at AMSTERDAM where BENCH_TARGET has EIP-7702 delegation to
/// `delegation_target`. Used to test warm/cold charging of the delegation target.
fn evm_with_7702_target(delegation_target: primitives::Address) -> MainEvm {
    Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.tx_gas_limit_cap = Some(u64::MAX);
        })
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new_eip7702(
            delegation_target,
        )))
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
    // tx.sender == tx.to: only TX_BASE_COST is charged.
    assert_eq!(gas.total_gas_spent(), eip2780::TX_BASE_COST);
    assert_eq!(gas.state_gas_spent_final(), 0);
}

#[test]
fn test_eip2780_self_transfer_with_value() {
    let mut evm = evm();
    let gas = run(&mut evm, TxKind::Call(BENCH_CALLER), U256::from(1u64));
    // Self-transfer with value: still no `to`/`value` charges per spec.
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
    // Intrinsic regular: TX_BASE_COST + COLD_ACCOUNT_ACCESS + ACCOUNT_WRITE + TRANSFER_LOG_COST = 23_356.
    // Top-level state gas: STATE_BYTES_PER_NEW_ACCOUNT × CPSB = 183_600.
    let expected_regular = eip2780::TX_BASE_COST
        + eip8038::COLD_ACCOUNT_ACCESS
        + eip8038::ACCOUNT_WRITE
        + eip2780::TRANSFER_LOG_COST;
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

#[test]
fn test_eip2780_call_precompile_delta_vs_legacy() {
    // Use the identity precompile (0x04). The precompile execution cost is
    // the same regardless of intrinsic accounting, so the delta between
    // EIP-2780 and legacy isolates the intrinsic-gas change.
    let precompile = address!("0x0000000000000000000000000000000000000004");

    let mut e1 = evm();
    let g1 = run(&mut e1, TxKind::Call(precompile), U256::ZERO);

    let mut e0 = evm_no_eip2780();
    let g0 = run(&mut e0, TxKind::Call(precompile), U256::ZERO);

    // EIP-2780 intrinsic for precompile with no value is just TX_BASE_COST;
    // legacy intrinsic is 21_000. Both share the same precompile exec cost.
    assert_eq!(
        g0.total_gas_spent() - g1.total_gas_spent(),
        LEGACY_BASE - eip2780::TX_BASE_COST,
    );
}

/// Per-address access-list cost at AMSTERDAM: base (2401) + data bytes (20 × 64).
const ACCESS_LIST_ADDR_COST: u64 = eip8038::ACCESS_LIST_ADDRESS_COST + 20 * 64;

#[test]
fn test_eip2780_to_in_access_list_uses_warm_cost() {
    // When tx.to is listed in the transaction access list, EIP-2780 must charge
    // WARM_ACCESS instead of COLD_ACCOUNT_ACCESS for the recipient.
    let to = address!("0x00000000000000000000000000000000000000cc");

    let warm_tx = TxEnv::builder_for_bench()
        .tx_type(None)
        .kind(TxKind::Call(to))
        .value(U256::ZERO)
        .gas_price(0)
        .gas_limit(TX_GAS_LIMIT)
        .access_list(AccessList(vec![AccessListItem {
            address: to,
            storage_keys: vec![],
        }]))
        .build_fill();

    let mut warm_evm = evm();
    let warm_gas = *warm_evm.transact_one(warm_tx).unwrap().gas();

    // to_cost = WARM_ACCESS; access-list entry itself costs ACCESS_LIST_ADDR_COST.
    let expected = eip2780::TX_BASE_COST + eip8038::WARM_ACCESS + ACCESS_LIST_ADDR_COST;
    assert_eq!(warm_gas.total_gas_spent(), expected);

    // Cold baseline (fresh EVM, no access list): to_cost = COLD_ACCOUNT_ACCESS.
    let mut cold_evm = evm();
    let cold_gas = run(&mut cold_evm, TxKind::Call(to), U256::ZERO);
    assert_eq!(
        cold_gas.total_gas_spent(),
        eip2780::TX_BASE_COST + eip8038::COLD_ACCOUNT_ACCESS
    );
}

#[test]
fn test_eip2780_7702_delegation_target_warm_vs_cold() {
    // At depth 0, a 7702-delegated recipient causes an extra access charge for
    // its delegation target.  The charge must be COLD_ACCOUNT_ACCESS when the
    // target is cold, and WARM_ACCESS when it has been pre-warmed via the
    // access list.
    let delegation_target = address!("0x00000000000000000000000000000000000000dd");

    // BENCH_TARGET carries EIP-7702 bytecode delegating to `delegation_target`.
    let mut evm = evm_with_7702_target(delegation_target);

    // Cold case: delegation target not in access list.
    let cold_tx = TxEnv::builder_for_bench()
        .kind(TxKind::Call(BENCH_TARGET))
        .value(U256::ZERO)
        .gas_price(0)
        .gas_limit(TX_GAS_LIMIT)
        .build_fill();
    let cold_gas = *evm.transact_one(cold_tx).unwrap().gas();
    // Intrinsic: TX_BASE_COST + COLD_ACCOUNT_ACCESS (for BENCH_TARGET)
    // Depth-0 charge: COLD_ACCOUNT_ACCESS (cold delegation target)
    let cold_expected =
        eip2780::TX_BASE_COST + eip8038::COLD_ACCOUNT_ACCESS + eip8038::COLD_ACCOUNT_ACCESS;
    assert_eq!(cold_gas.total_gas_spent(), cold_expected);

    // Warm case: delegation target in access list.
    let warm_tx = TxEnv::builder_for_bench()
        .tx_type(None)
        .kind(TxKind::Call(BENCH_TARGET))
        .value(U256::ZERO)
        .gas_price(0)
        .gas_limit(TX_GAS_LIMIT)
        .access_list(AccessList(vec![AccessListItem {
            address: delegation_target,
            storage_keys: vec![],
        }]))
        .build_fill();
    let mut evm2 = evm_with_7702_target(delegation_target);
    let warm_gas = *evm2.transact_one(warm_tx).unwrap().gas();
    // Intrinsic: TX_BASE_COST + COLD_ACCOUNT_ACCESS (for BENCH_TARGET) + ACCESS_LIST_ADDR_COST
    // Depth-0 charge: WARM_ACCESS (warm delegation target)
    let warm_expected = eip2780::TX_BASE_COST
        + eip8038::COLD_ACCOUNT_ACCESS
        + ACCESS_LIST_ADDR_COST
        + eip8038::WARM_ACCESS;
    assert_eq!(warm_gas.total_gas_spent(), warm_expected);
}
