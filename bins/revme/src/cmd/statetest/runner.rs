use crate::cmd::statetest::merkle_trie::{compute_test_roots, TestValidationResult};
use database::State;
use indicatif::{ProgressBar, ProgressDrawTarget};
use inspector::{inspectors::TracerEip3155, InspectCommitEvm};
use primitives::U256;
use revm::{
    context::{block::BlockEnv, cfg::CfgEnv, tx::TxEnv},
    context_interface::{
        result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction},
        Cfg,
    },
    database_interface::EmptyDB,
    primitives::{hardfork::SpecId, Bytes, B256},
    Context, ExecuteCommitEvm, MainBuilder, MainContext,
};
use serde_json::json;
use statetest_types::{SpecName, Test, TestSuite, TestUnit};
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

/// Error that occurs during test execution
#[derive(Debug, Error)]
#[error("Path: {path}\nName: {name}\nError: {kind}")]
pub struct TestError {
    pub name: String,
    pub path: String,
    pub kind: TestErrorKind,
}

/// Specific kind of error that occurred during test execution
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

/// Find all JSON test files in the given path
/// If path is a file, returns it in a vector
/// If path is a directory, recursively finds all .json files
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

/// Check if a test should be skipped based on its filename
/// Some tests are known to be problematic or take too long
fn skip_test(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        // Non-UTF file names or missing file name: do not skip by default.
        return false;
    };

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

struct TestExecutionContext<'a> {
    name: &'a str,
    unit: &'a TestUnit,
    test: &'a Test,
    cfg: &'a CfgEnv,
    block: &'a BlockEnv,
    tx: &'a TxEnv,
    cache_state: &'a database::CacheState,
    elapsed: &'a Arc<Mutex<Duration>>,
    trace: bool,
    print_json_outcome: bool,
}

struct DebugContext<'a> {
    name: &'a str,
    path: &'a str,
    index: usize,
    test: &'a Test,
    cfg: &'a CfgEnv,
    block: &'a BlockEnv,
    tx: &'a TxEnv,
    cache_state: &'a database::CacheState,
    error: &'a TestErrorKind,
}

fn build_json_output(
    test: &Test,
    test_name: &str,
    exec_result: &Result<ExecutionResult<HaltReason>, EVMError<Infallible, InvalidTransaction>>,
    validation: &TestValidationResult,
    spec: SpecId,
    error: Option<String>,
) -> serde_json::Value {
    json!({
        "stateRoot": validation.state_root,
        "logsRoot": validation.logs_root,
        "output": exec_result.as_ref().ok().and_then(|r| r.output().cloned()).unwrap_or_default(),
        "gasUsed": exec_result.as_ref().ok().map(|r| r.gas_used()).unwrap_or_default(),
        "pass": error.is_none(),
        "errorMsg": error.unwrap_or_default(),
        "evmResult": format_evm_result(exec_result),
        "postLogsHash": validation.logs_root,
        "fork": spec,
        "test": test_name,
        "d": test.indexes.data,
        "g": test.indexes.gas,
        "v": test.indexes.value,
    })
}

fn format_evm_result(
    exec_result: &Result<ExecutionResult<HaltReason>, EVMError<Infallible, InvalidTransaction>>,
) -> String {
    match exec_result {
        Ok(r) => match r {
            ExecutionResult::Success { reason, .. } => format!("Success: {reason:?}"),
            ExecutionResult::Revert { .. } => "Revert".to_string(),
            ExecutionResult::Halt { reason, .. } => format!("Halt: {reason:?}"),
        },
        Err(e) => e.to_string(),
    }
}

fn validate_exception(
    test: &Test,
    exec_result: &Result<ExecutionResult<HaltReason>, EVMError<Infallible, InvalidTransaction>>,
) -> Result<bool, TestErrorKind> {
    match (&test.expect_exception, exec_result) {
        (None, Ok(_)) => Ok(false), // No exception expected, execution succeeded
        (Some(_), Err(_)) => Ok(true), // Exception expected and occurred
        _ => Err(TestErrorKind::UnexpectedException {
            expected_exception: test.expect_exception.clone(),
            got_exception: exec_result.as_ref().err().map(|e| e.to_string()),
        }),
    }
}

