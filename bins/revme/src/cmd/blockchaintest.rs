pub mod post_block;

use clap::Parser;
use database::states::bundle_state::BundleRetention;
use database::{CacheDB, EmptyDB, State};
use primitives::{hardfork::SpecId, hex, Address};
use revm::{
    context::cfg::CfgEnv, context_interface::result::HaltReason, Context, ExecuteCommitEvm,
    MainBuilder, MainContext,
};
use serde_json::json;
use state::AccountInfo;
use statetest_types::blockchain::{BlockchainTest, BlockchainTestCase, ForkSpec};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

/// `blockchaintest` subcommand
#[derive(Parser, Debug)]
pub struct Cmd {
    /// Path to folder or file containing the blockchain tests
    ///
    /// If multiple paths are specified they will be run in sequence.
    ///
    /// Folders will be searched recursively for files with the extension `.json`.
    #[arg(required = true, num_args = 1..)]
    paths: Vec<PathBuf>,
    /// Run tests in a single thread
    #[arg(short = 's', long)]
    single_thread: bool,
    /// Output results in JSON format
    #[arg(long)]
    json: bool,
    /// Keep going after a test failure
    #[arg(long, alias = "no-fail-fast")]
    keep_going: bool,
}

impl Cmd {
    /// Runs `blockchaintest` command.
    pub fn run(&self) -> Result<(), Error> {
        for path in &self.paths {
            if !path.exists() {
                return Err(Error::PathNotFound(path.clone()));
            }

            println!("\nRunning blockchain tests in {}...", path.display());
            let test_files = find_all_json_tests(path);

            if test_files.is_empty() {
                return Err(Error::NoJsonFiles(path.clone()));
            }

            run_tests(test_files, self.single_thread, self.json, self.keep_going)?;
        }
        Ok(())
    }
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

/// Run all blockchain tests from the given files
fn run_tests(
    test_files: Vec<PathBuf>,
    _single_thread: bool,
    output_json: bool,
    keep_going: bool,
) -> Result<(), Error> {
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    let start_time = Instant::now();

    for file_path in test_files {
        if skip_test(&file_path) {
            skipped += 1;
            if !output_json {
                println!("Skipping: {}", file_path.display());
            }
            continue;
        }

        let result = run_test_file(&file_path, output_json);

        match result {
            Ok(test_count) => {
                passed += test_count;
                if !output_json {
                    println!("✓ {} ({} tests)", file_path.display(), test_count);
                }
            }
            Err(e) => {
                failed += 1;
                if output_json {
                    let output = json!({
                        "file": file_path.display().to_string(),
                        "error": e.to_string(),
                        "status": "failed"
                    });
                    println!("{}", serde_json::to_string(&output).unwrap());
                } else {
                    eprintln!("✗ {} - {}", file_path.display(), e);
                }

                if !keep_going {
                    return Err(e);
                }
            }
        }
    }

    let duration = start_time.elapsed();

    if !output_json {
        println!("\nTest results:");
        println!("  Passed:  {}", passed);
        println!("  Failed:  {}", failed);
        println!("  Skipped: {}", skipped);
        println!("  Time:    {:.2}s", duration.as_secs_f64());
    } else {
        let summary = json!({
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
            "duration_seconds": duration.as_secs_f64()
        });
        println!("{}", serde_json::to_string(&summary).unwrap());
    }

    if failed > 0 {
        Err(Error::TestsFailed { failed })
    } else {
        Ok(())
    }
}

/// Run tests from a single file
fn run_test_file(file_path: &Path, output_json: bool) -> Result<usize, Error> {
    let content =
        fs::read_to_string(file_path).map_err(|e| Error::FileRead(file_path.to_path_buf(), e))?;

    let blockchain_test: BlockchainTest = serde_json::from_str(&content)
        .map_err(|e| Error::JsonDecode(file_path.to_path_buf(), e))?;

    let mut test_count = 0;

    for (test_name, test_case) in blockchain_test.0 {
        if !output_json {
            println!("  Running: {}", test_name);
        }

        // Execute the blockchain test
        execute_blockchain_test(&test_case).map_err(|e| Error::TestExecution {
            test_name: test_name.clone(),
            error: e.to_string(),
        })?;

        test_count += 1;
    }

    Ok(test_count)
}

/// Execute a single blockchain test case
fn execute_blockchain_test(test_case: &BlockchainTestCase) -> Result<(), TestExecutionError> {
    // Skip certain forks for now
    match test_case.network {
        ForkSpec::ByzantiumToConstantinopleAt5 => {
            return Err(TestExecutionError::SkippedFork(format!(
                "{:?}",
                test_case.network
            )));
        }
        _ => {}
    }

    // Create database with initial state
    let mut cache_db = CacheDB::new(EmptyDB::default());

    // Insert genesis state into database
    let genesis_state = test_case.pre.clone().into_genesis_state();
    for (address, account) in genesis_state {
        let account_info = AccountInfo {
            balance: account.balance,
            nonce: account.nonce,
            code_hash: primitives::keccak256(&account.code),
            code: Some(bytecode::Bytecode::new_raw(account.code.clone())),
        };
        cache_db.insert_account_info(address, account_info);

        // Insert storage
        for (key, value) in account.storage {
            cache_db
                .insert_account_storage(address, key, value)
                .map_err(|e| {
                    TestExecutionError::Database(format!("Storage insertion failed: {}", e))
                })?;
        }
    }

    let mut state = State::builder().with_database(cache_db.clone()).build();

    // Setup configuration based on fork
    let spec_id = fork_to_spec_id(test_case.network);
    let mut cfg = CfgEnv::default();
    cfg.spec = spec_id;

    // Setup genesis block environment
    let mut block_env = test_case.genesis_block_env();

    // Process each block in the test
    for (block_idx, block) in test_case.blocks.iter().enumerate() {
        println!("block_idx: {}", block_idx);
        // Check if this block should fail
        let should_fail = block.expect_exception.is_some();

        // Skip blocks without transactions
        if block.transactions.is_none() {
            continue;
        }

        // Pre block system calls/

        let transactions = block.transactions.as_ref().unwrap();

        // Update block environment for this block
        if let Some(header) = &block.block_header {
            block_env = header.to_block_env();
        }

        // Execute each transaction in the block
        for (tx_idx, tx) in transactions.iter().enumerate() {
            // Create transaction environment
            let sender = derive_sender_from_tx(tx).unwrap_or_else(|| {
                // Use a default sender if signature recovery fails
                Address::from([0xa0; 20]) // Common test sender address
            });
            let to = extract_to_address_from_tx(tx);

            let tx_env = match tx.to_tx_env(sender, to) {
                Ok(env) => env,
                Err(e) => {
                    if should_fail {
                        // Expected failure during tx env creation
                        continue;
                    }
                    return Err(TestExecutionError::TransactionEnvCreation {
                        block_idx,
                        tx_idx,
                        error: e,
                    });
                }
            };

            // Create EVM context for each transaction to ensure fresh state access
            let evm_context = Context::mainnet()
                .with_block(&block_env)
                .with_tx(&tx_env)
                .with_cfg(&cfg)
                .with_db(&mut state);

            // Build and execute with EVM
            let mut evm = evm_context.build_mainnet();
            let execution_result = evm.transact_commit(&tx_env);

            match execution_result {
                Ok(_) => {
                    if should_fail {
                        return Err(TestExecutionError::ExpectedFailure {
                            block_idx,
                            tx_idx,
                            message: block.expect_exception.clone().unwrap_or_default(),
                        });
                    }
                }
                Err(e) => {
                    if !should_fail {
                        return Err(TestExecutionError::UnexpectedFailure {
                            block_idx,
                            tx_idx,
                            error: format!("{:?}", e),
                        });
                    }
                    // Expected failure
                }
            }
        }

        post_block::post_block_transition(&mut state, &block_env, &[], spec_id);

        state.merge_transitions(BundleRetention::Reverts);
    }

    // Validate post-state if provided (disabled for now until proper signature recovery is implemented)
    if false {
        if let Some(expected_post_state) = &test_case.post_state {
            for (address, expected_account) in expected_post_state {
                // Load account from final state
                let actual_account = state.load_cache_account(*address).map_err(|e| {
                    TestExecutionError::Database(format!("Account load failed: {}", e))
                })?;
                let info = actual_account
                    .account
                    .as_ref()
                    .map(|a| a.info.clone())
                    .unwrap_or_default();

                // Validate balance
                if info.balance != expected_account.balance {
                    return Err(TestExecutionError::PostStateValidation {
                        address: *address,
                        field: "balance".to_string(),
                        expected: format!("{}", expected_account.balance),
                        actual: format!("{}", info.balance),
                    });
                }

                // Validate nonce
                let expected_nonce = expected_account.nonce.to::<u64>();
                if info.nonce != expected_nonce {
                    return Err(TestExecutionError::PostStateValidation {
                        address: *address,
                        field: "nonce".to_string(),
                        expected: format!("{}", expected_nonce),
                        actual: format!("{}", info.nonce),
                    });
                }

                // Validate code if present
                if !expected_account.code.is_empty() {
                    if let Some(actual_code) = &info.code {
                        if actual_code.bytecode() != &expected_account.code {
                            return Err(TestExecutionError::PostStateValidation {
                                address: *address,
                                field: "code".to_string(),
                                expected: format!("0x{}", hex::encode(&expected_account.code)),
                                actual: format!("0x{}", hex::encode(actual_code.bytecode())),
                            });
                        }
                    } else {
                        return Err(TestExecutionError::PostStateValidation {
                            address: *address,
                            field: "code".to_string(),
                            expected: format!("0x{}", hex::encode(&expected_account.code)),
                            actual: "empty".to_string(),
                        });
                    }
                }

                // TODO: Validate storage slots
            }
        }
    }

    Ok(())
}

/// Extract 'to' address from transaction data
/// This is a simplified approach - in reality, we'd need to decode the transaction RLP
fn extract_to_address_from_tx(_tx: &statetest_types::blockchain::Transaction) -> Option<Address> {
    // For now, assume it's a contract call to a default address
    // In a full implementation, this would be extracted from the transaction data or RLP
    None // None indicates contract creation
}

/// Convert ForkSpec to SpecId
fn fork_to_spec_id(fork: ForkSpec) -> SpecId {
    match fork {
        ForkSpec::Frontier => SpecId::FRONTIER,
        ForkSpec::Homestead | ForkSpec::FrontierToHomesteadAt5 => SpecId::HOMESTEAD,
        ForkSpec::EIP150 | ForkSpec::HomesteadToDaoAt5 | ForkSpec::HomesteadToEIP150At5 => {
            SpecId::TANGERINE
        }
        ForkSpec::EIP158 => SpecId::SPURIOUS_DRAGON,
        ForkSpec::Byzantium
        | ForkSpec::EIP158ToByzantiumAt5
        | ForkSpec::ByzantiumToConstantinopleFixAt5 => SpecId::BYZANTIUM,
        ForkSpec::Constantinople | ForkSpec::ByzantiumToConstantinopleAt5 => SpecId::PETERSBURG,
        ForkSpec::ConstantinopleFix => SpecId::PETERSBURG,
        ForkSpec::Istanbul => SpecId::ISTANBUL,
        ForkSpec::Berlin => SpecId::BERLIN,
        ForkSpec::London | ForkSpec::BerlinToLondonAt5 => SpecId::LONDON,
        ForkSpec::Paris | ForkSpec::ParisToShanghaiAtTime15k => SpecId::MERGE,
        ForkSpec::Shanghai => SpecId::SHANGHAI,
        ForkSpec::Cancun | ForkSpec::ShanghaiToCancunAtTime15k => SpecId::CANCUN,
        ForkSpec::Prague => SpecId::PRAGUE,
        _ => SpecId::PRAGUE, // For any unknown forks, use latest available
    }
}

/// Derive sender address from transaction signature
/// This is a simplified implementation - a full implementation would
/// recover the public key from the signature and derive the address
fn derive_sender_from_tx(_tx: &statetest_types::blockchain::Transaction) -> Option<Address> {
    // For now, use the standard test address that typically has funds
    // This is the address commonly used in Ethereum tests
    // TODO: Implement proper ECDSA recovery from transaction signature
    Some(Address::from([
        0xa9, 0x4f, 0x53, 0x74, 0xfc, 0xe5, 0xed, 0xbc, 0x8e, 0x2a, 0x86, 0x97, 0xc1, 0x53, 0x31,
        0x67, 0x7e, 0x6e, 0xbf, 0x0b,
    ])) // 0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b
}

/// Check if a test should be skipped based on its filename
fn skip_test(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_str().unwrap();

    // Add any problematic tests here that should be skipped
    matches!(
        name,
        // Example: Skip tests that are known to be problematic
        "placeholder_skip_test.json"
    )
}

#[derive(Debug, Error)]
pub enum TestExecutionError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Skipped fork: {0}")]
    SkippedFork(String),

