//! BAL aggregation across independently finalized and committed EVM system calls.

use bytecode::Bytecode;
use context::{Context, ContextTr};
use database::InMemoryDB;
use database_interface::{bal::BalDatabase, Database};
use primitives::{bytes, hardfork::SpecId, Address, Bytes, U256};
use revm_handler::{
    ExecuteCommitEvm, MainBuilder, MainContext, MainnetContext, MainnetEvm, SystemCallEvm,
};
use state::{bal::BlockAccessIndex, AccountInfo};

const CONTRACT: Address = Address::with_last_byte(0x42);
const SLOT: U256 = U256::ZERO;
type TestEvm = MainnetEvm<MainnetContext<BalDatabase<InMemoryDB>>>;

fn evm(spec: SpecId) -> TestEvm {
    // With calldata, SSTORE its first word into slot 0. Without calldata, SLOAD
    // slot 0 and return the value. No invocation reverts its EVM execution.
    let code = Bytecode::new_legacy(bytes!("3615600a575f355f55005b5f545f5260205ff3"));
    let mut db = InMemoryDB::default();
    db.insert_account_info(CONTRACT, AccountInfo::default().with_code(code));
    Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(spec))
        .with_db(BalDatabase::new(db).with_bal_builder())
        .build_mainnet()
}

fn system_call_and_commit(evm: &mut TestEvm, value: Option<u64>, expected_before: u64) {
    let input = value
        .map(|value| Bytes::from(U256::from(value).to_be_bytes::<32>().to_vec()))
        .unwrap_or_default();
    let expected_after = U256::from(value.unwrap_or(expected_before));
    // system_call finalizes the journal, forcing the next invocation to load
    // its original value from the database after this commit.
    let output = evm.system_call(CONTRACT, input).unwrap();
    assert!(output.result.is_success(), "{:?}", output.result);
    let slot = &output.state[&CONTRACT].storage[&SLOT];
    assert_eq!(slot.original_value, U256::from(expected_before));
    assert_eq!(slot.present_value, expected_after);
    if value.is_none() {
        assert_eq!(
            output.result.output().unwrap().as_ref(),
            expected_after.to_be_bytes::<32>()
        );
    }
    evm.commit(output.state);
    assert_eq!(
        evm.db_mut().storage(CONTRACT, SLOT).unwrap(),
        expected_after
    );
}

#[test]
fn separately_committed_system_call_read_preserves_the_write() {
    for spec in [SpecId::PRAGUE, SpecId::AMSTERDAM] {
        let mut evm = evm(spec);
        system_call_and_commit(&mut evm, Some(42), 0);
        system_call_and_commit(&mut evm, None, 42);
        assert_eq!(
            evm.db().bal_state.bal_index(),
            BlockAccessIndex::PRE_EXECUTION
        );

        let bal = evm.db_mut().bal_state.take_built_alloy_bal().unwrap();
        assert_eq!(bal.len(), 1);
        assert_eq!(bal[0].address, CONTRACT);
        assert!(bal[0].storage_reads.is_empty());
        assert_eq!(bal[0].storage_changes.len(), 1);
        let slot = &bal[0].storage_changes[0];
        assert_eq!(slot.slot, SLOT);
        assert_eq!(slot.changes.len(), 1);
        assert_eq!(
            slot.changes[0].block_access_index,
            BlockAccessIndex::PRE_EXECUTION
        );
        assert_eq!(slot.changes[0].new_value, U256::from(42));
    }
}
