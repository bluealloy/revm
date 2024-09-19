use super::{
    merkle_trie::{log_rlp_hash, state_merkle_trie_root},
    models::{SpecName, Test, TestSuite},
    utils::recover_address,
};
use database::State;
use indicatif::{ProgressBar, ProgressDrawTarget};
use inspector::{inspector_handle_register, inspectors::TracerEip3155};
use revm::{
    bytecode::Bytecode,
    database_interface::EmptyDB,
    interpreter::analysis::to_analysed,
    primitives::{keccak256, Bytes, TxKind, B256},
    specification::{eip7702::AuthorizationList, hardfork::SpecId},
    wiring::{
        block::calc_excess_blob_gas,
        default::EnvWiring,
        result::{EVMResultGeneric, ExecutionResult, HaltReason},
        EthereumWiring,
    },
    Evm,
};
use serde_json::json;
use std::{
    fmt::Debug,
    io::{stderr, stdout},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

type ExecEvmWiring<'a> = EthereumWiring<&'a mut State<EmptyDB>, ()>;
type TraceEvmWiring<'a> = EthereumWiring<&'a mut State<EmptyDB>, TracerEip3155>;

#[derive(Debug, Error)]
#[error("Test {name} failed: {kind}")]
pub struct TestError {
    pub name: String,
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
        // funky test with `bigint 0x00` value in json :) not possible to happen on mainnet and require
        // custom json parser. https://github.com/ethereum/tests/issues/971
        |"ValueOverflow.json"| "ValueOverflowParis.json"

        // precompiles having storage is not possible
        | "RevertPrecompiledTouch_storage.json"
        | "RevertPrecompiledTouch.json"

        // txbyte is of type 02 and we don't parse tx bytes for this test to fail.
        | "typeTwoBerlin.json"

        // Need to handle Test errors
        | "transactionIntinsicBug.json"

        // Test check if gas price overflows, we handle this correctly but does not match tests specific exception.
        | "HighGasPrice.json"
        | "CREATE_HighNonce.json"
        | "CREATE_HighNonceMinus1.json"
        | "CreateTransactionHighNonce.json"

        // Skip test where basefee/accesslist/difficulty is present but it shouldn't be supported in
        // London/Berlin/TheMerge. https://github.com/ethereum/tests/blob/5b7e1ab3ffaf026d99d20b17bb30f533a2c80c8b/GeneralStateTests/stExample/eip1559.json#L130
        // It is expected to not execute these tests.
        | "basefeeExample.json"
        | "eip1559.json"
        | "mergeTest.json"

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

        // These tests are passing, but they take a lot of time to execute so we are going to skip them.
        | "loopExp.json"
        | "Call50000_sha256.json"
        | "static_Call50000_sha256.json"
        | "loopMul.json"
        | "CALLBlake2f_MaxRounds.json"

        // evmone statetest
        | "initcode_transaction_before_prague.json"
        | "invalid_tx_non_existing_sender.json"
        | "tx_non_existing_sender.json"
        | "block_apply_withdrawal.json"
        | "block_apply_ommers_reward.json"
        | "known_block_hash.json"
        | "eip7516_blob_base_fee.json"
    )
}