fn validate_output(
    expected_output: Option<&Bytes>,
    actual_result: &ExecutionResult<HaltReason>,
) -> Result<(), TestErrorKind> {
    if let Some((expected, actual)) = expected_output.zip(actual_result.output()) {
        if expected != actual {
            return Err(TestErrorKind::UnexpectedOutput {
                expected_output: Some(expected.clone()),
                got_output: actual_result.output().cloned(),
            });
        }
    }
    Ok(())
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
    let validation = compute_test_roots(exec_result, db);

    let print_json = |error: Option<&TestErrorKind>| {
        if print_json_outcome {
            let json = build_json_output(
                test,
                test_name,
                exec_result,
                &validation,
                spec,
                error.map(|e| e.to_string()),
            );
            eprintln!("{json}");
        }
    };

    // Check if exception handling is correct
    let exception_expected = validate_exception(test, exec_result).inspect_err(|e| {
        print_json(Some(e));
    })?;

    // If exception was expected and occurred, we're done
    if exception_expected {
        print_json(None);
        return Ok(());
    }

    // Validate output if execution succeeded
    if let Ok(result) = exec_result {
        validate_output(expected_output, result).inspect_err(|e| {
            print_json(Some(e));
        })?;
    }

    // Validate logs root
    if validation.logs_root != test.logs {
        let error = TestErrorKind::LogsRootMismatch {
            got: validation.logs_root,
            expected: test.logs,
        };
        print_json(Some(&error));
        return Err(error);
    }

    // Validate state root
    if validation.state_root != test.hash {
        let error = TestErrorKind::StateRootMismatch {
            got: validation.state_root,
            expected: test.hash,
        };
        print_json(Some(&error));
        return Err(error);
    }

    print_json(None);
    Ok(())
}

/// Execute a single test suite file containing multiple tests
///
/// # Arguments
/// * `path` - Path to the JSON test file
/// * `elapsed` - Shared counter for total execution time
/// * `trace` - Whether to enable EVM tracing
/// * `print_json_outcome` - Whether to print JSON formatted results
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
        // Prepare initial state
        let cache_state = unit.state();

        // Setup base configuration
        let mut cfg = CfgEnv::default();
        cfg.chain_id = unit
            .env
            .current_chain_id
            .unwrap_or(U256::ONE)
            .try_into()
            .unwrap_or(1);

        // Post and execution
        for (spec_name, tests) in &unit.post {
            // Skip Constantinople spec
            if *spec_name == SpecName::Constantinople {
                continue;
            }

            cfg.spec = spec_name.to_spec_id();

            // Configure max blobs per spec
            if cfg.spec.is_enabled_in(SpecId::OSAKA) {
                cfg.set_max_blobs_per_tx(6);
            } else if cfg.spec.is_enabled_in(SpecId::PRAGUE) {
                cfg.set_max_blobs_per_tx(9);
            } else {
                cfg.set_max_blobs_per_tx(6);
            }

            // Setup block environment for this spec
            let block = unit.block_env(&cfg);

            for (index, test) in tests.iter().enumerate() {
                // Setup transaction environment
                let tx = match test.tx_env(&unit) {
                    Ok(tx) => tx,
                    Err(_) if test.expect_exception.is_some() => continue,
                    Err(_) => {
                        return Err(TestError {
                            name: name.clone(),
                            path: path.clone(),
                            kind: TestErrorKind::UnknownPrivateKey(unit.transaction.secret_key),
                        });
                    }
                };

                // Execute the test
                let result = execute_single_test(TestExecutionContext {
                    name: &name,
                    unit: &unit,
                    test,
                    cfg: &cfg,
                    block: &block,
                    tx: &tx,
                    cache_state: &cache_state,
                    elapsed,
                    trace,
                    print_json_outcome,
                });

                if let Err(e) = result {
                    // Handle error with debug trace if needed
                    static FAILED: AtomicBool = AtomicBool::new(false);
                    if print_json_outcome || FAILED.swap(true, Ordering::SeqCst) {
                        return Err(TestError {
                            name: name.clone(),
                            path: path.clone(),
                            kind: e,
                        });
                    }

                    // Re-run with trace for debugging
                    debug_failed_test(DebugContext {
                        name: &name,
                        path: &path,
                        index,
                        test,
                        cfg: &cfg,
                        block: &block,
                        tx: &tx,
                        cache_state: &cache_state,
                        error: &e,
                    });

                    return Err(TestError {
                        path: path.clone(),
                        name: name.clone(),
                        kind: e,
                    });
                }
            }
        }
    }
    Ok(())
}

