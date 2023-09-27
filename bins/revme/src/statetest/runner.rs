use super::{
    merkle_trie::{log_rlp_hash, state_merkle_trie_root},
    models::{SpecName, TestSuite},
};
use hex_literal::hex;
use indicatif::ProgressBar;
use revm::{
    inspectors::TracerEip3155,
    interpreter::CreateScheme,
    primitives::{
        calc_excess_blob_gas, keccak256, Bytecode, Env, HashMap, SpecId, TransactTo, B160, B256,
        U256,
    },
};
use std::{
    io::stdout,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::{Duration, Instant},
};
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Error)]
#[error("Test {name} failed: {kind}")]
pub struct TestError {
    pub name: String,
    pub kind: TestErrorKind,
}

#[derive(Debug, Error)]
pub enum TestErrorKind {
    #[error("logs root mismatch: expected {expected:?}, got {got:?}")]
    LogsRootMismatch { got: B256, expected: B256 },
    #[error("state root mismatch: expected {expected:?}, got {got:?}")]
    StateRootMismatch { got: B256, expected: B256 },
    #[error("Unknown private key: {0:?}")]
    UnknownPrivateKey(B256),
    #[error("Unexpected exception: {got_exception:?} but test expects:{expected_exception:?}")]
    UnexpectedException {
        expected_exception: Option<String>,
        got_exception: Option<String>,
    },
    #[error(transparent)]
    SerdeDeserialize(#[from] serde_json::Error),
}

pub fn find_all_json_tests(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .map(DirEntry::into_path)
        .collect::<Vec<PathBuf>>()
}

fn skip_test(path: &Path) -> bool {
    let path_str = path.to_str().expect("Path is not valid UTF-8");
    let name = path.file_name().unwrap().to_str().unwrap();

    matches!(
        name,
        // funky test with `bigint 0x00` value in json :) not possible to happen on mainnet and require
        // custom json parser. https://github.com/ethereum/tests/issues/971
        | "ValueOverflow.json"

        // precompiles having storage is not possible
        | "RevertPrecompiledTouch_storage.json"
        | "RevertPrecompiledTouch.json"

        // txbyte is of type 02 and we dont parse tx bytes for this test to fail.
        | "typeTwoBerlin.json"

        // Test checks if nonce overflows. We are handling this correctly but we are not parsing
        // exception in testsuite There are more nonce overflow tests that are in internal
        // call/create, and those tests are passing and are enabled.
        | "CreateTransactionHighNonce.json"

        // Need to handle Test errors
        | "transactionIntinsicBug.json"

        // Test check if gas price overflows, we handle this correctly but does not match tests specific exception.
        | "HighGasPrice.json"
        | "CREATE_HighNonce.json"
        | "CREATE_HighNonceMinus1.json"

        // Skip test where basefee/accesslist/difficulty is present but it shouldn't be supported in
        // London/Berlin/TheMerge. https://github.com/ethereum/tests/blob/5b7e1ab3ffaf026d99d20b17bb30f533a2c80c8b/GeneralStateTests/stExample/eip1559.json#L130
        // It is expected to not execute these tests.
        | "accessListExample.json"
        | "basefeeExample.json"
        | "eip1559.json"
        | "mergeTest.json"

        // These tests are passing, but they take a lot of time to execute so we are going to skip them.
        | "loopExp.json"
        | "Call50000_sha256.json"
        | "static_Call50000_sha256.json"
        | "loopMul.json"
        | "CALLBlake2f_MaxRounds.json"
        | "shiftCombinations.json"
    ) || path_str.contains("stEOF")
}

pub fn execute_test_suite(
    path: &Path,
    elapsed: &Arc<Mutex<Duration>>,
    trace: bool,
) -> Result<(), TestError> {
    if skip_test(path) {
        return Ok(());
    }

    let s = std::fs::read_to_string(path).unwrap();
    let suite: TestSuite = serde_json::from_str(&s).map_err(|e| TestError {
        name: path.to_string_lossy().into_owned(),
        kind: e.into(),
    })?;

    let map_caller_keys: HashMap<_, _> = [
        (
            B256(hex!(
                "45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8"
            )),
            B160(hex!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b")),
        ),
        (
            B256(hex!(
                "c85ef7d79691fe79573b1a7064c19c1a9819ebdbd1faaab1a8ec92344438aaf4"
            )),
            B160(hex!("cd2a3d9f938e13cd947ec05abc7fe734df8dd826")),
        ),
        (
            B256(hex!(
                "044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d"
            )),
            B160(hex!("82a978b3f5962a5b0957d9ee9eef472ee55b42f1")),
        ),
        (
            B256(hex!(
                "6a7eeac5f12b409d42028f66b0b2132535ee158cfda439e3bfdd4558e8f4bf6c"
            )),
            B160(hex!("c9c5a15a403e41498b6f69f6f89dd9f5892d21f7")),
        ),
        (
            B256(hex!(
                "a95defe70ebea7804f9c3be42d20d24375e2a92b9d9666b832069c5f3cd423dd"
            )),
            B160(hex!("3fb1cd2cd96c6d5c0b5eb3322d807b34482481d4")),
        ),
        (
            B256(hex!(
                "fe13266ff57000135fb9aa854bbfe455d8da85b21f626307bf3263a0c2a8e7fe"
            )),
            B160(hex!("dcc5ba93a1ed7e045690d722f2bf460a51c61415")),
        ),
    ]
    .into();

    for (name, unit) in suite.0 {
        // Create database and insert cache
        let mut cache_state = revm::CacheState::new(false);
        for (address, info) in unit.pre {
            let acc_info = revm::primitives::AccountInfo {
                balance: info.balance,
                code_hash: keccak256(&info.code),
                code: Some(Bytecode::new_raw(info.code)),
                nonce: info.nonce,
            };
            cache_state.insert_account_with_storage(address, acc_info, info.storage);
        }

        let mut env = Env::default();
        // for mainnet
        env.cfg.chain_id = 1;
        // env.cfg.spec_id is set down the road

        // block env
        env.block.number = unit.env.current_number;
        env.block.coinbase = unit.env.current_coinbase;
        env.block.timestamp = unit.env.current_timestamp;
        env.block.gas_limit = unit.env.current_gas_limit;
        env.block.basefee = unit.env.current_base_fee.unwrap_or_default();
        env.block.difficulty = unit.env.current_difficulty;
        // after the Merge prevrandao replaces mix_hash field in block and replaced difficulty opcode in EVM.
        env.block.prevrandao = Some(unit.env.current_difficulty.to_be_bytes().into());
        // EIP-4844
        if let (Some(parent_blob_gas_used), Some(parent_excess_blob_gas)) = (
            unit.env.parent_blob_gas_used,
            unit.env.parent_excess_blob_gas,
        ) {
            env.block
                .set_blob_excess_gas_and_price(calc_excess_blob_gas(
                    parent_blob_gas_used.to(),
                    parent_excess_blob_gas.to(),
                ));
        }

        // tx env
        let pk = unit.transaction.secret_key;
        env.tx.caller = map_caller_keys.get(&pk).copied().ok_or_else(|| TestError {
            name: name.clone(),
            kind: TestErrorKind::UnknownPrivateKey(pk),
        })?;
        env.tx.gas_price = unit
            .transaction
            .gas_price
            .or(unit.transaction.max_fee_per_gas)
            .unwrap_or_default();
        env.tx.gas_priority_fee = unit.transaction.max_priority_fee_per_gas;
        // EIP-4844
        env.tx.blob_hashes = unit.transaction.blob_versioned_hashes;
        env.tx.max_fee_per_blob_gas = unit.transaction.max_fee_per_blob_gas;

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

            for (index, test) in tests.into_iter().enumerate() {
                let gas_limit = *unit.transaction.gas_limit.get(test.indexes.gas).unwrap();
                let gas_limit = u64::try_from(gas_limit).unwrap_or(u64::MAX);
                env.tx.gas_limit = gas_limit;
                env.tx.data = unit
                    .transaction
                    .data
                    .get(test.indexes.data)
                    .unwrap()
                    .clone();
                env.tx.value = *unit.transaction.value.get(test.indexes.value).unwrap();

                env.tx.access_list = unit
                    .transaction
                    .access_lists
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
                    .collect();

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
                let mut state = revm::db::State::builder()
                    .with_cached_prestate(cache)
                    .with_bundle_update()
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
                *elapsed.lock().unwrap() += timer.elapsed();

                // validate results
                // this is in a closure so we can have a common printing routine for errors
                let check = || {
                    // if we expect exception revm should return error from execution.
                    // So we do not check logs and state root.
                    //
                    // Note that some tests that have exception and run tests from before state clear
                    // would touch the caller account and make it appear in state root calculation.
                    // This is not something that we would expect as invalid tx should not touch state.
                    // but as this is a cleanup of invalid tx it is not properly defined and in the end
                    // it does not matter.
                    // Test where this happens: `tests/GeneralStateTests/stTransactionTest/NoSrcAccountCreate.json`
                    // and you can check that we have only two "hash" values for before and after state clear.
                    match (&test.expect_exception, &exec_result) {
                        // do nothing
                        (None, Ok(_)) => (),
                        // return okay, exception is expected.
                        (Some(_), Err(_)) => return Ok(()),
                        _ => {
                            return Err(TestError {
                                name: name.clone(),
                                kind: TestErrorKind::UnexpectedException {
                                    expected_exception: test.expect_exception.clone(),
                                    got_exception: exec_result.clone().err().map(|e| e.to_string()),
                                },
                            });
                        }
                    }

                    let logs_root =
                        log_rlp_hash(exec_result.as_ref().map(|r| r.logs()).unwrap_or_default());

                    if logs_root != test.logs {
                        return Err(TestError {
                            name: name.clone(),
                            kind: TestErrorKind::LogsRootMismatch {
                                got: logs_root,
                                expected: test.logs,
                            },
                        });
                    }

                    let db = evm.db.as_ref().unwrap();
                    let state_root = state_merkle_trie_root(db.cache.trie_account());

                    if state_root != test.hash {
                        return Err(TestError {
                            name: name.clone(),
                            kind: TestErrorKind::StateRootMismatch {
                                got: state_root,
                                expected: test.hash,
                            },
                        });
                    }

                    Ok(())
                };

                // dump state and traces if test failed
                let Err(e) = check() else { continue };

                // print only once
                static FAILED: AtomicBool = AtomicBool::new(false);
                if FAILED.swap(true, Ordering::SeqCst) {
                    return Err(e);
                }

                // re build to run with tracing
                let mut cache = cache_state.clone();
                cache.set_state_clear_flag(SpecId::enabled(
                    env.cfg.spec_id,
                    revm::primitives::SpecId::SPURIOUS_DRAGON,
                ));
                let mut state = revm::db::StateBuilder::default()
                    .with_cached_prestate(cache)
                    .build();
                evm.database(&mut state);

                let path = path.display();
                println!("Test {name:?} (index: {index}, path: {path}) failed:\n{e}");

                println!("\nTraces:");
                let _ = evm.inspect_commit(TracerEip3155::new(Box::new(stdout()), false, false));

                println!("\nExecution result: {exec_result:#?}");
                println!("\nExpected exception: {:?}", test.expect_exception);
                println!("\nState before: {cache_state:#?}");
                println!("\nState after: {:#?}", evm.db().unwrap().cache);
                println!("\nEnvironment: {env:#?}");
                return Err(e);
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
    let n_files = test_files.len();

    let endjob = Arc::new(AtomicBool::new(false));
    let console_bar = Arc::new(ProgressBar::new(n_files as u64));
    let queue = Arc::new(Mutex::new((0usize, test_files)));
    let elapsed = Arc::new(Mutex::new(std::time::Duration::ZERO));

    let num_threads = match (single_thread, std::thread::available_parallelism()) {
        (true, _) | (false, Err(_)) => 1,
        (false, Ok(n)) => n.get(),
    };
    let num_threads = num_threads.min(n_files);
    let mut handles = Vec::with_capacity(num_threads);
    for i in 0..num_threads {
        let queue = queue.clone();
        let endjob = endjob.clone();
        let console_bar = console_bar.clone();
        let elapsed = elapsed.clone();

        let thread = std::thread::Builder::new().name(format!("runner-{i}"));

        let f = move || loop {
            if endjob.load(Ordering::SeqCst) {
                return Ok(());
            }

            let (_index, test_path) = {
                let (current_idx, queue) = &mut *queue.lock().unwrap();
                let prev_idx = *current_idx;
                let Some(test_path) = queue.get(prev_idx).cloned() else {
                    return Ok(());
                };
                *current_idx = prev_idx + 1;
                (prev_idx, test_path)
            };

            if let Err(err) = execute_test_suite(&test_path, &elapsed, trace) {
                endjob.store(true, Ordering::SeqCst);
                return Err(err);
            }

            console_bar.inc(1);
        };
        handles.push(thread.spawn(f).unwrap());
    }

    // join all threads before returning an error
    let mut errors = Vec::new();
    for handle in handles {
        if let Err(e) = handle.join().unwrap() {
            errors.push(e);
        }
    }

    console_bar.finish();

    println!(
        "Finished execution. Total CPU time: {:.6}s",
        elapsed.lock().unwrap().as_secs_f64()
    );
    if errors.is_empty() {
        println!("All tests passed!");
        Ok(())
    } else {
        let n = errors.len();
        if n > 1 {
            println!("{n} threads returned an error, out of {num_threads} total:");
            for error in &errors {
                println!("{error}");
            }
        }
        Err(errors.swap_remove(0))
    }
}
