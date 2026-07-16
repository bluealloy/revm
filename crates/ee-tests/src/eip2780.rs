//! EIP-2780 integration tests.
//!
//! Verifies the decomposed intrinsic gas model (`TX_BASE_COST` + to-based +
//! value-based) and the runtime gas phase (ethereum/EIPs#11844): state gas for
//! an empty recipient with value, the warm/cold delegation-target access for
//! EIP-7702-delegated recipients, the conditional create-transaction
//! new-account state gas, the per-authority EIP-7702 runtime charges, and the
//! included-as-halt behavior when gas covers the intrinsic cost but not a
//! runtime charge.
//!
//! Self-transfers (`tx.to == sender`) are special-cased per execution-specs:
//! they pay only `TX_BASE_COST` with no `to`- or `value`-based charge. The
//! precompile zero-charge edge case from the draft is not implemented, so
//! precompile transfers are charged as regular recipients.

use revm::{
    context::TxEnv,
    context_interface::transaction::{
        AccessList, AccessListItem, Authorization, RecoveredAuthority, RecoveredAuthorization,
    },
    database::{BenchmarkDB, CacheDB, EmptyDB, BENCH_CALLER},
    handler::{MainnetContext, MainnetEvm},
    primitives::{
        address, eip2780, eip8037, eip8037::CPSB_GLAMSTERDAM, eip8038, hardfork::SpecId, Address,
        TxKind, KECCAK_EMPTY, U256,
    },
    state::{AccountInfo, Bytecode},
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

/// State gas for one 23-byte EIP-7702 delegation indicator (23 × 1530 = 35_190).
const STATE_BYTES_PER_AUTH_BASE: u64 = eip8037::AUTH_BASE_BYTES * CPSB_GLAMSTERDAM;

type CacheEvm = MainnetEvm<MainnetContext<CacheDB<EmptyDB>>>;

/// Builds an EVM at AMSTERDAM (EIP-2780 enabled) over a prepared [`CacheDB`].
fn evm_with_db(db: CacheDB<EmptyDB>) -> CacheEvm {
    Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.tx_gas_limit_cap = Some(u64::MAX);
        })
        .with_db(db)
        .build_mainnet()
}

fn recovered_auth(authority: Address, delegate: Address, nonce: u64) -> RecoveredAuthorization {
    RecoveredAuthorization::new_unchecked(
        Authorization {
            chain_id: U256::ZERO,
            address: delegate,
            nonce,
        },
        RecoveredAuthority::Valid(authority),
    )
}

fn tx_7702(to: Address, gas_limit: u64, auth: RecoveredAuthorization) -> TxEnv {
    let mut tx = TxEnv::builder_for_bench()
        .kind(TxKind::Call(to))
        .value(U256::ZERO)
        .gas_price(0)
        .gas_limit(gas_limit)
        .build_fill();
    tx.set_recovered_authorization(vec![auth]);
    tx.derive_tx_type().unwrap();
    tx
}

#[test]
fn test_eip2780_create_to_existing_target_no_state_gas() {
    // A create transaction whose deployment target already exists
    // (balance-only leaf) pays no new-account state gas: the charge is applied
    // at runtime, conditional on the target's existence.
    let created_address = BENCH_CALLER.create(0);
    let mut db = CacheDB::<EmptyDB>::default();
    db.insert_account_info(
        created_address,
        AccountInfo::new(U256::from(1u64), 0, KECCAK_EMPTY, Bytecode::new()),
    );
    let mut evm = evm_with_db(db);

    let result = evm.transact_one(tx(TxKind::Create, U256::ZERO)).unwrap();
    assert!(result.is_success());
    let gas = result.gas();
    assert_eq!(gas.state_gas_spent_final(), 0);
    assert_eq!(
        gas.total_gas_spent(),
        eip2780::TX_BASE_COST + eip8038::CREATE_ACCESS
    );
}

