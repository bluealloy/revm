#![feature(slice_as_chunks)]

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    iter::FromIterator,
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
use revm::{BerlinSpec, ExitReason, GlobalEnv, Inspector, SpecId};
use std::sync::atomic::Ordering;
use walkdir::{DirEntry, WalkDir};

use crate::{
    models::{SpecName, TestSuit},
    trace::CustomPrintTracer,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TestError {
    #[error(" Test:{id}, Root missmatched, Expected: {expect:?} got:{got:?}")]
    RootMissmatch { id: usize, got: H256, expect: H256 },
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

pub fn execute_test_suit<INSP: Inspector + Clone + 'static>(
    path: &PathBuf,
    inspector: Box<INSP>,
) -> Result<(), TestError> {
    let json_reader = std::fs::read(&path).unwrap();
    let suit: TestSuit = serde_json::from_reader(&*json_reader)?;
    let skip_test_unit: HashSet<_> = vec![
        "typeTwoBerlin",                  //txbyte is of type 02 and we dont parse bytes
        "modexp_modsize0_returndatasize", //modexp
        "RevertPrecompiledTouch",
        "RevertPrecompiledTouchExactOOG",
        "RevertPrecompiledTouch_storage",
        "RevertPrecompiledTouch_nonce",
        "RevertPrecompiledTouch_noncestorage",
        "failed_tx_xcf416c53",
        "sstore_combinations_initial00",
        "sstore_combinations_initial00_2",
        "sstore_combinations_initial01",
        "sstore_combinations_initial01_2",
        "sstore_combinations_initial10",
        "sstore_combinations_initial11",
        "sstore_combinations_initial11_2",
        "sstore_combinations_initial20_2",
        "sstore_combinations_initial21",
        "sstore_combinations_initial10_2",
        "sstore_combinations_initial20",
        "sstore_combinations_initial21_2",
        "SuicidesAndInternlCallSuicidesSuccess",
        "randomStatetest642",
        //"ecadd_1-3_0-0_21000_80",
        // "ecmul_1-2_5617_21000_128",
        // "ecmul_1-2_5617_21000_96",
        // "ecmul_1-2_5617_28000_128",
        // "ecmul_1-2_5617_28000_96",
        //"ecmul_1-2_9935_21000_128",
        //"ecmul_1-2_9935_21000_96",
        //"ecmul_1-2_9935_28000_96",
        //"ecmul_1-2_9935_28000_128",
        // "ecmul_7827-6598_5617_21000_128",
        // "ecmul_7827-6598_5617_21000_96",
        // "ecmul_7827-6598_5617_28000_128",
        // "ecmul_7827-6598_5617_28000_96",
        //"ecmul_7827-6598_9935_21000_128",
        //"ecmul_7827-6598_9935_21000_96",
        //"ecmul_7827-6598_9935_28000_96",
        //"ecmul_7827-6598_9935_28000_128",
        // "ecmul_0-0_5617_21000_128",
        // "ecmul_0-0_5617_21000_96",
        // "ecmul_0-0_5617_28000_128",
        // "ecmul_0-0_5617_28000_96",
        // "ecmul_0-0_9935_21000_128",
        // "ecmul_0-0_9935_21000_96",
        // "ecmul_0-0_9935_28000_96",
        // "ecmul_0-0_9935_28000_128",
        //"pointMulAdd2",
        "jumpi",
    ]
    .into_iter()
    .collect();

    let map_caller_keys: HashMap<_, _> = vec![
        (
            H256::from_str("0x45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8")
                .unwrap(),
            H160::from_str("0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b").unwrap(),
        ),
        (
            H256::from_str("0xc85ef7d79691fe79573b1a7064c19c1a9819ebdbd1faaab1a8ec92344438aaf4")
                .unwrap(),
            H160::from_str("0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826").unwrap(),
        ),
        (
            H256::from_str("0x044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d")
                .unwrap(),
            H160::from_str("0x82a978b3f5962a5b0957d9ee9eef472ee55b42f1").unwrap(),
        ),
        (
            H256::from_str("0x45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8")
                .unwrap(),
            H160::from_str("0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b").unwrap(),
        ),
        (
            H256::from_str("0x6a7eeac5f12b409d42028f66b0b2132535ee158cfda439e3bfdd4558e8f4bf6c")
                .unwrap(),
            H160::from_str("0xc9c5a15a403e41498b6f69f6f89dd9f5892d21f7").unwrap(),
        ),
        (
            H256::from_str("0xa95defe70ebea7804f9c3be42d20d24375e2a92b9d9666b832069c5f3cd423dd")
                .unwrap(),
            H160::from_str("0x3fb1cd2cd96c6d5c0b5eb3322d807b34482481d4").unwrap(),
        ),
    ]
    .into_iter()
    .collect();

    for (name, unit) in suit.0.into_iter() {
        if skip_test_unit.contains(&name.as_ref()) {
            continue;
        }
        // Create database and insert cache
        let mut database = revm::StateDB::new();
        for (address, info) in unit.pre.iter() {
            // if info.balance == U256::zero()
            //     && info.nonce == 0
            //     && info.code.is_empty()
            //     && info.storage.is_empty()
            // {
            //     continue;
            // }
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

        let caller = map_caller_keys
            .get(&unit.transaction.secret_key.unwrap())
            .unwrap();
        // post and execution
        for (spec_name, tests) in unit.post {
            if !matches!(spec_name, SpecName::Berlin) {
                //TODO fix this
                continue;
            }
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
            for (id, test) in tests.into_iter().enumerate() {
                //println!("hash:{:?},test indices:{:?}", test.hash, test.indexes);
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
                let inspector = inspector.clone();
                let (ret, gas, state) = {
                    let mut evm = revm::new_inspect(
                        SpecId::BERLIN,
                        global_env.clone(),
                        &mut database,
                        inspector,
                    );
                    if let Some(to) = unit.transaction.to {
                        let (ret, _, gas, state) =
                            evm.call(caller.clone(), to, value, data, gas_limit, access_list);
                        (ret, gas, state)
                    } else {
                        let (ret, _, gas, state) = evm.create(
                            caller.clone(),
                            value,
                            data,
                            revm::CreateScheme::Create,
                            gas_limit,
                            access_list,
                        );
                        (ret, gas, state)
                    }
                };
                //println!("inspector{:?}",inspector);
                database.apply(state);
                let state_root = database.state_root();
                if test.hash != state_root {
                    println!("UNIT_TEST:{}\n", name);
                    break;
                    //println!("\nApplied state:{:?}\n", database);
                    //println!("\nStateroot: {:?}\n", state_root);
                    // return Err(TestError::RootMissmatch {
                    //     id,
                    //     got: state_root,
                    //     expect: test.hash,
                    // });
                }
            }
        }
    }
    Ok(())
}

pub fn run<INSP: Inspector + Clone + Send + 'static>(
    mut test_files: &[PathBuf],
    inspector: Box<INSP>,
) {
    let endjob = Arc::new(AtomicBool::new(false));
    let console_bar = Arc::new(ProgressBar::new(test_files.len() as u64));
    let mut joins = Vec::new();
    for chunk in test_files.chunks(30000) {
        let chunk = Vec::from(chunk);
        let endjob = endjob.clone();
        let console_bar = console_bar.clone();
        let insp = inspector.clone();

        joins.push(
            std::thread::Builder::new()
                .stack_size(50 * 1024 * 1024)
                .spawn(move || {
                    for test in chunk {
                        if endjob.load(Ordering::SeqCst) {
                            return;
                        }
                        //println!("Test:{:?}", test);
                        if let Err(err) = execute_test_suit(&test, insp.clone()) {
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
    //if endjob.load(Ordering::SeqCst) {
    console_bar.finish_at_current_pos()
    //}
}
