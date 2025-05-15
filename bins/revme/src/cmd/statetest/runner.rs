use super::{
    merkle_trie::{log_rlp_hash, state_merkle_trie_root},
    utils::recover_address,
};
use context::either::Either;
use database::State;
use indicatif::{ProgressBar, ProgressDrawTarget};
use inspector::{inspectors::TracerEip3155, InspectCommitEvm};
use revm::{
    bytecode::Bytecode,
    context::{block::BlockEnv, cfg::CfgEnv, tx::TxEnv},
    context_interface::{
        block::calc_excess_blob_gas,
        result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction},
        Cfg,
    },
    database_interface::EmptyDB,
    primitives::{
        eip4844::TARGET_BLOB_GAS_PER_BLOCK_CANCUN, hardfork::SpecId, keccak256, Bytes, TxKind, B256,
    },
    Context, ExecuteCommitEvm, MainBuilder, MainContext,
};
use serde_json::json;
use statetest_types::{SpecName, Test, TestSuite};

use std::{
    convert::Infallible,
    fmt::Debug,
    io::stderr,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Error)]
#[error("Path: {path}\nName: {name}\nError: {kind}")]
pub struct TestError {
    pub name: String,
    pub path: String,
    pub kind: TestErrorKind,
}

#[derive(Debug, Error)]
pub enum TestErrorKind {
    #[error("logs root mismatch: got {got}, expected {expected}")]
    LogsRootMismatch { got: B256, expected: B256 },
    #[error("state root mismatch: got {got}, expected {expected}")]
    StateRootMismatch { got: B256, expected: B256 },
    #[error("unknown private key: {0:?}")]
    UnknownPrivateKey(B256),
    #[error("unexpected exception: got {got_exception:?}, expected {expected_exception:?}")]
    UnexpectedException {
        expected_exception: Option<String>,
        got_exception: Option<String>,
    },
    #[error("unexpected output: got {got_output:?}, expected {expected_output:?}")]
    UnexpectedOutput {
        expected_output: Option<Bytes>,
        got_output: Option<Bytes>,
    },
    #[error(transparent)]
    SerdeDeserialize(#[from] serde_json::Error),
    #[error("thread panicked")]
    Panic,
    #[error("path does not exist")]
    InvalidPath,
    #[error("no JSON test files found in path")]
    NoJsonFiles,
}

pub fn find_all_json_tests(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        vec![path.to_path_buf()]
    } else {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension() == Some("json".as_ref()))
            .map(DirEntry::into_path)
            .collect()
    }
}

fn skip_test(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_str().unwrap();

    matches!(
        name,
        // Test check if gas price overflows, we handle this correctly but does not match tests specific exception.
        | "CreateTransactionHighNonce.json"

        // Test with some storage check.
        | "RevertInCreateInInit_Paris.json"
        | "RevertInCreateInInit.json"
        | "dynamicAccountOverwriteEmpty.json"
        | "dynamicAccountOverwriteEmpty_Paris.json"
        | "RevertInCreateInInitCreate2Paris.json"
        | "create2collisionStorage.json"
        | "RevertInCreateInInitCreate2.json"
        | "create2collisionStorageParis.json"
        | "InitCollision.json"
        | "InitCollisionParis.json"

        // Malformed value.
        | "ValueOverflow.json"
        | "ValueOverflowParis.json"

        // These tests are passing, but they take a lot of time to execute so we are going to skip them.
        | "Call50000_sha256.json"
        | "static_Call50000_sha256.json"
        | "loopMul.json"
        | "CALLBlake2f_MaxRounds.json"
    )
}