#[test]
fn test_eip2780_create_runtime_state_gas_oog_is_included() {
    // Gas covers the intrinsic cost (TX_BASE + CREATE_ACCESS = 23,000) but not
    // the runtime new-account state gas: the transaction stays valid and is
    // included as an out-of-gas halt consuming all supplied gas.
    let gas_limit = eip2780::TX_BASE_COST + eip8038::CREATE_ACCESS;
    let mut evm = evm_with_db(CacheDB::<EmptyDB>::default());

    let tx = TxEnv::builder_for_bench()
        .kind(TxKind::Create)
        .value(U256::ZERO)
        .gas_price(0)
        .gas_limit(gas_limit)
        .build_fill();
    let result = evm.transact_one(tx).unwrap();
    assert!(result.is_halt());
    assert_eq!(result.gas().total_gas_spent(), gas_limit);
}

#[test]
fn test_eip2780_call_runtime_state_gas_oog_is_included() {
    // Value transfer to a non-existent recipient funded only for the intrinsic
    // 21,000: valid, included, halts out-of-gas in the runtime phase.
    let to = address!("0x00000000000000000000000000000000000000ab");
    let gas_limit = eip2780::TX_BASE_COST
        + eip8038::COLD_ACCOUNT_ACCESS
        + eip2780::TRANSFER_LOG_COST
        + eip2780::TX_VALUE_COST;
    let mut evm = evm();

    let tx = TxEnv::builder_for_bench()
        .kind(TxKind::Call(to))
        .value(U256::from(1u64))
        .gas_price(0)
        .gas_limit(gas_limit)
        .build_fill();
    let result = evm.transact_one(tx).unwrap();
    assert!(result.is_halt());
    assert_eq!(result.gas().total_gas_spent(), gas_limit);
}

#[test]
fn test_eip2780_delegated_recipient_cold_target() {
    // The delegation-target access is a runtime charge following the standard
    // EIP-2929 warm/cold model: a cold target pays COLD_ACCOUNT_ACCESS.
    let to = address!("0x00000000000000000000000000000000000000ac");
    let target = address!("0x00000000000000000000000000000000000000bb");
    let code = Bytecode::new_eip7702(target);
    let mut db = CacheDB::<EmptyDB>::default();
    db.insert_account_info(to, AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code));
    let mut evm = evm_with_db(db);

    let gas = run_cache(&mut evm, tx(TxKind::Call(to), U256::ZERO));
    // TX_BASE + COLD (recipient, intrinsic) + COLD (delegation target, runtime)
    // = 18,000; the empty target code executes as an immediate stop.
    assert_eq!(
        gas.total_gas_spent(),
        eip2780::TX_BASE_COST + 2 * eip8038::COLD_ACCOUNT_ACCESS
    );
}

#[test]
fn test_eip2780_delegated_recipient_warm_target() {
    // A delegation target pre-warmed by the access list pays WARM_ACCESS
    // instead of COLD_ACCOUNT_ACCESS.
    let to = address!("0x00000000000000000000000000000000000000ac");
    let target = address!("0x00000000000000000000000000000000000000bb");
    let code = Bytecode::new_eip7702(target);
    let mut db = CacheDB::<EmptyDB>::default();
    db.insert_account_info(to, AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code));
    let mut evm = evm_with_db(db);

    let mut tx = tx(TxKind::Call(to), U256::ZERO);
    tx.access_list = AccessList::from(vec![AccessListItem {
        address: target,
        storage_keys: vec![],
    }]);
    tx.derive_tx_type().unwrap();
    let gas = run_cache(&mut evm, tx);
    // TX_BASE + COLD (recipient) + access-list item + WARM (delegation target).
    let access_list_item_cost = eip8038::ACCESS_LIST_ADDRESS_COST + 20 * 64;
    assert_eq!(
        gas.total_gas_spent(),
        eip2780::TX_BASE_COST
            + eip8038::COLD_ACCOUNT_ACCESS
            + access_list_item_cost
            + eip8038::WARM_ACCESS
    );
}

