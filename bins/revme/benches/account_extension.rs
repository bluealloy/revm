//! Extension-independent fixtures: this file also builds on the pre-extension revision.
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use revm::{
    context::TxEnv,
    database::{states::bundle_state::BundleRetention, BundleAccount, InMemoryDB, State},
    primitives::{hardfork::SpecId, Address, Bytes, TxKind, U256},
    state::{
        bal::{AccountBal, AccountInfoBal, Bal, BlockAccessIndex},
        Account, AccountInfo, Bytecode,
    },
    Context, ExecuteCommitEvm, MainBuilder, MainContext,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    hint::black_box,
};

fn address(i: usize) -> Address {
    Address::from_word(U256::from(i + 0x1000).into())
}

fn info(i: usize) -> AccountInfo {
    AccountInfo {
        balance: U256::from(i + 1),
        code: None,
        ..Default::default()
    }
}

fn primitives(c: &mut Criterion) {
    eprintln!(
        "layout AccountInfo={} Account={} AccountInfoBal={} AccountBal={} BundleAccount={}",
        size_of::<AccountInfo>(),
        size_of::<Account>(),
        size_of::<AccountInfoBal>(),
        size_of::<AccountBal>(),
        size_of::<BundleAccount>()
    );
    for n in [64, 4096, 65536] {
        let accounts: Vec<_> = (0..n).map(info).collect();
        c.bench_function(&format!("account/clone/{n}"), |b| {
            b.iter(|| black_box(black_box(&accounts).clone()))
        });
        c.bench_function(&format!("account/copy_without_code/{n}"), |b| {
            b.iter(|| {
                for account in black_box(&accounts) {
                    black_box(account.copy_without_code());
                }
            })
        });
        c.bench_function(&format!("account/scan_balance/{n}"), |b| {
            b.iter(|| {
                let mut sum = U256::ZERO;
                for account in black_box(&accounts) {
                    sum += account.balance;
                }
                black_box(sum)
            })
        });
    }
    for (name, account) in [("empty", AccountInfo::default()), ("funded", info(0))] {
        let other = account.clone();
        c.bench_function(&format!("account/is_empty/{name}"), |b| {
            b.iter(|| black_box(&account).is_empty())
        });
        c.bench_function(&format!("account/equal/{name}"), |b| {
            b.iter(|| black_box(&account) == black_box(&other))
        });
        c.bench_function(&format!("account/hash/{name}"), |b| {
            b.iter(|| {
                let mut h = DefaultHasher::new();
                black_box(&account).hash(&mut h);
                black_box(h.finish())
            })
        });
        c.bench_function(&format!("account/from_info/{name}"), |b| {
            b.iter(|| black_box(Account::from(black_box(&account).clone())))
        });
    }
    let accounts: Vec<_> = (0..1024).map(info).collect();
    let encoded = serde_json::to_vec(&accounts).unwrap();
    c.bench_function("account/serde_encode/1024", |b| {
        b.iter(|| black_box(serde_json::to_vec(black_box(&accounts)).unwrap()))
    });
    c.bench_function("account/serde_decode/1024", |b| {
        b.iter(|| {
            black_box(serde_json::from_slice::<Vec<AccountInfo>>(black_box(&encoded)).unwrap())
        })
    });
}

