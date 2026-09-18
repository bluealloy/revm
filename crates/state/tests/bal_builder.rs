//! Stateful BAL construction, baseline conventions and lifecycle regressions.

use primitives::{Address, U256};
use revm_state::{
    bal::{BalBuilder, BlockAccessIndex},
    Account, AccountInfo, Bytecode, EvmStorageSlot, TransactionId,
};

const ADDRESS: Address = Address::with_last_byte(1);

fn account(original: u64, present: u64) -> Account {
    let mut account = Account::from(AccountInfo {
        nonce: original,
        balance: U256::from(original),
        ..Default::default()
    });
    account.info.nonce = present;
    account.info.balance = U256::from(present);
    account.storage.insert(
        U256::ZERO,
        EvmStorageSlot::new_changed(
            U256::from(original),
            U256::from(present),
            TransactionId::ZERO,
        ),
    );
    account
}

#[test]
fn repeated_commits_match_index_endpoints_with_either_original_convention() {
    // Compare the BAL with an oracle of each index's initial/final values.
    // Includes reads, restoration, writes after restoration and index changes.
    for indices in [[0, 0, 0, 0, 0, 0], [0, 0, 1, 1, 2, 2]] {
        for rebased in [false, true] {
            for initial in 0..3 {
                for sequence in 0..3_u64.pow(6) {
                    let mut remaining = sequence;
                    let mut previous = initial;
                    let mut index_start = initial;
                    let mut builder = BalBuilder::new();
                    let mut endpoints = Vec::new();
                    for (step, index) in indices.into_iter().enumerate() {
                        if step == 0 || indices[step - 1] != index {
                            index_start = previous;
                        }
                        let present = remaining % 3;
                        remaining /= 3;
                        let original = if rebased { previous } else { index_start };
                        builder.update_account(
                            BlockAccessIndex::new(index),
                            ADDRESS,
                            &account(original, present),
                        );
                        if (step == 5 || indices[step + 1] != index) && index_start != present {
                            endpoints.push((BlockAccessIndex::new(index), present));
                        }
                        previous = present;
                    }
                    let bal = builder.into_alloy_bal();
                    let entry = &bal[0];
                    let nonces: Vec<_> = entry
                        .nonce_changes
                        .iter()
                        .map(|c| (c.block_access_index, c.new_nonce))
                        .collect();
                    let balances: Vec<_> = entry
                        .balance_changes
                        .iter()
                        .map(|c| (c.block_access_index, c.post_balance.to::<u64>()))
                        .collect();
                    assert_eq!(nonces, endpoints, "indices={indices:?}, rebased={rebased}, initial={initial}, sequence={sequence}");
                    assert_eq!(balances, endpoints);
                    if endpoints.is_empty() {
                        assert!(entry.storage_changes.is_empty());
                        assert_eq!(entry.storage_reads, vec![U256::ZERO]);
                    } else {
                        assert!(entry.storage_reads.is_empty());
                        assert_eq!(entry.storage_changes.len(), 1);
                        let storage: Vec<_> = entry.storage_changes[0]
                            .changes
                            .iter()
                            .map(|c| (c.block_access_index, c.new_value.to::<u64>()))
                            .collect();
                        assert_eq!(storage, endpoints);
                    }
                }
            }
        }
    }
}

#[test]
fn clone_and_clear_preserve_or_reset_baselines_with_the_output() {
    let mut builder = BalBuilder::new();
    builder.update_account(BlockAccessIndex::new(4), ADDRESS, &account(0, 42));
    let mut cloned = builder.clone();
    cloned.update_account(BlockAccessIndex::new(4), ADDRESS, &account(42, 0));
    assert!(cloned.into_alloy_bal()[0].storage_changes.is_empty());
    assert_eq!(builder.bal().accounts[&ADDRESS].nonce.writes[0].1, 42);

    builder.clear();
    assert!(builder.bal().accounts.is_empty());
    builder.update_account(BlockAccessIndex::new(0), ADDRESS, &account(7, 42));
    builder.update_account(BlockAccessIndex::new(0), ADDRESS, &account(42, 7));
    assert!(builder.into_alloy_bal()[0].storage_changes.is_empty());
}