#[test]
fn test_eip2780_auth_new_authority_runtime_charges() {
    // A non-existent authority pays, at runtime, the new-account state gas
    // plus ACCOUNT_WRITE regular gas, and the net-new delegation bytes.
    let to = address!("0x00000000000000000000000000000000000000aa");
    let authority = address!("0x00000000000000000000000000000000000000cc");
    let delegate = address!("0x00000000000000000000000000000000000000dd");
    let mut evm = evm_with_db(CacheDB::<EmptyDB>::default());

    let result = evm
        .transact_one(tx_7702(
            to,
            TX_GAS_LIMIT,
            recovered_auth(authority, delegate, 0),
        ))
        .unwrap();
    assert!(result.is_success());
    let gas = result.gas();
    assert_eq!(
        gas.state_gas_spent_final(),
        STATE_BYTES_PER_NEW_ACCOUNT + STATE_BYTES_PER_AUTH_BASE
    );
    assert_eq!(
        regular_spent(gas),
        eip2780::TX_BASE_COST
            + eip8038::COLD_ACCOUNT_ACCESS
            + eip8038::EIP7702_PER_AUTH_BASE_REGULAR
            + eip8038::ACCOUNT_WRITE
    );
}

#[test]
fn test_eip2780_auth_existing_authority_runtime_charges() {
    // An existing authority does not pay the new-account state gas, but the
    // authorization is still the transaction's first write to its leaf, so
    // ACCOUNT_WRITE is charged (EIPs#11891) along with the net-new delegation
    // bytes.
    let to = address!("0x00000000000000000000000000000000000000aa");
    let authority = address!("0x00000000000000000000000000000000000000cc");
    let delegate = address!("0x00000000000000000000000000000000000000dd");
    let mut db = CacheDB::<EmptyDB>::default();
    db.insert_account_info(
        authority,
        AccountInfo::new(U256::from(1u64), 0, KECCAK_EMPTY, Bytecode::new()),
    );
    let mut evm = evm_with_db(db);

    let result = evm
        .transact_one(tx_7702(
            to,
            TX_GAS_LIMIT,
            recovered_auth(authority, delegate, 0),
        ))
        .unwrap();
    assert!(result.is_success());
    let gas = result.gas();
    assert_eq!(gas.state_gas_spent_final(), STATE_BYTES_PER_AUTH_BASE);
    assert_eq!(
        regular_spent(gas),
        eip2780::TX_BASE_COST
            + eip8038::COLD_ACCOUNT_ACCESS
            + eip8038::EIP7702_PER_AUTH_BASE_REGULAR
            + eip8038::ACCOUNT_WRITE
    );
}

#[test]
fn test_eip2780_auth_runtime_oog_reverts_delegation() {
    // Gas covers the intrinsic cost but not the authority's runtime charges:
    // the transaction is included as an out-of-gas halt, all gas is consumed,
    // and the applied delegation is reverted.
    let to = address!("0x00000000000000000000000000000000000000aa");
    let authority = address!("0x00000000000000000000000000000000000000cc");
    let delegate = address!("0x00000000000000000000000000000000000000dd");
    let gas_limit = eip2780::TX_BASE_COST
        + eip8038::COLD_ACCOUNT_ACCESS
        + eip8038::EIP7702_PER_AUTH_BASE_REGULAR;
    let mut evm = evm_with_db(CacheDB::<EmptyDB>::default());

    let out = evm
        .transact(tx_7702(
            to,
            gas_limit,
            recovered_auth(authority, delegate, 0),
        ))
        .unwrap();
    assert!(out.result.is_halt());
    assert_eq!(out.result.gas().total_gas_spent(), gas_limit);
    // The delegation (and the authority's nonce bump) must not survive.
    if let Some(acc) = out.state.get(&authority) {
        assert_eq!(acc.info.nonce, 0);
        assert!(acc.info.is_empty_code_hash());
    }
}