fn check_evm_execution<EXT: Debug>(
    test: &Test,
    expected_output: Option<&Bytes>,
    test_name: &str,
    exec_result: &EVMResultGeneric<
        ExecutionResult<HaltReason>,
        EthereumWiring<&mut State<EmptyDB>, EXT>,
    >,
    evm: &Evm<'_, EthereumWiring<&mut State<EmptyDB>, EXT>>,
    print_json_outcome: bool,
) -> Result<(), TestError> {
    let logs_root = log_rlp_hash(exec_result.as_ref().map(|r| r.logs()).unwrap_or_default());
    let state_root = state_merkle_trie_root(evm.context.evm.db.cache.trie_account());

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
                "fork": evm.handler.spec_id(),
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
        // do nothing
        (None, Ok(result)) => {
            // check output
            if let Some((expected_output, output)) = expected_output.zip(result.output()) {
                if expected_output != output {
                    let kind = TestErrorKind::UnexpectedOutput {
                        expected_output: Some(expected_output.clone()),
                        got_output: result.output().cloned(),
                    };
                    print_json_output(Some(kind.to_string()));
                    return Err(TestError {
                        name: test_name.to_string(),
                        kind,
                    });
                }
            }
        }
        // return okay, exception is expected.
        (Some(_), Err(_)) => return Ok(()),
        _ => {
            let kind = TestErrorKind::UnexpectedException {
                expected_exception: test.expect_exception.clone(),
                got_exception: exec_result.clone().err().map(|e| e.to_string()),
            };
            print_json_output(Some(kind.to_string()));
            return Err(TestError {
                name: test_name.to_string(),
                kind,
            });
        }
    }

    if logs_root != test.logs {
        let kind = TestErrorKind::LogsRootMismatch {
            got: logs_root,
            expected: test.logs,
        };
        print_json_output(Some(kind.to_string()));
        return Err(TestError {
            name: test_name.to_string(),
            kind,
        });
    }

    if state_root != test.hash {
        let kind = TestErrorKind::StateRootMismatch {
            got: state_root,
            expected: test.hash,
        };
        print_json_output(Some(kind.to_string()));
        return Err(TestError {
            name: test_name.to_string(),
            kind,
        });
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
    let suite: TestSuite = serde_json::from_str(&s).map_err(|e| TestError {
        name: path.to_string_lossy().into_owned(),
        kind: e.into(),
    })?;

    for (name, unit) in suite.0 {
        // Create database and insert cache
        let mut cache_state = database::CacheState::new(false);
        for (address, info) in unit.pre {
            let code_hash = keccak256(&info.code);
            let bytecode = to_analysed(Bytecode::new_raw(info.code));
            let acc_info = revm::state::AccountInfo {
                balance: info.balance,
                code_hash,
                code: Some(bytecode),
                nonce: info.nonce,
            };
            cache_state.insert_account_with_storage(address, acc_info, info.storage);
        }

        let mut env = Box::<EnvWiring<ExecEvmWiring>>::default();
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
        env.block.prevrandao = unit.env.current_random;
        // EIP-4844
        if let Some(current_excess_blob_gas) = unit.env.current_excess_blob_gas {
            env.block
                .set_blob_excess_gas_and_price(current_excess_blob_gas.to());
        } else if let (Some(parent_blob_gas_used), Some(parent_excess_blob_gas)) = (
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
        env.tx.caller = if let Some(address) = unit.transaction.sender {
            address
        } else {
            recover_address(unit.transaction.secret_key.as_slice()).ok_or_else(|| TestError {
                name: name.clone(),
                kind: TestErrorKind::UnknownPrivateKey(unit.transaction.secret_key),
            })?
        };
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
            // Constantinople was immediately extended by Petersburg.
            // There isn't any production Constantinople transaction
            // so we don't support it and skip right to Petersburg.
            if spec_name == SpecName::Constantinople || spec_name == SpecName::Osaka {
                continue;
            }

            // Enable EOF in Prague tests.
            let spec_id = if spec_name == SpecName::Prague {
                SpecId::PRAGUE_EOF
            } else {
                spec_name.to_spec_id()
            };

            if spec_id.is_enabled_in(SpecId::MERGE) && env.block.prevrandao.is_none() {
                // if spec is merge and prevrandao is not set, set it to default
                env.block.prevrandao = Some(B256::default());
            }

            for (index, test) in tests.into_iter().enumerate() {
                env.tx.gas_limit = unit.transaction.gas_limit[test.indexes.gas].saturating_to();

                env.tx.data = unit
                    .transaction
                    .data
                    .get(test.indexes.data)
                    .unwrap()
                    .clone();

                env.tx.nonce = u64::try_from(unit.transaction.nonce).unwrap();
                env.tx.value = unit.transaction.value[test.indexes.value];

                env.tx.access_list = unit
                    .transaction
                    .access_lists
                    .get(test.indexes.data)
                    .and_then(Option::as_deref)
                    .cloned()
                    .unwrap_or_default();

                env.tx.authorization_list =
                    unit.transaction
                        .authorization_list
                        .as_ref()
                        .map(|auth_list| {
                            AuthorizationList::Recovered(
                                auth_list.iter().map(|auth| auth.into_recovered()).collect(),
                            )
                        });

                let to = match unit.transaction.to {
                    Some(add) => TxKind::Call(add),
                    None => TxKind::Create,
                };
                env.tx.transact_to = to;

                let mut cache = cache_state.clone();
                cache.set_state_clear_flag(SpecId::enabled(spec_id, SpecId::SPURIOUS_DRAGON));
                let mut state = database::State::builder()
                    .with_cached_prestate(cache)
                    .with_bundle_update()
                    .build();
                let mut evm = Evm::<ExecEvmWiring>::builder()
                    .with_db(&mut state)
                    .with_default_ext_ctx()
                    .modify_env(|e| e.clone_from(&env))
                    .with_spec_id(spec_id)
                    .build();

                // do the deed
                let (e, exec_result) = if trace {
                    let mut evm = evm
                        .modify()
                        .reset_handler_with_external_context::<EthereumWiring<_, TracerEip3155>>()
                        .with_external_context(
                            TracerEip3155::new(Box::new(stderr())).without_summary(),
                        )
                        .with_spec_id(spec_id)
                        .append_handler_register(inspector_handle_register)
                        .build();

                    let timer = Instant::now();
                    let res = evm.transact_commit();
                    *elapsed.lock().unwrap() += timer.elapsed();

                    let Err(e) = check_evm_execution(
                        &test,
                        unit.out.as_ref(),
                        &name,
                        &res,
                        &evm,
                        print_json_outcome,
                    ) else {
                        continue;
                    };
                    // reset external context
                    (e, res)
                } else {
                    let timer = Instant::now();
                    let res = evm.transact_commit();
                    *elapsed.lock().unwrap() += timer.elapsed();

                    // dump state and traces if test failed
                    let output = check_evm_execution(
                        &test,
                        unit.out.as_ref(),
                        &name,
                        &res,
                        &evm,
                        print_json_outcome,
                    );
                    let Err(e) = output else {
                        continue;
                    };
                    (e, res)
                };

                // print only once or
                // if we are already in trace mode, just return error
                static FAILED: AtomicBool = AtomicBool::new(false);
                if trace || FAILED.swap(true, Ordering::SeqCst) {
                    return Err(e);
                }

                // re build to run with tracing
                let mut cache = cache_state.clone();
                cache.set_state_clear_flag(SpecId::enabled(spec_id, SpecId::SPURIOUS_DRAGON));
                let mut state = database::State::builder()
                    .with_cached_prestate(cache)
                    .with_bundle_update()
                    .build();

                let path = path.display();
                println!("\nTraces:");
                let mut evm = Evm::<TraceEvmWiring>::builder()
                    .with_db(&mut state)
                    .with_spec_id(spec_id)
                    .with_env(env.clone())
                    .reset_handler_with_external_context::<EthereumWiring<_, TracerEip3155>>()
                    .with_external_context(TracerEip3155::new(Box::new(stdout())).without_summary())
                    .with_spec_id(spec_id)
                    .append_handler_register(inspector_handle_register)
                    .build();
                let _ = evm.transact_commit();

                println!("\nExecution result: {exec_result:#?}");
                println!("\nExpected exception: {:?}", test.expect_exception);
                println!("\nState before: {cache_state:#?}");
                println!("\nState after: {:#?}", evm.context.evm.db.cache);
                println!("\nSpecification: {spec_id:?}");
                println!("\nEnvironment: {env:#?}");
                println!("\nTest name: {name:?} (index: {index}, path: {path}) failed:\n{e}");

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
    mut print_outcome: bool,
    keep_going: bool,
) -> Result<(), TestError> {
    // trace implies print_outcome
    if trace {
        print_outcome = true;
    }
    // print_outcome or trace implies single_thread
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