#[test]
fn slots_first_accessed_later_in_an_index_have_their_own_baseline() {
    let mut builder = BalBuilder::new();
    let index = BlockAccessIndex::new(0);
    builder.update_account(index, ADDRESS, &account(0, 42));
    let mut later = account(42, 42);
    let slot = U256::from(1);
    later.storage.insert(
        slot,
        EvmStorageSlot::new_changed(U256::from(7), U256::from(9), TransactionId::ZERO),
    );
    builder.update_account(index, ADDRESS, &later);
    later.storage.insert(
        slot,
        EvmStorageSlot::new_changed(U256::from(9), U256::from(7), TransactionId::ZERO),
    );
    builder.update_account(index, ADDRESS, &later);
    let bal = builder.into_alloy_bal();
    assert_eq!(bal[0].storage_reads, vec![slot]);
    assert_eq!(bal[0].storage_changes.len(), 1);
    assert_eq!(bal[0].storage_changes[0].slot, U256::ZERO);
}

#[test]
fn local_selfdestruct_removes_same_index_creation_changes_but_keeps_accesses() {
    let mut builder = BalBuilder::new();
    let index = BlockAccessIndex::new(0);
    let mut created = account(0, 42);
    let code = Bytecode::new_raw(vec![0x01].into());
    created.info.code_hash = code.hash_slow();
    created.info.code = Some(code);
    builder.update_account(index, ADDRESS, &created);
    created.set_current_info_as_original();
    // A later partial execution result may omit slots visited by an earlier
    // submission at this index. Destruction still clears those known slots.
    created.storage.clear();
    created.mark_selfdestructed_locally();
    builder.update_account(index, ADDRESS, &created);
    builder.update_account(index, ADDRESS, &account(0, 0));
    let bal = builder.into_alloy_bal();
    assert_eq!(bal.len(), 1);
    assert!(bal[0].nonce_changes.is_empty());
    assert!(bal[0].balance_changes.is_empty());
    assert!(bal[0].code_changes.is_empty());
    assert!(bal[0].storage_changes.is_empty());
    assert_eq!(bal[0].storage_reads, vec![U256::ZERO]);
}

#[test]
#[should_panic(expected = "BAL indices must be nondecreasing")]
fn decreasing_indices_require_a_new_builder() {
    let mut builder = BalBuilder::new();
    builder.update_account(BlockAccessIndex::new(1), ADDRESS, &account(0, 42));
    builder.update_account(BlockAccessIndex::new(0), ADDRESS, &account(42, 7));
}

#[cfg(feature = "serde")]
#[test]
fn serialized_builder_can_resume_same_index_commits() {
    let mut builder = BalBuilder::new();
    let index = BlockAccessIndex::new(0);
    builder.update_account(index, ADDRESS, &account(0, 42));
    let json = serde_json::to_string(&builder).unwrap();
    let binary = postcard::to_allocvec(&builder).unwrap();
    for mut restored in [
        serde_json::from_str::<BalBuilder>(&json).unwrap(),
        postcard::from_bytes::<BalBuilder>(&binary).unwrap(),
    ] {
        restored.update_account(index, ADDRESS, &account(42, 0));
        let bal = restored.into_alloy_bal();
        assert!(bal[0].storage_changes.is_empty());
        assert!(bal[0].balance_changes.is_empty());
        assert!(bal[0].nonce_changes.is_empty());
        assert_eq!(bal[0].storage_reads, vec![U256::ZERO]);
    }
    // Output serialization still has only the existing BAL data model.
    let output = serde_json::to_value(builder.into_bal()).unwrap();
    assert_eq!(output.as_object().unwrap().len(), 1);
    assert!(output.get("accounts").is_some());
}