fn run_cache(evm: &mut CacheEvm, tx: TxEnv) -> revm::context_interface::result::ResultGas {
    *evm.transact_one(tx).unwrap().gas()
}

// ---------------------------------------------------------------------------
// Spilled refundable-charge coverage.
//
// The tests above run with `tx_gas_limit_cap = u64::MAX`, so the EIP-8037
// reservoir is zero and every state charge — including the EIP-2780 refundable
// first-frame charge — spills into the regular gas budget. The tests below pin
// the gas accounting of that spilled charge across the interesting first-frame
// outcomes (success, revert, halt), through the reservoir "laundering" dance
// (a child DELEGATECALL clearing a slot its parent set mints reservoir on the
// child's tracker while the parent's `state_gas_spilled` counter goes stale),
// and through the precompile-recipient path where the charge is applied to the
// precompile's own result gas. They are the regression net for any refactor of
// where the refundable charge is recorded (see the reverted "Path C" redesign).
// ---------------------------------------------------------------------------

/// Address holding the slot-clearing helper called via DELEGATECALL.
const CLEAR_HELPER: Address = address!("0x00000000000000000000000000000000000000e1");

/// Helper runtime code: `SSTORE(slot0, 0); STOP`.
fn clear_slot0_code() -> Bytecode {
    Bytecode::new_raw([0x60, 0x00, 0x60, 0x00, 0x55, 0x00].as_slice().into())
}

/// Initcode performing the reservoir-laundering dance, ending in `tail`:
/// 1. `SSTORE(0, 1)` — 0→x state charge, fully spilled (reservoir is 0).
/// 2. `DELEGATECALL(CLEAR_HELPER)` — the child clears slot 0; the 0→x→0
///    refill happens on the child's tracker (child `state_gas_spilled` = 0),
///    so the credit mints child reservoir that the parent adopts on success,
///    while the parent's own `state_gas_spilled` stays stale.
/// 3. `SSTORE(1, 1)` — drawn from the minted reservoir (no spill).
/// 4. `SSTORE(1, 0)` — refill whose remaining-vs-reservoir split probes the
///    `min(refill, state_gas_spilled)` computation against the stale counter.
/// 5. `SSTORE(2, 1)`; `SSTORE(2, 0)` — one more spill/refill round.
fn laundering_initcode(tail: &[u8]) -> Vec<u8> {
    let mut code = vec![
        0x60, 0x01, 0x60, 0x00, 0x55, // SSTORE(0, 1)
        0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0x60,
        0x00, // retSize/retOffset/argsSize/argsOffset
        0x73, // PUSH20 helper
    ];
    code.extend_from_slice(CLEAR_HELPER.as_slice());
    code.extend_from_slice(&[
        0x5a, 0xf4, 0x50, // GAS; DELEGATECALL; POP
        0x60, 0x01, 0x60, 0x01, 0x55, // SSTORE(1, 1)
        0x60, 0x00, 0x60, 0x01, 0x55, // SSTORE(1, 0)
        0x60, 0x01, 0x60, 0x02, 0x55, // SSTORE(2, 1)
        0x60, 0x00, 0x60, 0x02, 0x55, // SSTORE(2, 0)
    ]);
    code.extend_from_slice(tail);
    code
}

/// EVM over a CacheDB with the clear helper deployed and the caller funded.
fn evm_with_helper() -> CacheEvm {
    let mut db = CacheDB::<EmptyDB>::default();
    let code = clear_slot0_code();
    db.insert_account_info(
        CLEAR_HELPER,
        AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code),
    );
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::new(
            U256::from(1_000_000_000u64),
            0,
            KECCAK_EMPTY,
            Bytecode::new(),
        ),
    );
    evm_with_db(db)
}

fn create_tx(initcode: Vec<u8>) -> TxEnv {
    TxEnv::builder_for_bench()
        .kind(TxKind::Create)
        .value(U256::ZERO)
        .gas_price(0)
        .gas_limit(TX_GAS_LIMIT)
        .data(initcode.into())
        .build_fill()
}