fn check_evm_execution(
    test: &Test,
    expected_output: Option<&Bytes>,
    test_name: &str,
    exec_result: &Result<ExecutionResult<HaltReason>, EVMError<Infallible, InvalidTransaction>>,
    db: &mut State<EmptyDB>,
    spec: SpecId,
    print_json_outcome: bool,
) -> Result<(), TestErrorKind> {
    let logs_root = log_rlp_hash(exec_result.as_ref().map(|r| r.logs()).unwrap_or_default());
    let state_root = state_merkle_trie_root(db.cache.trie_account());

    let print_json_output = |error: Option<String>| {
        if print_json_outcome {
            let json = json!({
                "stateRoot": state_root,
                "logsRoot": logs_root,
                "output": exec_result.as_ref().ok().and_then(|r| r.output().cloned()).unwrap_or_default(),
                "gasUsed": exec_result.as_ref().ok().map(|r| r.gas_used()).unwrap_or_default(),
                "pass": error.is_none(),
                "errorMsg": error.unwrap_or_default(),
                "evmResult": match exec_result {
                    Ok(r) => match r {
                        ExecutionResult::Success { reason, .. } => format!("Success: {reason:?}"),
                        ExecutionResult::Revert { .. } => "Revert".to_string(),
                        ExecutionResult::Halt { reason, .. } => format!("Halt: {reason:?}"),
                    },
                    Err(e) => e.to_string(),
                },
                "postLogsHash": logs_root,
                "fork": spec,
                "test": test_name,
                "d": test.indexes.data,
                "g": test.indexes.gas,
                "v": test.indexes.value,
            });
            eprintln!("{json}");
        }
    };

    // If we expect exception revm should return error from execution.
    // So we do not check logs and state root.
    //
    // Note that some tests that have exception and run tests from before state clear
    // would touch the caller account and make it appear in state root calculation.
    // This is not something that we would expect as invalid tx should not touch state.
    // but as this is a cleanup of invalid tx it is not properly defined and in the end
    // it does not matter.
    // Test where this happens: `tests/GeneralStateTests/stTransactionTest/NoSrcAccountCreate.json`
    // and you can check that we have only two "hash" values for before and after state clear.
    match (&test.expect_exception, exec_result) {
        // Do nothing
        (None, Ok(result)) => {
            // Check output
            if let Some((expected_output, output)) = expected_output.zip(result.output()) {
                if expected_output != output {
                    let kind = TestErrorKind::UnexpectedOutput {
                        expected_output: Some(expected_output.clone()),
                        got_output: result.output().cloned(),
                    };
                    print_json_output(Some(kind.to_string()));
                    return Err(kind);
                }
            }
        }
        // Return okay, exception is expected.
        (Some(_), Err(_)) => return Ok(()),
        _ => {
            let kind = TestErrorKind::UnexpectedException {
                expected_exception: test.expect_exception.clone(),
                got_exception: exec_result.clone().err().map(|e| e.to_string()),
            };
            print_json_output(Some(kind.to_string()));
            return Err(kind);
        }
    }

    if logs_root != test.logs {
        let kind = TestErrorKind::LogsRootMismatch {
            got: logs_root,
            expected: test.logs,
        };
        print_json_output(Some(kind.to_string()));
        return Err(kind);
    }

    if state_root != test.hash {
        let kind = TestErrorKind::StateRootMismatch {
            got: state_root,
            expected: test.hash,
        };
        print_json_output(Some(kind.to_string()));
        return Err(kind);
    }

    print_json_output(None);

    Ok(())
}

