//! BAL aggregation through real database commits and canonical Alloy output.

use revm_database::{
    bal::BalDatabase,
    bytecode::Bytecode,
    primitives::{Address, U256},
    state::{
        bal::{alloy::AlloyBal, BlockAccessIndex},
        Account, AccountInfo, EvmStorageSlot, TransactionId,
    },
    Database, DatabaseCommit, InMemoryDB,
};

const ADDRESS: Address = Address::with_last_byte(1);
const SLOT: U256 = U256::ZERO;

#[derive(Clone, Copy)]
enum CommitMethod {
    Map,
    Iter,
}

impl CommitMethod {
    fn commit(self, db: &mut BalDatabase<InMemoryDB>, account: Account) {
        let changes = [(ADDRESS, account)];
        match self {
            Self::Map => db.commit(changes.into_iter().collect()),
            Self::Iter => db.commit_iter(&mut changes.into_iter()),
        }
    }

    fn storage(self, db: &mut BalDatabase<InMemoryDB>, value: Option<u64>) {
        let mut account = Account::from(db.basic(ADDRESS).unwrap().unwrap());
        let original = db.storage(ADDRESS, SLOT).unwrap();
        let mut slot = EvmStorageSlot::new(original, TransactionId::ZERO);
        if let Some(value) = value {
            slot.present_value = U256::from(value);
            account.mark_touch();
        }
        account.storage.insert(SLOT, slot);
        self.commit(db, account);
    }

    fn info(self, db: &mut BalDatabase<InMemoryDB>, present: Option<AccountInfo>) {
        let mut account = Account::from(db.basic(ADDRESS).unwrap().unwrap());
        if let Some(present) = present {
            account.info = present;
            account.mark_touch();
        }
        self.commit(db, account);
    }
}

fn database() -> BalDatabase<InMemoryDB> {
    let mut db = InMemoryDB::default();
    db.insert_account_info(ADDRESS, AccountInfo::default());
    BalDatabase::new(db).with_bal_builder()
}