fn bal(c: &mut Criterion) {
    let original = info(0);
    for writes in [0, 1, 4, 16] {
        let mut fixture = AccountInfoBal::default();
        for i in 0..writes {
            fixture.update(
                BlockAccessIndex::new(i + 1),
                &original,
                &info(i as usize + 1),
            );
        }
        c.bench_function(&format!("bal/replay/{writes}"), |b| {
            b.iter(|| {
                let mut account = original.clone();
                black_box(&fixture)
                    .populate_account_info(BlockAccessIndex::new(writes + 1), &mut account);
                black_box(account)
            })
        });
    }
    for changed in [false, true] {
        c.bench_function(&format!("bal/update_1024/changed={changed}"), |b| {
            b.iter(|| {
                let mut bal = AccountInfoBal::default();
                for i in 0..1024 {
                    let present = if changed {
                        info(i + 1)
                    } else {
                        original.clone()
                    };
                    bal.update(
                        BlockAccessIndex::new(i as u64 + 1),
                        black_box(&original),
                        black_box(&present),
                    );
                }
                black_box(bal)
            })
        });
    }
    let fixture: Bal = (0..4096)
        .map(|i| {
            let mut a = AccountBal::default();
            a.update(BlockAccessIndex::new(1), &Account::from(info(i)));
            (address(i), a)
        })
        .collect();
    c.bench_function("bal/clone/4096", |b| {
        b.iter(|| black_box(black_box(&fixture).clone()))
    });
    c.bench_function("bal/canonical_conversion/4096", |b| {
        b.iter_batched(
            || fixture.clone(),
            |bal| black_box(bal.into_alloy_bal()),
            BatchSize::LargeInput,
        )
    });
}

fn execution(c: &mut Criterion) {
    // Same opcode count; vary address reuse and account existence independently.
    for opcode in [0x31, 0x3b, 0x3f] {
        for distinct in [false, true] {
            for empty in [false, true] {
                let mut db = InMemoryDB::default();
                db.insert_account_info(address(0), info(1_000_000));
                let mut code = Vec::new();
                for i in 0..256 {
                    let target = address(2 + if distinct { i } else { 0 });
                    if !empty {
                        db.insert_account_info(target, info(i));
                    }
                    code.push(0x73);
                    code.extend_from_slice(target.as_slice());
                    code.extend_from_slice(&[opcode, 0x50]);
                }
                code.push(0x00);
                db.insert_account_info(
                    address(1),
                    AccountInfo::default().with_code(Bytecode::new_raw(Bytes::from(code))),
                );
                let mut evm = Context::mainnet()
                    .with_db(db)
                    .modify_cfg_chained(|cfg| {
                        cfg.spec = SpecId::CANCUN;
                        cfg.disable_nonce_check = true;
                    })
                    .build_mainnet();
                let tx = TxEnv::builder()
                    .caller(address(0))
                    .kind(TxKind::Call(address(1)))
                    .gas_limit(5_000_000)
                    .build()
                    .unwrap();
                assert!(evm.transact_commit(tx.clone()).unwrap().is_success());
                c.bench_function(
                    &format!("execution/op_{opcode:x}/distinct={distinct}/empty={empty}"),
                    |b| {
                        b.iter_batched(
                            || tx.clone(),
                            |tx| black_box(evm.transact_commit(tx).unwrap()),
                            BatchSize::SmallInput,
                        )
                    },
                );
            }
        }
    }
    for payload_len in [0, 32] {
        for bal in [false, true] {
            for distinct in [false, true] {
                for storage in [false, true] {
                    let mut db = InMemoryDB::default();
                    db.insert_account_info(
                        address(0),
                        AccountInfo::from_balance(U256::MAX / U256::from(2))
                            .with_extension(Bytes::from(vec![42; payload_len])),
                    );
                    for i in 1..=256 {
                        let account = if storage {
                            // Increment slot zero, exercising original/present storage and transitions.
                            info(i).with_code(Bytecode::new_raw(Bytes::from_static(&[
                                0x5f, 0x54, 0x60, 1, 0x01, 0x5f, 0x55, 0x00,
                            ])))
                        } else {
                            info(i)
                        };
                        db.insert_account_info(
                            address(i),
                            account.with_extension(Bytes::from(vec![42; payload_len])),
                        );
                    }
                    let state = State::builder()
                        .with_database(db)
                        .with_bundle_update()
                        .with_bal_builder_if(bal)
                        .build();
                    let mut evm = Context::mainnet()
                        .with_db(state)
                        .modify_cfg_chained(|cfg| {
                            cfg.spec = SpecId::CANCUN;
                            cfg.disable_nonce_check = true;
                        })
                        .build_mainnet();
                    let txs: Vec<_> = (0..256)
                        .map(|i| {
                            TxEnv::builder()
                                .caller(address(0))
                                .kind(TxKind::Call(address(1 + if distinct { i } else { 0 })))
                                .value(U256::from(1))
                                .gas_limit(100_000)
                                .build()
                                .unwrap()
                        })
                        .collect();
                    for tx in &txs {
                        assert!(evm.transact_commit(tx.clone()).unwrap().is_success());
                    }
                    c.bench_function(
                    &format!("block/256/payload={payload_len}/bal={bal}/distinct={distinct}/storage={storage}"),
                    |b| {
                        b.iter(|| {
                            for tx in &txs {
                                black_box(evm.transact_commit(tx.clone()).unwrap());
                                evm.ctx.journaled_state.database.bump_bal_index();
                            }
                            let db = &mut evm.ctx.journaled_state.database;
                            db.merge_transitions(BundleRetention::Reverts);
                            black_box(db.take_bundle());
                            if bal {
                                black_box(db.take_built_bal());
                                db.bal_state.bal_builder = Some(Bal::new());
                                db.reset_bal_index();
                            }
                        })
                    },
                );
                }
            }
        }
    }
}