pub fn execute_test_suite(
    path: &Path,
    elapsed: &Arc<Mutex<Duration>>,
    trace: bool,
    print_json_outcome: bool,
) -> Result<(), TestError> {
    if skip_test(path) {
        return Ok(());
    }

    let s = std::fs::read_to_string(path).unwrap();
    let path = path.to_string_lossy().into_owned();
    let suite: TestSuite = serde_json::from_str(&s).map_err(|e| TestError {
        name: "Unknown".to_string(),
        path: path.clone(),
        kind: e.into(),
    })?;

    for (name, unit) in suite.0 {
        // Create database and insert cache
        let mut cache_state = database::CacheState::new(false);
        for (address, info) in unit.pre {
            let code_hash = keccak256(&info.code);
            let bytecode = Bytecode::new_raw_checked(info.code.clone())
                .unwrap_or(Bytecode::new_legacy(info.code));
            let acc_info = revm::state::AccountInfo {
                balance: info.balance,
                code_hash,
                code: Some(bytecode),
                nonce: info.nonce,
            };
            cache_state.insert_account_with_storage(address, acc_info, info.storage);
        }

        let mut cfg = CfgEnv::default();
        let mut block = BlockEnv::default();
        let mut tx = TxEnv::default();
        // For mainnet
        cfg.chain_id = 1;

        // Block env
        block.number = unit.env.current_number.try_into().unwrap_or(u64::MAX);
        block.beneficiary = unit.env.current_coinbase;
        block.timestamp = unit.env.current_timestamp.try_into().unwrap_or(u64::MAX);
        block.gas_limit = unit.env.current_gas_limit.try_into().unwrap_or(u64::MAX);
        block.basefee = unit
            .env
            .current_base_fee
            .unwrap_or_default()
            .try_into()
            .unwrap_or(u64::MAX);
        block.difficulty = unit.env.current_difficulty;
        // After the Merge prevrandao replaces mix_hash field in block and replaced difficulty opcode in EVM.
        block.prevrandao = unit.env.current_random;

        // Tx env
        tx.caller = if let Some(address) = unit.transaction.sender {
            address
        } else {
            recover_address(unit.transaction.secret_key.as_slice()).ok_or_else(|| TestError {
                name: name.clone(),
                path: path.clone(),
                kind: TestErrorKind::UnknownPrivateKey(unit.transaction.secret_key),
            })?
        };
        tx.gas_price = unit
            .transaction
            .gas_price
            .or(unit.transaction.max_fee_per_gas)
            .unwrap_or_default()
            .try_into()
            .unwrap_or(u128::MAX);
        tx.gas_priority_fee = unit
            .transaction
            .max_priority_fee_per_gas
            .map(|b| u128::try_from(b).expect("max priority fee less than u128::MAX"));
        // EIP-4844
        tx.blob_hashes = unit.transaction.blob_versioned_hashes.clone();
        tx.max_fee_per_blob_gas = unit
            .transaction
            .max_fee_per_blob_gas
            .map(|b| u128::try_from(b).expect("max fee less than u128::MAX"))
            .unwrap_or(u128::MAX);

        // Post and execution
        for (spec_name, tests) in unit.post {
            // Constantinople was immediately extended by Petersburg.
            // There isn't any production Constantinople transaction
            // so we don't support it and skip right to Petersburg.
            if spec_name == SpecName::Constantinople {
                continue;
            }

            cfg.spec = spec_name.to_spec_id();

            // EIP-4844
            if let Some(current_excess_blob_gas) = unit.env.current_excess_blob_gas {
                block.set_blob_excess_gas_and_price(
                    current_excess_blob_gas.to(),
                    cfg.spec.is_enabled_in(SpecId::PRAGUE),
                );
            } else if let (Some(parent_blob_gas_used), Some(parent_excess_blob_gas)) = (
                unit.env.parent_blob_gas_used,
                unit.env.parent_excess_blob_gas,
            ) {
                block.set_blob_excess_gas_and_price(
                    calc_excess_blob_gas(
                        parent_blob_gas_used.to(),
                        parent_excess_blob_gas.to(),
                        unit.env
                            .parent_target_blobs_per_block
                            .map(|i| i.to())
                            .unwrap_or(TARGET_BLOB_GAS_PER_BLOCK_CANCUN),
                    ),
                    cfg.spec.is_enabled_in(SpecId::PRAGUE),
                );
            }

            if cfg.spec.is_enabled_in(SpecId::MERGE) && block.prevrandao.is_none() {
                // If spec is merge and prevrandao is not set, set it to default
                block.prevrandao = Some(B256::default());
            }

            for (index, test) in tests.into_iter().enumerate() {
                let Some(tx_type) = unit.transaction.tx_type(test.indexes.data) else {
                    if test.expect_exception.is_some() {
                        continue;
                    } else {
                        panic!("Invalid transaction type without expected exception");
                    }
                };
                tx.tx_type = tx_type as u8;

                tx.gas_limit = unit.transaction.gas_limit[test.indexes.gas].saturating_to();
                tx.data = unit
                    .transaction
                    .data
                    .get(test.indexes.data)
                    .unwrap()
                    .clone();

                tx.nonce = u64::try_from(unit.transaction.nonce).unwrap();
                tx.value = unit.transaction.value[test.indexes.value];

                tx.access_list = unit
                    .transaction
                    .access_lists
                    .get(test.indexes.data)
                    .cloned()
                    .flatten()
                    .unwrap_or_default();

                // TODO(EOF)
                //tx.initcodes = unit.transaction.initcodes.clone().unwrap_or_default();

                tx.authorization_list = unit
                    .transaction
                    .authorization_list
                    .clone()
                    .map(|auth_list| {
                        auth_list
                            .into_iter()
                            .map(|i| Either::Left(i.into()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                let to = match unit.transaction.to {
                    Some(add) => TxKind::Call(add),
                    None => TxKind::Create,
                };
                tx.kind = to;

                let mut cache = cache_state.clone();
                cache.set_state_clear_flag(cfg.spec.is_enabled_in(SpecId::SPURIOUS_DRAGON));
                let mut state = database::State::builder()
                    .with_cached_prestate(cache)
                    .with_bundle_update()
                    .build();

                let evm_context = Context::mainnet()
                    .with_block(&block)
                    .with_tx(&tx)
                    .with_cfg(&cfg)
                    .with_db(&mut state);

                // Do the deed
                let timer = Instant::now();
                let (db, exec_result) = if trace {
                    let mut evm = evm_context.build_mainnet_with_inspector(
                        TracerEip3155::buffered(stderr()).without_summary(),
                    );
                    let res = evm.inspect_replay_commit();
                    let db = evm.ctx.journaled_state.database;
                    (db, res)
                } else {
                    let mut evm = evm_context.build_mainnet();
                    let res = evm.replay_commit();
                    let db = evm.ctx.journaled_state.database;
                    (db, res)
                };
                *elapsed.lock().unwrap() += timer.elapsed();
                let spec = cfg.spec();
                // Dump state and traces if test failed
                let output = check_evm_execution(
                    &test,
                    unit.out.as_ref(),
                    &name,
                    &exec_result,
                    db,
                    spec,
                    print_json_outcome,
                );
                let Err(e) = output else {
                    continue;
                };

                // Print only once or if we are already in trace mode, just return error
                // If trace is true that print_json_outcome will be also true.
                static FAILED: AtomicBool = AtomicBool::new(false);
                if print_json_outcome || FAILED.swap(true, Ordering::SeqCst) {
                    return Err(TestError {
                        name: name.clone(),
                        path: path.clone(),
                        kind: e,
                    });
                }

                // Re-build to run with tracing
                let mut cache = cache_state.clone();
                cache.set_state_clear_flag(cfg.spec.is_enabled_in(SpecId::SPURIOUS_DRAGON));
                let mut state = database::State::builder()
                    .with_cached_prestate(cache)
                    .with_bundle_update()
                    .build();

                println!("\nTraces:");

                let mut evm = Context::mainnet()
                    .with_db(&mut state)
                    .with_block(&block)
                    .with_tx(&tx)
                    .with_cfg(&cfg)
                    .build_mainnet_with_inspector(
                        TracerEip3155::buffered(stderr()).without_summary(),
                    );

                let _ = evm.inspect_replay_commit();

                println!("\nExecution result: {exec_result:#?}");
                println!("\nExpected exception: {:?}", test.expect_exception);
                println!("\nState before: {cache_state:#?}");
                println!(
                    "\nState after: {:#?}",
                    evm.ctx.journaled_state.database.cache
                );
                println!("\nSpecification: {:?}", cfg.spec);
                println!("\nTx: {tx:#?}");
                println!("Block: {block:#?}");
                println!("Cfg: {cfg:#?}");
                println!("\nTest name: {name:?} (index: {index}, path: {path:?}) failed:\n{e}");

                return Err(TestError {
                    path: path.clone(),
                    name: name.clone(),
                    kind: e,
                });
            }
        }
    }
    Ok(())
}

pub fn run(
    test_files: Vec<PathBuf>,
    mut single_thread: bool,
    trace: bool,
    mut print_outcome: bool,
    keep_going: bool,
) -> Result<(), TestError> {
    // Trace implies print_outcome
    if trace {
        print_outcome = true;
    }
    // `print_outcome` or trace implies single_thread
    if print_outcome {
        single_thread = true;
    }
    let n_files = test_files.len();

    let n_errors = Arc::new(AtomicUsize::new(0));
    let console_bar = Arc::new(ProgressBar::with_draw_target(
        Some(n_files as u64),
        ProgressDrawTarget::stdout(),
    ));
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
        let n_errors = n_errors.clone();
        let console_bar = console_bar.clone();
        let elapsed = elapsed.clone();

        let thread = std::thread::Builder::new().name(format!("runner-{i}"));

        let f = move || loop {
            if !keep_going && n_errors.load(Ordering::SeqCst) > 0 {
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

            let result = execute_test_suite(&test_path, &elapsed, trace, print_outcome);

            // Increment after the test is done.
            console_bar.inc(1);

            if let Err(err) = result {
                n_errors.fetch_add(1, Ordering::SeqCst);
                if !keep_going {
                    return Err(err);
                }
            }
        };
        handles.push(thread.spawn(f).unwrap());
    }

    // join all threads before returning an error
    let mut thread_errors = Vec::new();
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => thread_errors.push(e),
            Err(_) => thread_errors.push(TestError {
                name: format!("thread {i} panicked"),
                path: "".to_string(),
                kind: TestErrorKind::Panic,
            }),
        }
    }
    console_bar.finish();

    println!(
        "Finished execution. Total CPU time: {:.6}s",
        elapsed.lock().unwrap().as_secs_f64()
    );

    let n_errors = n_errors.load(Ordering::SeqCst);
    let n_thread_errors = thread_errors.len();
    if n_errors == 0 && n_thread_errors == 0 {
        println!("All tests passed!");
        Ok(())
    } else {
        println!("Encountered {n_errors} errors out of {n_files} total tests");

        if n_thread_errors == 0 {
            std::process::exit(1);
        }

        if n_thread_errors > 1 {
            println!("{n_thread_errors} threads returned an error, out of {num_threads} total:");
            for error in &thread_errors {
                println!("{error}");
            }
        }
        Err(thread_errors.swap_remove(0))
    }
}
