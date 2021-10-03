#![feature(slice_as_chunks)]

use std::{
    error::Error,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize},
        Arc,
    },
    thread,
};

use sha3::{Digest, Keccak256};

use bytes::Bytes;
use indicatif::ProgressBar;
use primitive_types::{H160, H256, U256};
use revm::{BerlinSpec, ExitReason, GlobalEnv};
use std::sync::atomic::Ordering;
use walkdir::{DirEntry, WalkDir};

use crate::models::{SpecName, TestSuit};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TestError {
    #[error(" Test:{id}, Root missmatched, Expected: {expect:?} got:{got:?}")]
    RootMissmatch { id:usize, got: H256, expect: H256 },
    #[error("EVM returned error: {0:?}")]
    EVMReturnError(revm::ExitReason),
    #[error("Serde json error")]
    SerdeDeserialize(#[from] serde_json::Error),
}

pub fn find_all_json_tests(path: PathBuf) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .map(DirEntry::into_path)
        .collect::<Vec<PathBuf>>()
}

pub fn execute_test_suit(path: &PathBuf) -> Result<(), TestError> {
    let json_reader = std::fs::read(&path).unwrap();
    let suit: TestSuit = serde_json::from_reader(&*json_reader)?;
    for (name, unit) in suit.0.into_iter() {
        // Create database and insert cache
        let mut database = revm::StateDB::new();
        for (address, info) in unit.pre.iter() {
            let acc_info = revm::AccountInfo {
                balance: info.balance,
                code_hash: Some(H256::from_slice(Keccak256::digest(&info.code).as_slice())), //try with dummy hash.
                code: Some(info.code.clone()),
                nonce: info.nonce,
            };
            database.insert_cache(*address, acc_info);
            // insert storage:
            for (&slot, &value) in info.storage.iter() {
                database.insert_cache_storage(
                    address.clone(),
                    H256(slot.into()),
                    H256(value.into()),
                )
            }
        }

        // post and execution
        for (spec_name, tests) in unit.post {
            if !matches!(spec_name, SpecName::Berlin) {
                //TODO fix this
                continue;
            }
            let caller = H160::from_str("0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b").unwrap();
            let global_env = GlobalEnv {
                gas_price: unit.transaction.gas_price.unwrap_or_default(),
                block_number: unit.env.current_number,
                block_coinbase: unit.env.current_coinbase,
                block_timestamp: unit.env.current_timestamp,
                block_difficulty: unit.env.current_difficulty,
                block_gas_limit: unit.env.current_gas_limit,
                block_basefee: unit.env.current_base_fee,
                chain_id: 1.into(),     // TODO ?
                origin: caller.clone(), // TODO ?
            };
            for (id,test) in tests.into_iter().enumerate() {
                let mut database = database.clone();
                let gas_limit = unit
                    .transaction
                    .gas_limit
                    .get(test.indexes.gas)
                    .unwrap()
                    .clone();
                let data = unit
                    .transaction
                    .data
                    .get(test.indexes.data)
                    .unwrap()
                    .clone();
                let value = unit
                    .transaction
                    .value
                    .get(test.indexes.value)
                    .unwrap()
                    .clone();
                let access_list = match unit.transaction.access_lists {
                    Some(ref access_list) => access_list
                        .get(test.indexes.data)
                        .cloned()
                        .flatten()
                        .unwrap_or(Vec::new())
                        .into_iter()
                        .map(|item| (item.address, item.storage_keys))
                        .collect(),
                    None => Vec::new(),
                };
                let gas_limit = if gas_limit > U256::from(u64::MAX) {
                    u64::MAX
                } else {
                    gas_limit.as_u64()
                };
                let mut evm = revm::EVM::new(&mut database, global_env.clone());
                let (ret, gas, state) = if let Some(to) = unit.transaction.to {
                    let (ret, _, gas, state) = evm.call::<BerlinSpec>(
                        caller.clone(),
                        to,
                        value,
                        data,
                        gas_limit,
                        access_list,
                    );
                    (ret, gas, state)
                } else {
                    let (ret, _, gas, state) = evm.create::<BerlinSpec>(
                        caller.clone(),
                        value,
                        data,
                        revm::CreateScheme::Create,
                        gas_limit,
                        access_list,
                    );
                    (ret, gas, state)
                };
                database.apply(state);
                let state_root = database.state_root();
                if test.hash != state_root {
                    println!("is_create:{}", unit.transaction.to.is_none());
                    println!("\nApplied state:{:?}\n", database);
                    println!("\nStateroot: {:?}\n", state_root);
                    return Err(TestError::RootMissmatch {
                        id,
                        got: state_root,
                        expect: test.hash,
                    });
                }
            }
        }
    }
    Ok(())
}

pub fn run(mut test_files: Vec<PathBuf>) {
    let endjob = Arc::new(AtomicBool::new(false));
    let console_bar = Arc::new(ProgressBar::new(test_files.len() as u64));
    let mut joins = Vec::new();
    for chunk in test_files.chunks(5000) {
        let chunk = Vec::from(chunk);
        let endjob = endjob.clone();
        let console_bar = console_bar.clone();

        joins.push(
            std::thread::Builder::new()
                .stack_size(20 * 1024 * 1024)
                .spawn(move || {
                    for test in chunk {
                        if endjob.load(Ordering::SeqCst) {
                            return;
                        }
                        if let Err(err) = execute_test_suit(&test) {
                            endjob.store(true, Ordering::SeqCst);
                            println!("{:?} failed: {}", test, err);
                            return;
                        } else {
                            //println!("{:?} is okay", test);
                        }
                        console_bar.inc(1);
                    }
                })
                .unwrap(),
        );
    }
    for handler in joins {
        let _ = handler.join();
    }
    // if not error finish console bar
    if !endjob.load(Ordering::SeqCst) {
        console_bar.finish();
    }
}
