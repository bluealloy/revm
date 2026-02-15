use super::runner::skip_test;
use criterion::{BatchSize, Criterion};
use revm::{
    context::{block::BlockEnv, cfg::CfgEnv, tx::TxEnv},
    database::{self, CacheState},
    primitives::{hardfork::SpecId, U256},
    statetest_types::{SpecName, Test, TestSuite, TestUnit},
    Context, ExecuteCommitEvm, MainBuilder, MainContext,
};
use std::path::{Path, PathBuf};

/// Configuration for benchmark execution
struct BenchConfig {
    cfg: CfgEnv,
    block: BlockEnv,
    tx: TxEnv,
    cache_state: CacheState,
}

impl BenchConfig {
    /// Create a new benchmark configuration from test unit and test
    fn new(unit: &TestUnit, test: &Test, spec_name: &SpecName) -> Option<Self> {
        // Setup base configuration
        let mut cfg = CfgEnv::default();
        cfg.chain_id = unit
            .env
            .current_chain_id
            .unwrap_or(U256::ONE)
            .try_into()
            .unwrap_or(1);

        cfg.spec = spec_name.to_spec_id();

        // Configure max blobs per spec
        if cfg.spec.is_enabled_in(SpecId::OSAKA) {
            cfg.set_max_blobs_per_tx(6);
        } else if cfg.spec.is_enabled_in(SpecId::PRAGUE) {
            cfg.set_max_blobs_per_tx(9);
        } else {
            cfg.set_max_blobs_per_tx(6);
        }

        // Setup block environment
        let block = unit.block_env(&mut cfg);

        // Setup transaction environment
        let tx = match test.tx_env(unit) {
            Ok(tx) => tx,
            Err(_) => return None,
        };

        // Prepare initial state
        let cache_state = unit.state();

        Some(Self {
            cfg,
            block,
            tx,
            cache_state,
        })
    }
}

/// Execute a single benchmark iteration
fn execute_bench_iteration(config: &BenchConfig) {
    // Clone fresh state (Must clone because `transact_commit` modifies state)
    let mut cache = config.cache_state.clone(); // Clones the pre-state
    cache.set_state_clear_flag(config.cfg.spec.is_enabled_in(SpecId::SPURIOUS_DRAGON));

    // Build state database
    let mut state = database::State::builder()
        .with_cached_prestate(cache)
        .with_bundle_update()
        .build();

    // Build EVM instance
    let mut evm = Context::mainnet()
        .with_block(&config.block) // block number, timestamp, coinbase, etc.
        .with_tx(&config.tx) // caller, value, data, gas limit, etc.
        .with_cfg(&config.cfg) // chain_id, spec_id (Cancun, Prague, etc.)
        .with_db(&mut state)
        .build_mainnet();

    // Execute transaction and commit state changes
    let _ = evm.transact_commit(&config.tx);

    // Benchmarks measure execution speed, not correctness
}

/// Result type for benchmarking files
enum BenchmarkResult {
    /// Successfully benchmarked
    Success,
    /// File is not a state test (e.g., difficulty test)
    /// or filtered out by `skip_test` function
    Skip,
    /// Actual error during benchmarking
    Error(Box<dyn std::error::Error>),
}

/// Check if a deserialization error indicates a non-state-test file
///
/// This function detects when a JSON file cannot be deserialized as a state test
/// because it's missing required fields like `env`, `pre`, `post`, or `transaction`.
/// This typically indicates the file is a different type of test (e.g., difficulty test)
/// rather than a state test.
///
/// # Arguments
///
/// * `error` - The serde JSON deserialization error
///
/// # Returns
///
/// `true` if the error indicates a non-state-test file, `false` otherwise
fn is_non_state_test_error(error: &serde_json::Error) -> bool {
    // Check if the error message indicates missing required fields like "env"
    // State tests require these fields, but other test types (like difficulty tests) don't have them
    let error_msg = error.to_string();
    error_msg.contains("missing field")
        && (error_msg.contains("`env`")
            || error_msg.contains("`pre`")
            || error_msg.contains("`post`")
            || error_msg.contains("`transaction`"))
}

/// Benchmark a single test file
fn benchmark_test_file(criterion: &mut Criterion, path: &Path) -> BenchmarkResult {
    if skip_test(path) {
        return BenchmarkResult::Skip;
    }

    let s = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return BenchmarkResult::Error(Box::new(e)),
    };

    let suite: TestSuite = match serde_json::from_str(&s) {
        Ok(suite) => suite,
        Err(e) => {
            // Check if this is a non-state-test file (like difficulty tests)
            if is_non_state_test_error(&e) {
                return BenchmarkResult::Skip;
            }
            return BenchmarkResult::Error(Box::new(e));
        }
    };

    let Some(group_name) = path.parent().and_then(|p| p.as_os_str().to_str()) else {
        return BenchmarkResult::Error(Box::new(std::io::Error::other("Invalid group name")));
    };
    let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
        return BenchmarkResult::Error(Box::new(std::io::Error::other("Invalid file name")));
    };
    for (_name, test_unit) in suite.0 {
        // Benchmark only the first valid spec/test to avoid excessive runs
        for (spec_name, tests) in &test_unit.post {
            // Skip Constantinople spec never actually deployed on Ethereum mainnet)
            // Refer to the SpecName enum documentation for more details
            if *spec_name == SpecName::Constantinople {
                continue;
            }

            // Take first test that we can create a valid config for
            for test in tests {
                if let Some(config) = BenchConfig::new(&test_unit, test, spec_name) {
                    let mut criterion_group = criterion.benchmark_group(group_name);
                    criterion_group.bench_function(file_name, |b| {
                        b.iter_batched(|| &config, execute_bench_iteration, BatchSize::SmallInput);
                    });
                    criterion_group.finish();

                    // Only benchmark first valid test per test unit
                    return BenchmarkResult::Success;
                }
            }
        }
    }

    BenchmarkResult::Success
}

/// Run benchmarks on all test files
pub fn run_benchmarks(test_files: Vec<PathBuf>, warmup: Option<u64>, time: Option<u64>) {
    let mut criterion = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(warmup.unwrap_or(300)))
        .measurement_time(std::time::Duration::from_secs(time.unwrap_or(2)))
        .without_plots();

    let mut success_count = 0;
    let mut skip_count = 0;
    let mut error_count = 0;

    for path in &test_files {
        match benchmark_test_file(&mut criterion, path) {
            BenchmarkResult::Success => success_count += 1,
            BenchmarkResult::Skip => {
                skip_count += 1;
            }
            BenchmarkResult::Error(e) => {
                eprintln!("Failed to benchmark {}: {}", path.display(), e);
                error_count += 1;
            }
        }
    }

    println!(
        "\nBenchmark summary: {} succeeded, {} skipped, {} failed out of {} total",
        success_count,
        skip_count,
        error_count,
        test_files.len()
    );
}