fn assert_storage(bal: &AlloyBal, writes: &[(u64, u64)]) {
    assert_eq!(bal.len(), 1);
    assert_eq!(bal[0].address, ADDRESS);
    if writes.is_empty() {
        assert!(bal[0].storage_changes.is_empty(), "{bal:?}");
        assert_eq!(bal[0].storage_reads, vec![SLOT]);
    } else {
        assert!(bal[0].storage_reads.is_empty());
        assert_eq!(bal[0].storage_changes.len(), 1);
        assert_eq!(bal[0].storage_changes[0].slot, SLOT);
        assert_eq!(
            bal[0].storage_changes[0]
                .changes
                .iter()
                .map(|change| (change.block_access_index, change.new_value))
                .collect::<Vec<_>>(),
            writes
                .iter()
                .map(|&(index, value)| (BlockAccessIndex::new(index), U256::from(value)))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn restoring_first_write_to_index_start_is_a_storage_read() {
    for method in [CommitMethod::Map, CommitMethod::Iter] {
        let mut db = database();
        method.storage(&mut db, Some(42));
        assert_eq!(db.storage(ADDRESS, SLOT).unwrap(), U256::from(42));
        method.storage(&mut db, Some(0));
        assert_eq!(db.storage(ADDRESS, SLOT).unwrap(), U256::ZERO);

        let snapshot = db.bal_state.bal_builder().unwrap().into_alloy_bal();
        assert_storage(&snapshot, &[]);
        for _ in 0..3 {
            method.storage(&mut db, None);
        }
        assert_storage(&db.bal_state.take_built_alloy_bal().unwrap(), &[]);
    }
}

#[test]
fn reads_after_a_write_preserve_the_net_change() {
    for method in [CommitMethod::Map, CommitMethod::Iter] {
        let mut db = database();
        method.storage(&mut db, Some(42));
        for _ in 0..3 {
            method.storage(&mut db, None);
        }

        assert_eq!(db.storage(ADDRESS, SLOT).unwrap(), U256::from(42));
        assert_storage(&db.bal_state.take_built_alloy_bal().unwrap(), &[(0, 42)]);
    }
}

#[test]
fn repeated_commits_preserve_history_and_advance_the_baseline() {
    for method in [CommitMethod::Map, CommitMethod::Iter] {
        let mut db = database();
        method.storage(&mut db, Some(7));

        db.bump_bal_index();
        method.storage(&mut db, None);
        method.storage(&mut db, Some(42));
        method.storage(&mut db, None);
        method.storage(&mut db, Some(7));
        method.storage(&mut db, None);
        assert_storage(
            &db.bal_state.bal_builder().unwrap().into_alloy_bal(),
            &[(0, 7)],
        );

        db.bump_bal_index();
        for value in [None, None, Some(0), Some(42), None, Some(0), None] {
            method.storage(&mut db, value);
        }
        assert_eq!(db.storage(ADDRESS, SLOT).unwrap(), U256::ZERO);
        assert_storage(
            &db.bal_state.take_built_alloy_bal().unwrap(),
            &[(0, 7), (2, 0)],
        );
    }
}

#[test]
fn a_read_only_index_does_not_supply_the_next_indexes_baseline() {
    for method in [CommitMethod::Map, CommitMethod::Iter] {
        let mut db = database();
        method.storage(&mut db, None);
        db.bump_bal_index();
        method.storage(&mut db, Some(42));
        method.storage(&mut db, Some(0));
        db.bump_bal_index();
        method.storage(&mut db, Some(7));
        method.storage(&mut db, None);

        assert_storage(&db.bal_state.take_built_alloy_bal().unwrap(), &[(2, 7)]);
    }
}

#[test]
fn cloned_builders_keep_their_baselines_and_new_builders_start_fresh() {
    let mut db = database();
    CommitMethod::Map.storage(&mut db, Some(42));
    let mut cloned = db.clone();
    CommitMethod::Iter.storage(&mut cloned, Some(0));
    CommitMethod::Map.storage(&mut db, None);
    assert_storage(&cloned.bal_state.take_built_alloy_bal().unwrap(), &[]);
    assert_storage(&db.bal_state.take_built_alloy_bal().unwrap(), &[(0, 42)]);
    assert_eq!(db.bal_state.bal_index(), BlockAccessIndex::PRE_EXECUTION);
    assert!(db.bal_state.bal_builder().is_none());

    db = db.with_bal_builder();
    CommitMethod::Map.storage(&mut db, Some(0));
    CommitMethod::Iter.storage(&mut db, Some(42));
    assert_eq!(db.storage(ADDRESS, SLOT).unwrap(), U256::from(42));
    assert_storage(&db.bal_state.take_built_alloy_bal().unwrap(), &[]);
}

#[test]
fn account_fields_keep_writes_across_reads_and_remove_restored_values() {
    for method in [CommitMethod::Map, CommitMethod::Iter] {
        let mut db = database();
        let initial = db.basic(ADDRESS).unwrap().unwrap();
        let code = Bytecode::new_raw(vec![0x60, 0x00].into());
        let changed = AccountInfo::new(U256::from(42), 1, code.hash_slow(), code.clone());
        method.info(&mut db, Some(changed.clone()));
        assert_eq!(db.basic(ADDRESS).unwrap().unwrap(), changed);

        // Account reads need not load bytecode. The database still stores it by hash.
        let mut unloaded = db.basic(ADDRESS).unwrap().unwrap();
        unloaded.code = None;
        db.insert_account_info(ADDRESS, unloaded);
        method.info(&mut db, None);
        let snapshot = db.bal_state.bal_builder().unwrap().into_alloy_bal();
        assert_eq!(snapshot[0].balance_changes.len(), 1);
        assert_eq!(snapshot[0].balance_changes[0].post_balance, U256::from(42));
        assert_eq!(snapshot[0].nonce_changes.len(), 1);
        assert_eq!(snapshot[0].nonce_changes[0].new_nonce, 1);
        assert_eq!(snapshot[0].code_changes.len(), 1);
        assert_eq!(
            snapshot[0].code_changes[0].new_code(),
            &code.original_bytes()
        );

        method.info(&mut db, Some(initial.clone()));
        method.info(&mut db, None);
        assert_eq!(db.basic(ADDRESS).unwrap().unwrap(), initial);
        let bal = db.bal_state.take_built_alloy_bal().unwrap();
        assert_eq!(bal.len(), 1);
        assert_eq!(bal[0].address, ADDRESS);
        assert!(bal[0].balance_changes.is_empty());
        assert!(bal[0].nonce_changes.is_empty());
        assert!(bal[0].code_changes.is_empty());
    }
}