fn execute_single_test(ctx: TestExecutionContext) -> Result<(), TestErrorKind> {
    // Prepare state
    let mut cache = ctx.cache_state.clone();
    cache.set_state_clear_flag(ctx.cfg.spec.is_enabled_in(SpecId::SPURIOUS_DRAGON));
    let mut state = database::State::builder()
        .with_cached_prestate(cache)
        .with_bundle_update()
        .build();

    let evm_context = Context::mainnet()
        .with_block(ctx.block)
        .with_tx(ctx.tx)
        .with_cfg(ctx.cfg)
        .with_db(&mut state);

    // Execute
    let timer = Instant::now();
    let (db, exec_result) = if ctx.trace {
        let mut evm = evm_context
            .build_mainnet_with_inspector(TracerEip3155::buffered(stderr()).without_summary());
        let res = evm.inspect_tx_commit(ctx.tx);
        let db = evm.ctx.journaled_state.database;
        (db, res)
    } else {
        let mut evm = evm_context.build_mainnet();
        let res = evm.transact_commit(ctx.tx);
        let db = evm.ctx.journaled_state.database;
        (db, res)
    };
    *ctx.elapsed.lock().unwrap() += timer.elapsed();

    // Check results
    check_evm_execution(
        ctx.test,
        ctx.unit.out.as_ref(),
        ctx.name,
        &exec_result,
        db,
        ctx.cfg.spec(),
        ctx.print_json_outcome,
    )
}

fn debug_failed_test(ctx: DebugContext) {
    println!("\nTraces:");

    // Re-run with tracing
    let mut cache = ctx.cache_state.clone();
    cache.set_state_clear_flag(ctx.cfg.spec.is_enabled_in(SpecId::SPURIOUS_DRAGON));
    let mut state = database::State::builder()
        .with_cached_prestate(cache)
        .with_bundle_update()
        .build();

    let mut evm = Context::mainnet()
        .with_db(&mut state)
        .with_block(ctx.block)
        .with_tx(ctx.tx)
        .with_cfg(ctx.cfg)
        .build_mainnet_with_inspector(TracerEip3155::buffered(stderr()).without_summary());

    let exec_result = evm.inspect_tx_commit(ctx.tx);

    println!("\nExecution result: {exec_result:#?}");
    println!("\nExpected exception: {:?}", ctx.test.expect_exception);
    println!("\nState before:\n{}", ctx.cache_state.pretty_print());
    println!(
        "\nState after:\n{}",
        evm.ctx.journaled_state.database.cache.pretty_print()
    );
    println!("\nSpecification: {:?}", ctx.cfg.spec);
    println!("\nTx: {:#?}", ctx.tx);
    println!("Block: {:#?}", ctx.block);
    println!("Cfg: {:#?}", ctx.cfg);
    println!(
        "\nTest name: {:?} (index: {}, path: {:?}) failed:\n{}",
        ctx.name, ctx.index, ctx.path, ctx.error
    );
}

#[derive(Clone, Copy)]
struct TestRunnerConfig {
    single_thread: bool,
    trace: bool,
    print_outcome: bool,
    keep_going: bool,
}

