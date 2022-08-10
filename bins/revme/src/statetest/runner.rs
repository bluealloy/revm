use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::{Duration, Instant},
};

use sha3::{Digest, Keccak256};

use indicatif::ProgressBar;
use primitive_types::{H160, H256, U256};
use revm::{db::AccountState, Bytecode, CreateScheme, Env, ExecutionResult, SpecId, TransactTo};
use std::sync::atomic::Ordering;
use walkdir::{DirEntry, WalkDir};

use super::{
    merkle_trie::{log_rlp_hash, state_merkle_trie_root},
    models::{SpecName, TestSuit},
    trace::CustomPrintTracer,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TestError {
    #[error(" Test:{spec_id:?}:{id}, Root missmatched, Expected: {expect:?} got:{got:?}")]
    RootMissmatch {
        spec_id: SpecId,
        id: usize,
        got: H256,
        expect: H256,
    },
    #[error("Serde json error")]
    SerdeDeserialize(#[from] serde_json::Error),
    #[error("Internal system error")]
    SystemError,
}

pub fn find_all_json_tests(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .map(DirEntry::into_path)
        .collect::<Vec<PathBuf>>()
}

pub fn execute_test_suit(path: &Path, elapsed: &Arc<Mutex<Duration>>) -> Result<(), TestError> {
    // funky test with `bigint` value in json :) not possible to happen on mainnet and hard to parse.
    // https://github.com/ethereum/tests/issues/971
    if path.file_name() == Some(OsStr::new("ValueOverflow.json")) {
        return Ok(());
    }
    /*
    These tests are passing, but they take a lot of time to execute so we are going to skip them.
    */
    if path.file_name() == Some(OsStr::new("loopExp.json")) {
        return Ok(());
    }
    if path.file_name() == Some(OsStr::new("Call50000_sha256.json")) {
        return Ok(());
    }
    if path.file_name() == Some(OsStr::new("static_Call50000_sha256.json")) {
        return Ok(());
    }
    if path.file_name() == Some(OsStr::new("loopMul.json")) {
        return Ok(());
    }
    if path.file_name() == Some(OsStr::new("CALLBlake2f_MaxRounds.json")) {
        return Ok(());
    }
    // */
    // /*
    // Skip test where basefee/accesslist is present but it shoulnd not be supported.
    // https://github.com/ethereum/tests/blob/5b7e1ab3ffaf026d99d20b17bb30f533a2c80c8b/GeneralStateTests/stExample/eip1559.json#L130
    // It is expected to skip these tests.
    if path.file_name() == Some(OsStr::new("accessListExample.json")) {
        return Ok(());
    }
    if path.file_name() == Some(OsStr::new("basefeeExample.json")) {
        return Ok(());
    }
    if path.file_name() == Some(OsStr::new("eip1559.json")) {
        return Ok(());
    }
    //*/
    let json_reader = std::fs::read(&path).unwrap();
    let suit: TestSuit = serde_json::from_reader(&*json_reader)?;
    let skip_test_unit: HashSet<_> = vec![
        "typeTwoBerlin", //txbyte is of type 02 and we dont parse bytes for this test to fail.
        "CREATE2_HighNonce", //testing nonce > u64::MAX not really possible on mainnet.
        "CREATE_HighNonce", //testing nonce > u64::MAX not really possible on mainnet.
        "CreateTransactionHighNonce", // testing nonce >u64::MAX not really possible on mainnet. code: https://github.com/ethereum/tests/blob/5b7e1ab3ffaf026d99d20b17bb30f533a2c80c8b/BlockchainTests/GeneralStateTests/stCreateTest/CreateTransactionHighNonce.json#L74
        "CREATE2_HighNonceDelegatecall", // test with very high nonce that in revm overflows. Impossible to happen. https://github.com/bluealloy/revm/issues/28
        "doubleSelfdestructTouch",       // CHECK THIS
        "mergeTest",                     // CHECK THIS
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
        let mut database = revm::InMemoryDB::default();
        for (address, info) in unit.pre.iter() {
            let acc_info = revm::AccountInfo {
                balance: info.balance,
                code_hash: H256::from_slice(Keccak256::digest(&info.code).as_slice()), //try with dummy hash.
                code: Some(Bytecode::new_raw(info.code.clone())),
                nonce: info.nonce,
            };
            database.insert_account_info(*address, acc_info);
            // insert storage:
            for (&slot, &value) in info.storage.iter() {
                database.insert_account_storage(*address, slot, value)
            }
        }
        let mut env = Env::default();
        // cfg env. SpecId is set down the road
        env.cfg.chain_id = 1i32.into(); // for mainnet

        // block env
        env.block.number = unit.env.current_number;
        env.block.coinbase = unit.env.current_coinbase;
        env.block.timestamp = unit.env.current_timestamp;
        env.block.gas_limit = unit.env.current_gas_limit;
        env.block.basefee = unit.env.current_base_fee.unwrap_or_default();
        env.block.difficulty = unit.env.current_difficulty;

        //tx env
        env.tx.caller = *map_caller_keys
            .get(&unit.transaction.secret_key.unwrap())
            .unwrap();
        env.tx.gas_price = unit
            .transaction
            .gas_price
            .unwrap_or_else(|| unit.transaction.max_fee_per_gas.unwrap_or_default());
        env.tx.gas_priority_fee = unit.transaction.max_priority_fee_per_gas;

        // post and execution
        for (spec_name, tests) in unit.post {
            if !matches!(
                spec_name,
                SpecName::Merge | SpecName::London | SpecName::Berlin | SpecName::Istanbul
            ) {
                continue;
            }

            env.cfg.spec_id = spec_name.to_spec_id();

            for (id, test) in tests.into_iter().enumerate() {
                let gas_limit = *unit.transaction.gas_limit.get(test.indexes.gas).unwrap();
                let gas_limit = if gas_limit > U256::from(u64::MAX) {
                    u64::MAX
                } else {
                    gas_limit.as_u64()
                };
                env.tx.gas_limit = gas_limit;
                env.tx.data = unit
                    .transaction
                    .data
                    .get(test.indexes.data)
                    .unwrap()
                    .clone();
                env.tx.value = *unit.transaction.value.get(test.indexes.value).unwrap();

                let access_list = match unit.transaction.access_lists {
                    Some(ref access_list) => access_list
                        .get(test.indexes.data)
                        .cloned()
                        .flatten()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|item| {
                            (
                                item.address,
                                item.storage_keys
                                    .iter()
                                    .map(|f| U256::from_big_endian(f.as_ref()))
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .collect(),
                    None => Vec::new(),
                };
                env.tx.access_list = access_list;

                let to = match unit.transaction.to {
                    Some(add) => TransactTo::Call(add),
                    None => TransactTo::Create(CreateScheme::Create),
                };
                env.tx.transact_to = to;

                let mut database_cloned = database.clone();
                let mut evm = revm::new();
                evm.database(&mut database_cloned);
                evm.env = env.clone();
                // do the deed

                let timer = Instant::now();
                let ExecutionResult {
                    exit_reason,
                    gas_used,
                    gas_refunded,
                    logs,
                    ..
                } = evm.transact_commit();
                let timer = timer.elapsed();

                *elapsed.lock().unwrap() += timer;

                let db = evm.db().unwrap();
                let state_root = state_merkle_trie_root(
                    db.accounts
                        .iter()
                        .filter(|(_, acc)| {
                            !acc.info.is_empty() || matches!(acc.account_state, AccountState::None)
                        })
                        .map(|(k, v)| (*k, v.clone())),
                );
                let logs_root = log_rlp_hash(logs);
                if test.hash != state_root || test.logs != logs_root {
                    println!(
                        "ROOTS mismath:\nstate_root:{:?}:{:?}\nlogs_root:{:?}:{:?}",
                        test.hash, state_root, test.logs, logs_root
                    );
                    let mut database_cloned = database.clone();
                    evm.database(&mut database_cloned);
                    evm.inspect_commit(CustomPrintTracer::new());
                    let db = evm.db().unwrap();
                    println!("{:?} UNIT_TEST:{}\n", path, name);
                    println!(
                        "fail reson: {:?} {:?} UNIT_TEST:{}\n gas:{:?} ({:?} refunded)",
                        exit_reason, path, name, gas_used, gas_refunded,
                    );
                    println!("\nApplied state:{:?}\n", db);
                    println!("\nStateroot: {:?}\n", state_root);
                    return Err(TestError::RootMissmatch {
                        spec_id: env.cfg.spec_id,
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

pub fn run(test_files: Vec<PathBuf>) -> Result<(), TestError> {
    let endjob = Arc::new(AtomicBool::new(false));
    let console_bar = Arc::new(ProgressBar::new(test_files.len() as u64));
    let mut joins = Vec::new();
    let queue = Arc::new(Mutex::new((0, test_files)));
    let elapsed = Arc::new(Mutex::new(std::time::Duration::ZERO));
    for _ in 0..10 {
        let queue = queue.clone();
        let endjob = endjob.clone();
        let console_bar = console_bar.clone();
        let elapsed = elapsed.clone();

        joins.push(
            std::thread::Builder::new()
                .stack_size(50 * 1024 * 1024)
                .spawn(move || loop {
                    let (index, test_path) = {
                        let mut queue = queue.lock().unwrap();
                        if queue.1.len() <= queue.0 {
                            return Ok(());
                        }
                        let test_path = queue.1[queue.0].clone();
                        queue.0 += 1;
                        (queue.0 - 1, test_path)
                    };
                    if endjob.load(Ordering::SeqCst) {
                        return Ok(());
                    }
                    //println!("Test:{:?}\n",test_path);
                    if let Err(err) = execute_test_suit(&test_path, &elapsed) {
                        endjob.store(true, Ordering::SeqCst);
                        println!("Test[{}] named:\n{:?} failed: {}\n", index, test_path, err);
                        return Err(err);
                    }

                    //println!("TestDone:{:?}\n",test_path);
                    console_bar.inc(1);
                })
                .unwrap(),
        );
    }
    for handler in joins {
        handler.join().map_err(|_| TestError::SystemError)??;
    }
    console_bar.finish_at_current_pos();
    println!("Finished execution. Time:{:?}", elapsed.lock().unwrap());
    Ok(())
}