#[test]
fn test_eip2780_create_spilled_charge_reverted_initcode() {
    // Refundable create charge fully spilled; initcode reverts immediately:
    // the charge must be rolled back (spilled portion returned as unused
    // regular gas) and reported state gas must be zero.
    let mut evm = evm_with_helper();
    let result = evm
        .transact_one(create_tx(vec![0x60, 0x00, 0x60, 0x00, 0xfd])) // PUSH1 0 PUSH1 0 REVERT
        .unwrap();
    assert!(!result.is_success());
    assert!(!result.is_halt());
    let gas = result.gas();
    assert_eq!(gas.state_gas_spent_final(), 0);
    // TX_BASE_COST + CREATE_ACCESS + initcode words + REVERT-path costs; the
    // rolled-back spilled charge is returned as unused regular gas.
    assert_eq!(gas.total_gas_spent(), 23_064);
}

#[test]
fn test_eip2780_create_spilled_charge_halted_initcode() {
    // Refundable create charge fully spilled; initcode halts (INVALID): the
    // spilled charge is rolled back to `remaining`, which the halt then
    // consumes — all gas spent, no state gas reported.
    let mut evm = evm_with_helper();
    let result = evm.transact_one(create_tx(vec![0xfe])).unwrap();
    assert!(result.is_halt());
    let gas = result.gas();
    assert_eq!(gas.state_gas_spent_final(), 0);
    assert_eq!(gas.total_gas_spent(), TX_GAS_LIMIT);
}

#[test]
fn test_eip2780_create_spilled_charge_laundering_success() {
    let mut evm = evm_with_helper();
    let result = evm
        .transact_one(create_tx(laundering_initcode(&[0x00]))) // STOP
        .unwrap();
    assert!(result.is_success());
    let gas = result.gas();
    // Only the create charge remains as state gas: all three storage slots
    // were restored to zero, so their 0->x charges were refilled.
    assert_eq!(gas.state_gas_spent_final(), STATE_BYTES_PER_NEW_ACCOUNT);
    assert_eq!(gas.total_gas_spent(), 249_563);
}

#[test]
fn test_eip2780_create_spilled_charge_laundering_revert() {
    let mut evm = evm_with_helper();
    let result = evm
        .transact_one(create_tx(laundering_initcode(&[
            0x60, 0x00, 0x60, 0x00, 0xfd, // PUSH1 0 PUSH1 0 REVERT
        ])))
        .unwrap();
    assert!(!result.is_success());
    assert!(!result.is_halt());
    let gas = result.gas();
    assert_eq!(gas.state_gas_spent_final(), 0);
    assert_eq!(gas.total_gas_spent(), 66_021);
}

#[test]
fn test_eip2780_create_spilled_charge_laundering_halt() {
    let mut evm = evm_with_helper();
    let result = evm
        .transact_one(create_tx(laundering_initcode(&[0xfe]))) // INVALID
        .unwrap();
    assert!(result.is_halt());
    let gas = result.gas();
    assert_eq!(gas.state_gas_spent_final(), 0);
    assert_eq!(gas.total_gas_spent(), TX_GAS_LIMIT);
}

fn value_call_tx(to: Address, data: Vec<u8>) -> TxEnv {
    TxEnv::builder_for_bench()
        .kind(TxKind::Call(to))
        .value(U256::from(1u64))
        .gas_price(0)
        .gas_limit(TX_GAS_LIMIT)
        .data(data.into())
        .build_fill()
}

#[test]
fn test_eip2780_precompile_recipient_value_success() {
    // Value transfer to an empty precompile account (SHA-256): the refundable
    // new-account charge fires and must be applied to the precompile's own
    // result gas (spilled, since the reservoir is 0).
    let sha256 = address!("0x0000000000000000000000000000000000000002");
    let mut evm = evm_with_helper();
    let result = evm.transact_one(value_call_tx(sha256, vec![])).unwrap();
    assert!(result.is_success());
    let gas = result.gas();
    assert_eq!(gas.state_gas_spent_final(), STATE_BYTES_PER_NEW_ACCOUNT);
    // 21_000 intrinsic + 183_600 new-account state gas (spilled) + 60 SHA-256.
    assert_eq!(gas.total_gas_spent(), 204_660);
}

