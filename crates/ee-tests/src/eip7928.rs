//! EIP-7928 block access list tests for incorporations sharing a block access index.

use alloy_eips::eip7002::{
    WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_PREDEPLOY_CODE,
};
use revm::{
    context::CfgEnv,
    database::{CacheDB, EmptyDB, State},
    database_interface::DatabaseCommitExt,
    handler::SystemCallCommitEvm,
    primitives::{address, hardfork::SpecId, Address, Bytes, U256},
    state::{bal::BlockAccessIndex, AccountInfo, Bytecode},
    Context, MainBuilder, MainContext,
};

/// Empty calldata reads slot 0, otherwise stores the calldata word in slot 0.
const SHARED: Address = address!("0x00000000000000000000000000000000005ea5ed");
const WRITER: Address = address!("0x0000000000000000000000000000000000001001");
const READER: Address = address!("0x0000000000000000000000000000000000001002");

/// Index n+1 of an empty block, shared by withdrawals and post-execution system calls.
const INDEX: BlockAccessIndex = BlockAccessIndex::new(1);

/// `CALL(gas, SHARED, 0, 0, size, 0, 0)`, storing `value` at memory 0 first when given.
fn call_shared(value: Option<u8>) -> Vec<u8> {
    let mut code = Vec::new();
    if let Some(value) = value {
        code.extend([0x60, value, 0x5f, 0x52]);
    }
    let size = if value.is_some() { 0x20 } else { 0x00 };
    code.extend([0x5f, 0x5f, 0x60, size, 0x5f, 0x5f, 0x73]);
    code.extend(SHARED.as_slice());
    code.extend([0x5a, 0xf1, 0x50, 0x00]);
    code
}

fn state() -> State<CacheDB<EmptyDB>> {
    let mut db = CacheDB::new(EmptyDB::default());
    for (address, code) in [
        (
            SHARED,
            vec![
                0x36, 0x60, 0x08, 0x57, 0x5f, 0x54, 0x50, 0x00, 0x5b, 0x5f, 0x35, 0x5f, 0x55, 0x00,
            ],
        ),
        (WRITER, call_shared(Some(1))),
        (READER, call_shared(None)),
        (
            WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS,
            WITHDRAWAL_REQUEST_PREDEPLOY_CODE.to_vec(),
        ),
    ] {
        let code = Bytecode::new_raw(code.into());
        db.insert_account_info(
            address,
            AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code),
        );
    }
    let mut state = State::builder()
        .with_database(db)
        .with_bal_builder()
        .build();
    state.set_bal_index(INDEX);
    state
}

fn system_calls(state: &mut State<CacheDB<EmptyDB>>, targets: &[Address]) {
    let mut cfg = CfgEnv::default();
    cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
    let mut evm = Context::mainnet()
        .with_cfg(cfg)
        .with_db(state)
        .build_mainnet();
    for target in targets {
        evm.system_call_commit(*target, Bytes::new()).unwrap();
    }
}

/// Two system calls at the same index: the first writes a slot, the second only reads it.
#[test]
fn test_system_call_read_keeps_earlier_storage_write_at_same_index() {
    let mut state = state();
    system_calls(&mut state, &[WRITER, READER]);

    let bal = state.take_built_bal().unwrap().into_alloy_bal();
    let shared = bal.iter().find(|a| a.address == SHARED).unwrap();
    assert!(shared.storage_reads.is_empty());
    assert_eq!(shared.storage_changes.len(), 1);
    let change = &shared.storage_changes[0];
    assert_eq!(change.slot, U256::ZERO);
    let values: Vec<_> = change
        .changes
        .iter()
        .map(|c| (c.block_access_index, c.new_value))
        .collect();
    assert_eq!(values, vec![(INDEX, U256::from(1))]);
}

/// A withdrawal to the canonical EIP-7002 predeploy, followed by its system call at the same index.
#[test]
fn test_system_call_keeps_earlier_withdrawal_credit_at_same_index() {
    let mut state = state();
    state
        .increment_balances([(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, 1_000_000_000)])
        .unwrap();
    system_calls(&mut state, &[WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS]);

    let bal = state.take_built_bal().unwrap().into_alloy_bal();
    let credited = bal
        .iter()
        .find(|a| a.address == WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS)
        .unwrap();
    let balances: Vec<_> = credited
        .balance_changes
        .iter()
        .map(|c| (c.block_access_index, c.post_balance))
        .collect();
    assert_eq!(balances, vec![(INDEX, U256::from(1_000_000_000u64))]);
}