impl TestRunnerConfig {
    fn new(single_thread: bool, trace: bool, print_outcome: bool, keep_going: bool) -> Self {
        // Trace implies print_outcome
        let print_outcome = print_outcome || trace;
        // print_outcome or trace implies single_thread
        let single_thread = single_thread || print_outcome;

        Self {
            single_thread,
            trace,
            print_outcome,
            keep_going,
        }
    }
}

#[derive(Clone)]
struct TestRunnerState {
    n_errors: Arc<AtomicUsize>,
    console_bar: Arc<ProgressBar>,
    queue: Arc<Mutex<(usize, Vec<PathBuf>)>>,
    elapsed: Arc<Mutex<Duration>>,
}

impl TestRunnerState {
    fn new(test_files: Vec<PathBuf>) -> Self {
        let n_files = test_files.len();
        Self {
            n_errors: Arc::new(AtomicUsize::new(0)),
            console_bar: Arc::new(ProgressBar::with_draw_target(
                Some(n_files as u64),
                ProgressDrawTarget::stdout(),
            )),
            queue: Arc::new(Mutex::new((0usize, test_files))),
            elapsed: Arc::new(Mutex::new(Duration::ZERO)),
        }
    }

    fn next_test(&self) -> Option<PathBuf> {
        let (current_idx, queue) = &mut *self.queue.lock().unwrap();
        let idx = *current_idx;
        let test_path = queue.get(idx).cloned()?;
        *current_idx = idx + 1;
        Some(test_path)
    }
}

fn run_test_worker(state: TestRunnerState, config: TestRunnerConfig) -> Result<(), TestError> {
    loop {
        if !config.keep_going && state.n_errors.load(Ordering::SeqCst) > 0 {
            return Ok(());
        }

        let Some(test_path) = state.next_test() else {
            return Ok(());
        };

        let result = execute_test_suite(
            &test_path,
            &state.elapsed,
            config.trace,
            config.print_outcome,
        );

        state.console_bar.inc(1);

        if let Err(err) = result {
            state.n_errors.fetch_add(1, Ordering::SeqCst);
            if !config.keep_going {
                return Err(err);
            }
        }
    }
}

fn determine_thread_count(single_thread: bool, n_files: usize) -> usize {
    match (single_thread, std::thread::available_parallelism()) {
        (true, _) | (false, Err(_)) => 1,
        (false, Ok(n)) => n.get().min(n_files),
    }
}

/// Run all test files in parallel or single-threaded mode
///
/// # Arguments
/// * `test_files` - List of test files to execute
/// * `single_thread` - Force single-threaded execution
/// * `trace` - Enable EVM execution tracing
/// * `print_outcome` - Print test outcomes in JSON format
/// * `keep_going` - Continue running tests even if some fail
pub fn run(
    test_files: Vec<PathBuf>,
    single_thread: bool,
    trace: bool,
    print_outcome: bool,
    keep_going: bool,
) -> Result<(), TestError> {
    let config = TestRunnerConfig::new(single_thread, trace, print_outcome, keep_going);
    let n_files = test_files.len();
    let state = TestRunnerState::new(test_files);
    let num_threads = determine_thread_count(config.single_thread, n_files);

    // Spawn worker threads
    let mut handles = Vec::with_capacity(num_threads);
    for i in 0..num_threads {
        let state = state.clone();

        let thread = std::thread::Builder::new()
            .name(format!("runner-{i}"))
            .spawn(move || run_test_worker(state, config))
            .unwrap();

        handles.push(thread);
    }

    // Collect results from all threads
    let mut thread_errors = Vec::new();
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => thread_errors.push(e),
            Err(_) => thread_errors.push(TestError {
                name: format!("thread {i} panicked"),
                path: String::new(),
                kind: TestErrorKind::Panic,
            }),
        }
    }

    state.console_bar.finish();

    // Print summary
    println!(
        "Finished execution. Total CPU time: {:.6}s",
        state.elapsed.lock().unwrap().as_secs_f64()
    );

    let n_errors = state.n_errors.load(Ordering::SeqCst);
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