#[test]
fn test_eip2780_precompile_recipient_value_failure() {
    // Value transfer to an empty precompile that errors (blake2f with an
    // invalid input length): the transfer is reverted, so the refundable
    // charge must not be reported as spent state gas.
    let blake2f = address!("0x0000000000000000000000000000000000000009");
    let mut evm = evm_with_helper();
    let result = evm.transact_one(value_call_tx(blake2f, vec![])).unwrap();
    assert!(result.is_halt());
    let gas = result.gas();
    assert_eq!(gas.state_gas_spent_final(), 0);
    assert_eq!(gas.total_gas_spent(), TX_GAS_LIMIT);
}

// ---------------------------------------------------------------------------
// Delegate bytecode must be loaded even when `Database::basic` omits it.

/// A database shaped like a node's state provider: `basic` returns account
/// info without bytecode (`code: None`); code is only served by
/// `code_by_hash`. `CacheDB` keeps code inline in `basic`, which masks any
/// path that forgets to go through the code-loading load.
struct NoCodeInBasicDB(CacheDB<EmptyDB>);

impl revm::Database for NoCodeInBasicDB {
    type Error = <CacheDB<EmptyDB> as revm::Database>::Error;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(
            revm::Database::basic(&mut self.0, address)?.map(|mut info| {
                info.code = None;
                info
            }),
        )
    }

    fn code_by_hash(&mut self, code_hash: revm::primitives::B256) -> Result<Bytecode, Self::Error> {
        revm::Database::code_by_hash(&mut self.0, code_hash)
    }

    fn storage(
        &mut self,
        address: Address,
        index: revm::primitives::StorageKey,
    ) -> Result<revm::primitives::StorageValue, Self::Error> {
        revm::Database::storage(&mut self.0, address, index)
    }

    fn block_hash(&mut self, number: u64) -> Result<revm::primitives::B256, Self::Error> {
        revm::Database::block_hash(&mut self.0, number)
    }
}

#[test]
fn test_eip2780_delegated_recipient_code_loaded_from_hash() {
    // Regression test for the glamsterdam-devnet-7 block 6371 gas divergence
    // (bug017): the first-frame delegate load metered the access correctly but
    // read the bytecode from an account load that does not fetch code. With a
    // provider-style database the delegate then executed as empty code,
    // under-counting gas by the delegated code's execution cost.
    let to = address!("0x00000000000000000000000000000000000000ac");
    let target = address!("0x00000000000000000000000000000000000000bb");
    // PUSH1 1, PUSH1 2, ADD, STOP: 9 gas of observable execution.
    let target_code = Bytecode::new_raw([0x60, 0x01, 0x60, 0x02, 0x01, 0x00].into());
    let delegation = Bytecode::new_eip7702(target);

    let mut db = CacheDB::<EmptyDB>::default();
    db.insert_account_info(
        to,
        AccountInfo::new(U256::ZERO, 1, delegation.hash_slow(), delegation),
    );
    db.insert_account_info(
        target,
        AccountInfo::new(U256::ZERO, 1, target_code.hash_slow(), target_code),
    );

    let mut evm = Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.tx_gas_limit_cap = Some(u64::MAX);
        })
        .with_db(NoCodeInBasicDB(db))
        .build_mainnet();

    let result = evm.transact_one(tx(TxKind::Call(to), U256::ZERO)).unwrap();
    assert!(result.is_success());
    // TX_BASE + COLD (recipient) + COLD (delegation target) + 9 gas executed.
    assert_eq!(
        result.gas().total_gas_spent(),
        eip2780::TX_BASE_COST + 2 * eip8038::COLD_ACCOUNT_ACCESS + 9
    );
}