fn journal(c: &mut Criterion) {
    use revm::context::{journal::JournalInner, JournalEntry};
    let mut db = InMemoryDB::default();
    for i in 0..4096 {
        db.insert_account_info(address(i), info(i + 10000));
    }
    for n in [64, 4096] {
        c.bench_function(&format!("journal/cold_load/{n}"), |b| {
            b.iter(|| {
                let mut journal = JournalInner::<JournalEntry>::new();
                journal.cfg.spec = SpecId::CANCUN;
                for i in 0..n {
                    black_box(journal.load_account(&mut db, address(i)).unwrap());
                }
                black_box(journal)
            })
        });
        let mut journal = JournalInner::<JournalEntry>::new();
        journal.cfg.spec = SpecId::CANCUN;
        for i in 0..n {
            journal.load_account(&mut db, address(i)).unwrap();
        }
        c.bench_function(&format!("journal/warm_load/{n}"), |b| {
            b.iter(|| {
                for i in 0..n {
                    black_box(journal.load_account(&mut db, address(i)).unwrap());
                }
            })
        });
        c.bench_function(&format!("journal/transfer_revert/{n}"), |b| {
            b.iter(|| {
                let checkpoint = journal.checkpoint();
                for i in 1..n {
                    assert!(journal
                        .transfer(&mut db, address(0), address(i), U256::from(1))
                        .unwrap()
                        .is_none());
                }
                journal.checkpoint_revert(checkpoint);
                black_box(&journal);
            })
        });
    }
}

fn populated_extension(c: &mut Criterion) {
    // Serde setup keeps this source compilable on main, which ignores the unknown field.
    // Skip these PR-only cases if that happened; never compare a payload to an absent field.
    for n in [32, 256, 4096] {
        let mut json = serde_json::to_value(info(1)).unwrap();
        json["extension"] = serde_json::json!(format!("0x{}", "ab".repeat(n)));
        let account: AccountInfo = serde_json::from_value(json).unwrap();
        if serde_json::to_value(&account)
            .unwrap()
            .get("extension")
            .is_none()
        {
            return;
        }
        let mut json = serde_json::to_value(&account).unwrap();
        let other: AccountInfo = serde_json::from_value(json.clone()).unwrap();
        json["extension"] = serde_json::json!(format!("0x{}cd", "ab".repeat(n - 1)));
        let changed: AccountInfo = serde_json::from_value(json).unwrap();
        c.bench_function(&format!("populated/clone/{n}"), |b| {
            b.iter(|| black_box(black_box(&account).clone()))
        });
        c.bench_function(&format!("populated/equal_independent/{n}"), |b| {
            b.iter(|| black_box(&account) == black_box(&other))
        });
        for (name, present) in [("unchanged", &other), ("changed", &changed)] {
            c.bench_function(&format!("populated/bal_update/{name}/{n}"), |b| {
                b.iter(|| {
                    let mut bal = AccountInfoBal::default();
                    bal.update(
                        BlockAccessIndex::new(1),
                        black_box(&account),
                        black_box(present),
                    );
                    black_box(bal)
                })
            });
        }
    }
}

