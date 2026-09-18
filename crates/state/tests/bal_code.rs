//! Code changes with a fixed per-index baseline.

use revm_state::{
    bal::{AccountInfoBal, BalBuilder, BlockAccessIndex},
    Account, AccountInfo, Bytecode,
};

fn info(byte: u8) -> AccountInfo {
    let code = Bytecode::new_raw(vec![byte].into());
    AccountInfo {
        code_hash: code.hash_slow(),
        code: Some(code),
        ..Default::default()
    }
}

#[test]
fn code_restores_fixed_index_baseline() {
    let original = info(0x00);
    let changed = info(0x01);
    let mut bal = AccountInfoBal::default();
    let index = BlockAccessIndex::new(0);
    bal.update(index, &original, &changed);
    bal.update(index, &original, &original);
    assert!(bal.code.is_empty());
}

#[test]
fn code_read_without_loaded_bytes_preserves_recorded_code() {
    let original = info(0x00);
    let changed = info(0x01);
    let mut read = changed.clone();
    read.code = None;
    let mut bal = AccountInfoBal::default();
    let index = BlockAccessIndex::new(0);
    bal.update(index, &original, &changed);
    bal.update(index, &original, &read);
    assert_eq!(
        bal.code.writes,
        vec![(index, (changed.code_hash, changed.code.unwrap()))]
    );
}

#[test]
fn rebased_code_restoration_preserves_previous_index_bytes() {
    let original = info(0x00);
    let first = info(0x01);
    let second = info(0x02);
    let address = primitives::Address::with_last_byte(1);
    let mut builder = BalBuilder::new();
    for (index, original, present) in [
        (0, &original, &first),
        (1, &first, &second),
        (1, &second, &first),
    ] {
        let mut account = Account::from(original.clone());
        account.info = present.clone();
        builder.update_account(BlockAccessIndex::new(index), address, &account);
    }
    let mut read = Account::from(first.clone());
    read.info.code = None;
    builder.update_account(BlockAccessIndex::new(1), address, &read);
    let output = builder.into_alloy_bal();
    assert_eq!(output[0].code_changes.len(), 1);
    assert_eq!(
        output[0].code_changes[0].block_access_index,
        BlockAccessIndex::new(0)
    );
    assert_eq!(
        output[0].code_changes[0].new_code(),
        &first.code.unwrap().original_bytes()
    );
}