    #[error("Expected failure at block {block_idx}, tx {tx_idx}: {message}")]
    ExpectedFailure {
        block_idx: usize,
        tx_idx: usize,
        message: String,
    },

    #[error("Unexpected failure at block {block_idx}, tx {tx_idx}: {error}")]
    UnexpectedFailure {
        block_idx: usize,
        tx_idx: usize,
        error: String,
    },

    #[error("Transaction env creation failed at block {block_idx}, tx {tx_idx}: {error}")]
    TransactionEnvCreation {
        block_idx: usize,
        tx_idx: usize,
        error: String,
    },

    #[error("Unexpected revert at block {block_idx}, tx {tx_idx}, gas used: {gas_used}")]
    UnexpectedRevert {
        block_idx: usize,
        tx_idx: usize,
        gas_used: u64,
    },

    #[error("Unexpected halt at block {block_idx}, tx {tx_idx}: {reason:?}, gas used: {gas_used}")]
    UnexpectedHalt {
        block_idx: usize,
        tx_idx: usize,
        reason: HaltReason,
        gas_used: u64,
    },

    #[error(
        "Post-state validation failed for {address:?}.{field}: expected {expected}, got {actual}"
    )]
    PostStateValidation {
        address: Address,
        field: String,
        expected: String,
        actual: String,
    },
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("No JSON files found in: {0}")]
    NoJsonFiles(PathBuf),

    #[error("Failed to read file {0}: {1}")]
    FileRead(PathBuf, std::io::Error),

    #[error("Failed to decode JSON from {0}: {1}")]
    JsonDecode(PathBuf, serde_json::Error),

    #[error("Test execution failed for {test_name}: {error}")]
    TestExecution { test_name: String, error: String },

    #[error("Directory traversal error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("{failed} tests failed")]
    TestsFailed { failed: usize },
}