fn populated_accounts(c: &mut Criterion) {
    use revm::context::{journal::JournalInner, JournalEntry};
    let accounts: Vec<_> = (0..4096)
        .map(|i| info(i).with_extension(Bytes::from(vec![42; 32])))
        .collect();
    c.bench_function("populated32/account/clone/4096", |b| {
        b.iter(|| black_box(black_box(&accounts).clone()))
    });
    c.bench_function("populated32/account/copy_without_code/4096", |b| {
        b.iter(|| {
            for a in black_box(&accounts) {
                black_box(a.copy_without_code());
            }
        })
    });
    let mut db = InMemoryDB::default();
    for (i, a) in accounts.iter().enumerate() {
        db.insert_account_info(address(i), a.clone());
    }
    c.bench_function("populated32/journal/cold_load/4096", |b| {
        b.iter(|| {
            let mut journal = JournalInner::<JournalEntry>::new();
            journal.cfg.spec = SpecId::CANCUN;
            for i in 0..4096 {
                black_box(journal.load_account(&mut db, address(i)).unwrap());
            }
            black_box(journal)
        })
    });
    let mut journal = JournalInner::<JournalEntry>::new();
    journal.cfg.spec = SpecId::CANCUN;
    for i in 0..4096 {
        journal.load_account(&mut db, address(i)).unwrap();
    }
    c.bench_function("populated32/journal/warm_load/4096", |b| {
        b.iter(|| {
            for i in 0..4096 {
                black_box(journal.load_account(&mut db, address(i)).unwrap());
            }
        })
    });
    let original = &accounts[0];
    for writes in [0, 1, 4, 16] {
        let mut bal = AccountInfoBal::default();
        for i in 0..writes {
            let present = info(0).with_extension(Bytes::from(vec![i as u8; 32]));
            bal.update(BlockAccessIndex::new(i + 1), original, &present);
        }
        c.bench_function(&format!("populated32/bal/replay/{writes}"), |b| {
            b.iter(|| {
                let mut account = original.clone();
                black_box(&bal)
                    .populate_account_info(BlockAccessIndex::new(writes + 1), &mut account);
                black_box(account)
            })
        });
    }
}

fn transitions(c: &mut Criterion) {
    use revm::{
        state::{EvmState, EvmStorageSlot},
        DatabaseCommit,
    };
    for operation in ["update", "create", "destroy"] {
        for hook in [false, true] {
            c.bench_function(
                &format!("state/commit_merge_revert/{operation}/hook={hook}"),
                |b| {
                    b.iter_batched(
                        || {
                            let mut state = State::builder().with_bundle_update().build();
                            if hook {
                                state.set_state_hook(Some(Box::new(|changes: EvmState| {
                                    black_box(changes);
                                })));
                            }
                            let mut changes = EvmState::default();
                            for i in 0..256 {
                                let original = info(i);
                                let mut account = if operation == "create" {
                                    state.insert_not_existing(address(i));
                                    Account::new_not_existing(Default::default())
                                } else {
                                    state.insert_account(address(i), original.clone());
                                    Account::from(original)
                                };
                                account.mark_touch();
                                account.info.nonce += 1;
                                match operation {
                                    "create" => account.mark_created(),
                                    "destroy" => account.mark_selfdestruct(),
                                    _ => {}
                                }
                                for slot in 0..4 {
                                    account.storage.insert(
                                        U256::from(slot),
                                        EvmStorageSlot::new_changed(
                                            U256::ZERO,
                                            U256::from(1),
                                            Default::default(),
                                        ),
                                    );
                                }
                                changes.insert(address(i), account);
                            }
                            (state, changes)
                        },
                        |(mut state, changes)| {
                            state.commit(changes);
                            state.merge_transitions(BundleRetention::Reverts);
                            let mut bundle = state.take_bundle();
                            bundle.revert_latest();
                            black_box(bundle);
                        },
                        BatchSize::LargeInput,
                    );
                },
            );
        }
    }
}

criterion_group!(
    benches,
    primitives,
    bal,
    execution,
    journal,
    populated_extension,
    populated_accounts,
    transitions
);
criterion_main!(benches);
