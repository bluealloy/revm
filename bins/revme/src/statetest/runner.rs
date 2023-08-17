use std::io::stdout;
use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::{Duration, Instant},
};

use super::{
    merkle_trie::{log_rlp_hash, state_merkle_trie_root},
    models::{SpecName, TestSuite},
};
use indicatif::ProgressBar;
use revm::inspectors::TracerEip3155;
use revm::primitives::keccak256;
use revm::{
    interpreter::CreateScheme,
    primitives::{
        address, b256, Bytecode, Env, ExecutionResult, HashMap, SpecId, TransactTo, B256, U256,
    },
};
use std::sync::atomic::Ordering;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Error)]
pub enum TestError {
    #[error("Test: {id} ({spec_id:?}), root mismatched, expected: {expect:?} got: {got:?}")]
    RootMismatch {
        spec_id: SpecId,
        id: usize,
        got: B256,
        expect: B256,
    },
    #[error("Serde json error: {0}")]
    SerdeDeserialize(#[from] serde_json::Error),
    #[error("Internal system error")]
    SystemError,
    #[error("Unknown private key: {private_key:?}")]
    UnknownPrivateKey { private_key: B256 },
}

pub fn find_all_json_tests(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .map(DirEntry::into_path)
        .collect::<Vec<PathBuf>>()
}

pub fn execute_test_suite(
    path: &Path,
    elapsed: &Arc<Mutex<Duration>>,
    trace: bool,
) -> Result<(), TestError> {
    let name = path.file_name().unwrap().to_str().unwrap();

    // funky test with `bigint 0x00` value in json :) not possible to happen on mainnet and require custom json parser.
    // https://github.com/ethereum/tests/issues/971
    if name == "ValueOverflow.json" {
        return Ok(());
    }

    // precompiles having storage is not possible
    if name == "RevertPrecompiledTouch_storage.json" || name == "RevertPrecompiledTouch.json" {
        return Ok(());
    }

    // txbyte is of type 02 and we dont parse tx bytes for this test to fail.
    if name == "typeTwoBerlin.json" {
        return Ok(());
    }

    // Test checks if nonce overflows. We are handling this correctly but we are not parsing exception in testsuite
    // There are more nonce overflow tests that are in internal call/create, and those tests are passing and are enabled.
    if name == "CreateTransactionHighNonce.json" {
        return Ok(());
    }

    // Need to handle Test errors
    if name == "transactionIntinsicBug.json" {
        return Ok(());
    }

    // Test check if gas price overflows, we handle this correctly but does not match tests specific exception.
    if name == "HighGasPrice.json"
        || name == "CREATE_HighNonce.json"
        || name == "CREATE_HighNonceMinus1.json"
    {
        return Ok(());
    }

    // Skip test where basefee/accesslist/diffuculty is present but it shouldn't be supported in London/Berlin/TheMerge.
    // https://github.com/ethereum/tests/blob/5b7e1ab3ffaf026d99d20b17bb30f533a2c80c8b/GeneralStateTests/stExample/eip1559.json#L130
    // It is expected to not execute these tests.
    if name == "accessListExample.json"
        || name == "basefeeExample.json"
        || name == "eip1559.json"
        || name == "mergeTest.json"
    {
        return Ok(());
    }

    // These tests are passing, but they take a lot of time to execute so we are going to skip them.
    if name == "loopExp.json"
        || name == "Call50000_sha256.json"
        || name == "static_Call50000_sha256.json"
        || name == "loopMul.json"
        || name == "CALLBlake2f_MaxRounds.json"
    {
        return Ok(());
    }

    if path.to_str().unwrap().contains("stEOF") {
        return Ok(());
    }

    let s = std::fs::read_to_string(path).unwrap();
    let suite: TestSuite = serde_json::from_str(&s)?;

    let map_caller_keys: HashMap<_, _> = [
        (
            b256!("45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8"),
            address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b"),
        ),
        (
            b256!("c85ef7d79691fe79573b1a7064c19c1a9819ebdbd1faaab1a8ec92344438aaf4"),
            address!("cd2a3d9f938e13cd947ec05abc7fe734df8dd826"),
        ),
        (
            b256!("044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d"),
            address!("82a978b3f5962a5b0957d9ee9eef472ee55b42f1"),
        ),
        (
            b256!("6a7eeac5f12b409d42028f66b0b2132535ee158cfda439e3bfdd4558e8f4bf6c"),
            address!("c9c5a15a403e41498b6f69f6f89dd9f5892d21f7"),
        ),
        (
            b256!("a95defe70ebea7804f9c3be42d20d24375e2a92b9d9666b832069c5f3cd423dd"),
            address!("3fb1cd2cd96c6d5c0b5eb3322d807b34482481d4"),
        ),
        (
            b256!("fe13266ff57000135fb9aa854bbfe455d8da85b21f626307bf3263a0c2a8e7fe"),
            address!("dcc5ba93a1ed7e045690d722f2bf460a51c61415"),
        ),
    ]
    .into();

    for (name, unit) in suite.0 {
        // Create database and insert cache
        let mut cache_state = revm::CacheState::new(false);
        for (address, info) in unit.pre {
            let acc_info = revm::primitives::AccountInfo {
                balance: info.balance,
                code_hash: keccak256(&info.code), // try with dummy hash.
                code: Some(Bytecode::new_raw(info.code.clone())),
                nonce: info.nonce,
            };
            cache_state.insert_account_with_storage(address, acc_info, info.storage.clone());
        }
        let mut env = Env::default();
        // cfg env. SpecId is set down the road
        env.cfg.chain_id = U256::from(1); // for mainnet

        // block env
        env.block.number = unit.env.current_number;
        env.block.coinbase = unit.env.current_coinbase;
        env.block.timestamp = unit.env.current_timestamp;
        env.block.gas_limit = unit.env.current_gas_limit;
        env.block.basefee = unit.env.current_base_fee.unwrap_or_default();
        env.block.difficulty = unit.env.current_difficulty;
        // after the Merge prevrandao replaces mix_hash field in block and replaced difficulty opcode in EVM.
        env.block.prevrandao = Some(unit.env.current_difficulty.to_be_bytes().into());

        //tx env
        env.tx.caller =
            if let Some(caller) = map_caller_keys.get(&unit.transaction.secret_key.unwrap()) {
                *caller
            } else {
                let private_key = unit.transaction.secret_key.unwrap();
                return Err(TestError::UnknownPrivateKey { private_key });
            };
        env.tx.gas_price = unit
            .transaction
            .gas_price
            .unwrap_or_else(|| unit.transaction.max_fee_per_gas.unwrap_or_default());
        env.tx.gas_priority_fee = unit.transaction.max_priority_fee_per_gas;

        // post and execution
        for (spec_name, tests) in unit.post {
            if matches!(
                spec_name,
                SpecName::ByzantiumToConstantinopleAt5
                    | SpecName::Constantinople
                    | SpecName::Unknown
            ) {
                continue;
            }

            env.cfg.spec_id = spec_name.to_spec_id();

            for (id, test) in tests.into_iter().enumerate() {
                env.tx.gas_limit = unit.transaction.gas_limit[test.indexes.gas].saturating_to();
                env.tx.data = unit
                    .transaction
                    .data
                    .get(test.indexes.data)
                    .unwrap()
                    .clone();
                env.tx.value = unit.transaction.value[test.indexes.value];

                let access_list = match unit.transaction.access_lists {
                    Some(ref access_list) => access_list
                        .get(test.indexes.data)
                        .and_then(Option::as_deref)
                        .unwrap_or_default()
                        .iter()
                        .map(|item| {
                            (
                                item.address,
                                item.storage_keys
                                    .iter()
                                    .map(|key| U256::from_be_bytes(key.0))
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

                let mut cache = cache_state.clone();
                cache.set_state_clear_flag(SpecId::enabled(
                    env.cfg.spec_id,
                    revm::primitives::SpecId::SPURIOUS_DRAGON,
                ));
                let mut state = revm::db::StateBuilder::default()
                    .with_cached_prestate(cache)
                    .build();
                let mut evm = revm::new();
                evm.database(&mut state);
                evm.env = env.clone();
                // do the deed

                let timer = Instant::now();

                let exec_result = if trace {
                    evm.inspect_commit(TracerEip3155::new(Box::new(stdout()), false, false))
                } else {
                    evm.transact_commit()
                };
                let timer = timer.elapsed();

                *elapsed.lock().unwrap() += timer;

                let db = evm.db().unwrap();
                let state_root = state_merkle_trie_root(db.cache.trie_account());
                let logs = match &exec_result {
                    Ok(ExecutionResult::Success { logs, .. }) => logs.clone(),
                    _ => Vec::new(),
                };
                let logs_root = log_rlp_hash(&logs);
                if test.hash != state_root || test.logs != logs_root {
                    println!(
                        "Roots did not match:\nState root: wanted {:?}, got {state_root:?}\nLogs root: wanted {:?}, got {logs_root:?}",
                        test.hash, test.logs
                    );

                    let mut cache = cache_state.clone();
                    cache.set_state_clear_flag(SpecId::enabled(
                        env.cfg.spec_id,
                        revm::primitives::SpecId::SPURIOUS_DRAGON,
                    ));
                    let mut state = revm::db::StateBuilder::default()
                        .with_cached_prestate(cache)
                        .build();
                    evm.database(&mut state);
                    let _ =
                        evm.inspect_commit(TracerEip3155::new(Box::new(stdout()), false, false));
                    let db = evm.db().unwrap();
                    println!("{path:?} UNIT_TEST:{name}\n");
                    match &exec_result {
                        Ok(ExecutionResult::Success {
                            reason,
                            gas_used,
                            gas_refunded,
                            ..
                        }) => {
                            println!("Failed reason: {reason:?} {path:?} UNIT_TEST:{name}\n gas:{gas_used:?} ({gas_refunded:?} refunded)");
                        }
                        Ok(ExecutionResult::Revert { gas_used, output }) => {
                            println!(
                                "Reverted: {output:?} {path:?} UNIT_TEST:{name}\n gas:{gas_used:?}"
                            );
                        }
                        Ok(ExecutionResult::Halt { reason, gas_used }) => {
                            println!(
                                "Halted: {reason:?} {path:?} UNIT_TEST:{name}\n gas:{gas_used:?}"
                            );
                        }
                        Err(out) => {
                            println!("Output: {out:?} {path:?} UNIT_TEST:{name}\n");
                        }
                    }
                    println!(" TEST NAME: {:?}", name);
                    println!("\nApplied state:\n{:#?}\n", db.cache);
                    println!("\nState root: {state_root:?}\n");
                    println!("env.tx: {:?}\n", env.tx);
                    println!("env.block: {:?}\n", env.block);
                    println!("env.cfg: {:?}\n", env.cfg);
                    return Err(TestError::RootMismatch {
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

pub fn run(
    test_files: Vec<PathBuf>,
    mut single_thread: bool,
    trace: bool,
) -> Result<(), TestError> {
    if trace {
        single_thread = true;
    }

    let endjob = Arc::new(AtomicBool::new(false));
    let console_bar = Arc::new(ProgressBar::new(test_files.len() as u64));
    let mut joins: Vec<std::thread::JoinHandle<Result<(), TestError>>> = Vec::new();
    let queue = Arc::new(Mutex::new((0, test_files)));
    let elapsed = Arc::new(Mutex::new(std::time::Duration::ZERO));
    let num_threads = if single_thread { 1 } else { 10 };
    for _ in 0..num_threads {
        let queue = queue.clone();
        let endjob = endjob.clone();
        let console_bar = console_bar.clone();
        let elapsed = elapsed.clone();

        let mut thread = std::thread::Builder::new();

        // Allow bigger stack in debug mode to prevent stack overflow errors
        //if cfg!(debug_assertions) {
        thread = thread.stack_size(4 * 1024 * 1024);
        //}

        joins.push(
            thread
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
                    if let Err(err) = execute_test_suite(&test_path, &elapsed, trace) {
                        endjob.store(true, Ordering::SeqCst);
                        println!("Test[{index}] named:\n{test_path:?} failed: {err}\n");
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
    console_bar.finish();
    println!("Finished execution. Time:{:?}", elapsed.lock().unwrap());
    Ok(())
}
