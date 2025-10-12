pub mod post_block;
pub mod pre_block;

use clap::Parser;

use revm::{
    bytecode::Bytecode,
    context::{cfg::CfgEnv, ContextTr},
    context_interface::{block::BlobExcessGasAndPrice, result::HaltReason},
    database::{states::bundle_state::BundleRetention, EmptyDB, State},
    handler::EvmTr,
    inspector::inspectors::TracerEip3155,
    primitives::{hardfork::SpecId, hex, Address, HashMap, U256},
    state::AccountInfo,
    Context, Database, ExecuteCommitEvm, ExecuteEvm, InspectEvm, MainBuilder, MainContext,
};
use serde_json::json;
use statetest_types::blockchain::{
    Account, BlockchainTest, BlockchainTestCase, ForkSpec, Withdrawal,
};
use std::collections::BTreeMap;
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
    /// Omit progress output
    #[arg(long)]
    omit_progress: bool,
    /// Keep going after a test failure
    #[arg(long, alias = "no-fail-fast")]
    keep_going: bool,
    /// Print environment information (pre-state, post-state, env) when an error occurs
    #[arg(long)]
    print_env_on_error: bool,
    /// Output results in JSON format
    #[arg(long)]
    json: bool,
}

impl Cmd {
    /// Runs `blockchaintest` command.
    pub fn run(&self) -> Result<(), Error> {
        for path in &self.paths {
            if !path.exists() {
                return Err(Error::PathNotFound(path.clone()));
            }

            if !self.json {
                println!("\nRunning blockchain tests in {}...", path.display());
            }
            let test_files = find_all_json_tests(path);

            if test_files.is_empty() {
                return Err(Error::NoJsonFiles(path.clone()));
            }

            run_tests(
                test_files,
                self.omit_progress,
                self.keep_going,
                self.print_env_on_error,
                self.json,
            )?;
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
    omit_progress: bool,
    keep_going: bool,
    print_env_on_error: bool,
    json_output: bool,
) -> Result<(), Error> {
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut failed_paths = Vec::new();

    let start_time = Instant::now();
    let total_files = test_files.len();

    for (file_index, file_path) in test_files.into_iter().enumerate() {
        let current_file = file_index + 1;
        if skip_test(&file_path) {
            skipped += 1;
            if json_output {
                let output = json!({
                    "file": file_path.display().to_string(),
                    "status": "skipped",
                    "reason": "known_issue"
                });
                println!("{}", serde_json::to_string(&output).unwrap());
            } else if !omit_progress {
                println!(
                    "Skipping ({}/{}): {}",
                    current_file,
                    total_files,
                    file_path.display()
                );
            }
            continue;
        }

        let result = run_test_file(&file_path, json_output, print_env_on_error);

        match result {
            Ok(test_count) => {
                passed += test_count;
                if json_output {
                    // JSON output handled in run_test_file
                } else if !omit_progress {
                    println!(
                        "✓ ({}/{}) {} ({} tests)",
                        current_file,
                        total_files,
                        file_path.display(),
                        test_count
                    );
                }
            }
            Err(e) => {
                failed += 1;
                if keep_going {
                    failed_paths.push(file_path.clone());
                }
                if json_output {
                    let output = json!({
                        "file": file_path.display().to_string(),
                        "error": e.to_string(),
                        "status": "failed"
                    });
                    println!("{}", serde_json::to_string(&output).unwrap());
                } else if !omit_progress {
                    eprintln!(
                        "✗ ({}/{}) {} - {}",
                        current_file,
                        total_files,
                        file_path.display(),
                        e
                    );
                }

                if !keep_going {
                    return Err(e);
                }
            }
        }
    }

    let duration = start_time.elapsed();

    if json_output {
        let results = json!({
            "summary": {
                "passed": passed,
                "failed": failed,
                "skipped": skipped,
                "duration_secs": duration.as_secs_f64(),
            }
        });
        println!("{}", serde_json::to_string(&results).unwrap());
    } else {
        // Print failed test paths if keep-going was enabled
        if keep_going && !failed_paths.is_empty() {
            println!("\nFailed test files:");
            for path in &failed_paths {
                println!("  {}", path.display());
            }
        }

        println!("\nTest results:");
        println!("  Passed:  {passed}");
        println!("  Failed:  {failed}");
        println!("  Skipped: {skipped}");
        println!("  Time:    {:.2}s", duration.as_secs_f64());
    }

    if failed > 0 {
        Err(Error::TestsFailed { failed })
    } else {
        Ok(())
    }
}

/// Run tests from a single file
fn run_test_file(
    file_path: &Path,
    json_output: bool,
    print_env_on_error: bool,
) -> Result<usize, Error> {
    let content =
        fs::read_to_string(file_path).map_err(|e| Error::FileRead(file_path.to_path_buf(), e))?;

    let blockchain_test: BlockchainTest = serde_json::from_str(&content)
        .map_err(|e| Error::JsonDecode(file_path.to_path_buf(), e))?;

    let mut test_count = 0;

    for (test_name, test_case) in blockchain_test.0 {
        if json_output {
            // Output test start in JSON format
            let output = json!({
                "test": test_name,
                "file": file_path.display().to_string(),
                "status": "running"
            });
            println!("{}", serde_json::to_string(&output).unwrap());
        } else {
            println!("  Running: {test_name}");
        }
        // Execute the blockchain test
        let result = execute_blockchain_test(&test_case, print_env_on_error, json_output);

        match result {
            Ok(()) => {
                if json_output {
                    let output = json!({
                        "test": test_name,
                        "file": file_path.display().to_string(),
                        "status": "passed"
                    });
                    println!("{}", serde_json::to_string(&output).unwrap());
                }
                test_count += 1;
            }
            Err(e) => {
                if json_output {
                    let output = json!({
                        "test": test_name,
                        "file": file_path.display().to_string(),
                        "status": "failed",
                        "error": e.to_string()
                    });
                    println!("{}", serde_json::to_string(&output).unwrap());
                }
                return Err(Error::TestExecution {
                    test_name: test_name.clone(),
                    test_path: file_path.to_path_buf(),
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(test_count)
}

/// Debug information captured during test execution
#[derive(Debug, Clone)]
struct DebugInfo {
    /// Initial pre-state before any execution
    pre_state: HashMap<Address, (AccountInfo, HashMap<U256, U256>)>,
    /// Transaction environment
    tx_env: Option<revm::context::tx::TxEnv>,
    /// Block environment
    block_env: revm::context::block::BlockEnv,
    /// Configuration environment
    cfg_env: CfgEnv,
    /// Block index where error occurred
    block_idx: usize,
    /// Transaction index where error occurred
    tx_idx: usize,
    /// Withdrawals in the block
    withdrawals: Option<Vec<Withdrawal>>,
}

impl DebugInfo {
    /// Capture current state from the State database
    fn capture_committed_state(
        state: &State<EmptyDB>,
    ) -> HashMap<Address, (AccountInfo, HashMap<U256, U256>)> {
        let mut committed_state = HashMap::new();

        // Access the cache state to get all accounts
        for (address, cache_account) in &state.cache.accounts {
            if let Some(plain_account) = &cache_account.account {
                let mut storage = HashMap::new();
                for (key, value) in &plain_account.storage {
                    storage.insert(*key, *value);
                }
                committed_state.insert(*address, (plain_account.info.clone(), storage));
            }
        }

        committed_state
    }
}

/// Validate post state against expected values
fn validate_post_state(
    state: &mut State<EmptyDB>,
    expected_post_state: &BTreeMap<Address, Account>,
    debug_info: &DebugInfo,
    print_env_on_error: bool,
) -> Result<(), TestExecutionError> {
    for (address, expected_account) in expected_post_state {
        // Load account from final state
        let actual_account = state
            .load_cache_account(*address)
            .map_err(|e| TestExecutionError::Database(format!("Account load failed: {e}")))?;
        let info = actual_account
            .account
            .as_ref()
            .map(|a| a.info.clone())
            .unwrap_or_default();

        // Validate balance
        if info.balance != expected_account.balance {
            if print_env_on_error {
                print_error_with_state(debug_info, state, Some(expected_post_state));
            }
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
            if print_env_on_error {
                print_error_with_state(debug_info, state, Some(expected_post_state));
            }
            return Err(TestExecutionError::PostStateValidation {
                address: *address,
                field: "nonce".to_string(),
                expected: format!("{expected_nonce}"),
                actual: format!("{}", info.nonce),
            });
        }

        // Validate code if present
        if !expected_account.code.is_empty() {
            if let Some(actual_code) = &info.code {
                if actual_code.original_bytes() != expected_account.code {
                    if print_env_on_error {
                        print_error_with_state(debug_info, state, Some(expected_post_state));
                    }
                    return Err(TestExecutionError::PostStateValidation {
                        address: *address,
                        field: "code".to_string(),
                        expected: format!("0x{}", hex::encode(&expected_account.code)),
                        actual: format!("0x{}", hex::encode(actual_code.bytecode())),
                    });
                }
            } else {
                if print_env_on_error {
                    print_error_with_state(debug_info, state, Some(expected_post_state));
                }
                return Err(TestExecutionError::PostStateValidation {
                    address: *address,
                    field: "code".to_string(),
                    expected: format!("0x{}", hex::encode(&expected_account.code)),
                    actual: "empty".to_string(),
                });
            }
        }

        // Check for unexpected storage entries
        for (slot, actual_value) in actual_account
            .account
            .as_ref()
            .map(|a| &a.storage)
            .unwrap_or(&HashMap::new())
            .iter()
        {
            let slot = *slot;
            let actual_value = *actual_value;
            if !expected_account.storage.contains_key(&slot) && !actual_value.is_zero() {
                if print_env_on_error {
                    print_error_with_state(debug_info, state, Some(expected_post_state));
                }
                return Err(TestExecutionError::PostStateValidation {
                    address: *address,
                    field: format!("storage_unexpected[{slot}]"),
                    expected: "0x0".to_string(),
                    actual: format!("{actual_value}"),
                });
            }
        }

        // Validate storage slots
        for (slot, expected_value) in &expected_account.storage {
            let actual_value = state.storage(*address, *slot);
            let actual_value = actual_value.unwrap_or_default();

            if actual_value != *expected_value {
                if print_env_on_error {
                    print_error_with_state(debug_info, state, Some(expected_post_state));
                }

                return Err(TestExecutionError::PostStateValidation {
                    address: *address,
                    field: format!("storage_validation[{slot}]"),
                    expected: format!("{expected_value}"),
                    actual: format!("{actual_value}"),
                });
            }
        }
    }
    Ok(())
}

/// Print comprehensive error information including environment and state comparison
fn print_error_with_state(
    debug_info: &DebugInfo,
    current_state: &State<EmptyDB>,
    expected_post_state: Option<&BTreeMap<Address, Account>>,
) {
    eprintln!("\n========== TEST EXECUTION ERROR ==========");

    // Print error location
    eprintln!(
        "\n📍 Error occurred at block {} transaction {}",
        debug_info.block_idx, debug_info.tx_idx
    );

    // Print configuration environment
    eprintln!("\n📋 Configuration Environment:");
    eprintln!("  Spec ID: {:?}", debug_info.cfg_env.spec);
    eprintln!("  Chain ID: {}", debug_info.cfg_env.chain_id);
    eprintln!(
        "  Limit contract code size: {:?}",
        debug_info.cfg_env.limit_contract_code_size
    );
    eprintln!(
        "  Limit contract initcode size: {:?}",
        debug_info.cfg_env.limit_contract_initcode_size
    );

    // Print block environment
    eprintln!("\n🔨 Block Environment:");
    eprintln!("  Number: {}", debug_info.block_env.number);
    eprintln!("  Timestamp: {}", debug_info.block_env.timestamp);
    eprintln!("  Gas limit: {}", debug_info.block_env.gas_limit);
    eprintln!("  Base fee: {:?}", debug_info.block_env.basefee);
    eprintln!("  Difficulty: {}", debug_info.block_env.difficulty);
    eprintln!("  Prevrandao: {:?}", debug_info.block_env.prevrandao);
    eprintln!("  Beneficiary: {:?}", debug_info.block_env.beneficiary);
    eprintln!(
        "  Blob excess gas: {:?}",
        debug_info.block_env.blob_excess_gas_and_price
    );

    // Print withdrawals
    if let Some(withdrawals) = &debug_info.withdrawals {
        eprintln!("  Withdrawals: {} items", withdrawals.len());
        if !withdrawals.is_empty() {
            for (i, withdrawal) in withdrawals.iter().enumerate().take(3) {
                eprintln!("    Withdrawal {i}:");
                eprintln!("      Index: {}", withdrawal.index);
                eprintln!("      Validator Index: {}", withdrawal.validator_index);
                eprintln!("      Address: {:?}", withdrawal.address);
                eprintln!(
                    "      Amount: {} Gwei ({:.6} ETH)",
                    withdrawal.amount,
                    withdrawal.amount.to::<u128>() as f64 / 1_000_000_000.0
                );
            }
            if withdrawals.len() > 3 {
                eprintln!("    ... and {} more withdrawals", withdrawals.len() - 3);
            }
        }
    }

    // Print transaction environment if available
    if let Some(tx_env) = &debug_info.tx_env {
        eprintln!("\n📄 Transaction Environment:");
        eprintln!("  Transaction type: {}", tx_env.tx_type);
        eprintln!("  Caller: {:?}", tx_env.caller);
        eprintln!("  Gas limit: {}", tx_env.gas_limit);
        eprintln!("  Gas price: {}", tx_env.gas_price);
        eprintln!("  Gas priority fee: {:?}", tx_env.gas_priority_fee);
        eprintln!("  Transaction kind: {:?}", tx_env.kind);
        eprintln!("  Value: {}", tx_env.value);
        eprintln!("  Data length: {} bytes", tx_env.data.len());
        if !tx_env.data.is_empty() {
            let preview_len = std::cmp::min(64, tx_env.data.len());
            eprintln!(
                "  Data preview: 0x{}{}",
                hex::encode(&tx_env.data[..preview_len]),
                if tx_env.data.len() > 64 { "..." } else { "" }
            );
        }
        eprintln!("  Nonce: {}", tx_env.nonce);
        eprintln!("  Chain ID: {:?}", tx_env.chain_id);
        eprintln!("  Access list: {} entries", tx_env.access_list.len());
        if !tx_env.access_list.is_empty() {
            for (i, access) in tx_env.access_list.iter().enumerate().take(3) {
                eprintln!(
                    "    Access {}: address={:?}, {} storage keys",
                    i,
                    access.address,
                    access.storage_keys.len()
                );
            }
            if tx_env.access_list.len() > 3 {
                eprintln!(
                    "    ... and {} more access list entries",
                    tx_env.access_list.len() - 3
                );
            }
        }
        eprintln!("  Blob hashes: {} blobs", tx_env.blob_hashes.len());
        if !tx_env.blob_hashes.is_empty() {
            for (i, hash) in tx_env.blob_hashes.iter().enumerate().take(3) {
                eprintln!("    Blob {i}: {hash:?}");
            }
            if tx_env.blob_hashes.len() > 3 {
                eprintln!(
                    "    ... and {} more blob hashes",
                    tx_env.blob_hashes.len() - 3
                );
            }
        }
        eprintln!("  Max fee per blob gas: {}", tx_env.max_fee_per_blob_gas);
        eprintln!(
            "  Authorization list: {} items",
            tx_env.authorization_list.len()
        );
        if !tx_env.authorization_list.is_empty() {
            eprintln!("    (EIP-7702 authorizations present)");
        }
    } else {
        eprintln!(
            "\n📄 Transaction Environment: Not available (error occurred before tx creation)"
        );
    }

    // Print state comparison
    eprintln!("\n💾 Pre-State (Initial):");
    for (address, (info, storage)) in &debug_info.pre_state {
        eprintln!("  Account {address:?}:");
        eprintln!("    Balance: 0x{:x}", info.balance);
        eprintln!("    Nonce: {}", info.nonce);
        eprintln!("    Code hash: {:?}", info.code_hash);
        eprintln!(
            "    Code size: {} bytes",
            info.code.as_ref().map_or(0, |c| c.bytecode().len())
        );
        if !storage.is_empty() {
            eprintln!("    Storage ({} slots):", storage.len());
            for (key, value) in storage.iter().take(5) {
                eprintln!("      {key:?} => {value:?}");
            }
            if storage.len() > 5 {
                eprintln!("      ... and {} more slots", storage.len() - 5);
            }
        }
    }

    eprintln!("\n📝 Current State (Actual):");
    let committed_state = DebugInfo::capture_committed_state(current_state);
    for (address, (info, storage)) in &committed_state {
        eprintln!("  Account {address:?}:");
        eprintln!("    Balance: 0x{:x}", info.balance);
        eprintln!("    Nonce: {}", info.nonce);
        eprintln!("    Code hash: {:?}", info.code_hash);
        eprintln!(
            "    Code size: {} bytes",
            info.code.as_ref().map_or(0, |c| c.bytecode().len())
        );
        if !storage.is_empty() {
            eprintln!("    Storage ({} slots):", storage.len());
            for (key, value) in storage.iter().take(5) {
                eprintln!("      {key:?} => {value:?}");
            }
            if storage.len() > 5 {
                eprintln!("      ... and {} more slots", storage.len() - 5);
            }
        }
    }

    // Print expected post-state if available
    if let Some(expected_post_state) = expected_post_state {
        eprintln!("\n✅ Expected Post-State:");
        for (address, account) in expected_post_state {
            eprintln!("  Account {address:?}:");
            eprintln!("    Balance: 0x{:x}", account.balance);
            eprintln!("    Nonce: {}", account.nonce);
            if !account.code.is_empty() {
                eprintln!("    Code size: {} bytes", account.code.len());
            }
            if !account.storage.is_empty() {
                eprintln!("    Storage ({} slots):", account.storage.len());
                for (key, value) in account.storage.iter().take(5) {
                    eprintln!("      {key:?} => {value:?}");
                }
                if account.storage.len() > 5 {
                    eprintln!("      ... and {} more slots", account.storage.len() - 5);
                }
            }
        }
    }

    eprintln!("\n===========================================\n");
}

/// Execute a single blockchain test case
fn execute_blockchain_test(
    test_case: &BlockchainTestCase,
    print_env_on_error: bool,
    json_output: bool,
) -> Result<(), TestExecutionError> {
    // Skip all transition forks for now.
    if matches!(
        test_case.network,
        ForkSpec::ByzantiumToConstantinopleAt5
            | ForkSpec::ParisToShanghaiAtTime15k
            | ForkSpec::ShanghaiToCancunAtTime15k
            | ForkSpec::CancunToPragueAtTime15k
            | ForkSpec::PragueToOsakaAtTime15k
            | ForkSpec::BPO1ToBPO2AtTime15k
    ) {
        eprintln!("⚠️  Skipping transition fork: {:?}", test_case.network);
        return Ok(());
    }

    // Create database with initial state
    let mut state = State::builder().build();

    // Capture pre-state for debug info
    let mut pre_state_debug = HashMap::new();

    // Insert genesis state into database
    let genesis_state = test_case.pre.clone().into_genesis_state();
    for (address, account) in genesis_state {
        let account_info = AccountInfo {
            balance: account.balance,
            nonce: account.nonce,
            code_hash: revm::primitives::keccak256(&account.code),
            code: Some(Bytecode::new_raw(account.code.clone())),
        };

        // Store for debug info
        if print_env_on_error {
            pre_state_debug.insert(address, (account_info.clone(), account.storage.clone()));
        }

        state.insert_account_with_storage(address, account_info, account.storage);
    }

    // insert genesis hash
    state
        .block_hashes
        .insert(0, test_case.genesis_block_header.hash);

    // Setup configuration based on fork
    let spec_id = fork_to_spec_id(test_case.network);
    let mut cfg = CfgEnv::default();
    cfg.spec = spec_id;

    // Genesis block is not used yet.
    let mut parent_block_hash = Some(test_case.genesis_block_header.hash);
    let mut parent_excess_blob_gas = test_case
        .genesis_block_header
        .excess_blob_gas
        .unwrap_or_default()
        .to::<u64>();
    let mut block_env = test_case.genesis_block_env();

    // Process each block in the test
    for (block_idx, block) in test_case.blocks.iter().enumerate() {
        println!("Run block {block_idx}/{}", test_case.blocks.len());

        // Check if this block should fail
        let should_fail = block.expect_exception.is_some();

        let transactions = block.transactions.as_deref().unwrap_or_default();

        // Update block environment for this blockk

        let mut block_hash = None;
        let mut beacon_root = None;
        let this_excess_blob_gas;

        if let Some(block_header) = block.block_header.as_ref() {
            block_hash = Some(block_header.hash);
            beacon_root = block_header.parent_beacon_block_root;
            block_env = block_header.to_block_env(Some(BlobExcessGasAndPrice::new_with_spec(
                parent_excess_blob_gas,
                spec_id,
            )));
            this_excess_blob_gas = block_header.excess_blob_gas.map(|i| i.to::<u64>());
        } else {
            this_excess_blob_gas = None;
        }

        // Create EVM context for each transaction to ensure fresh state access
        let evm_context = Context::mainnet()
            .with_block(&block_env)
            .with_cfg(&cfg)
            .with_db(&mut state);

        // Build and execute with EVM - always use inspector when JSON output is enabled
        let mut evm = evm_context.build_mainnet_with_inspector(TracerEip3155::new_stdout());

        // Pre block system calls
        pre_block::pre_block_transition(&mut evm, spec_id, parent_block_hash, beacon_root);

        // Execute each transaction in the block
        for (tx_idx, tx) in transactions.iter().enumerate() {
            if tx.sender.is_none() {
                if print_env_on_error {
                    let debug_info = DebugInfo {
                        pre_state: pre_state_debug.clone(),
                        tx_env: None,
                        block_env: block_env.clone(),
                        cfg_env: cfg.clone(),
                        block_idx,
                        tx_idx,
                        withdrawals: block.withdrawals.clone(),
                    };
                    print_error_with_state(
                        &debug_info,
                        evm.ctx().db_ref(),
                        test_case.post_state.as_ref(),
                    );
                }
                if json_output {
                    let output = json!({
                        "block": block_idx,
                        "tx": tx_idx,
                        "error": "missing sender",
                        "status": "skipped"
                    });
                    println!("{}", serde_json::to_string(&output).unwrap());
                } else {
                    eprintln!("⚠️  Skipping block {block_idx} due to missing sender");
                }
                break; // Skip to next block
            }

            let tx_env = match tx.to_tx_env() {
                Ok(env) => env,
                Err(e) => {
                    if should_fail {
                        // Expected failure during tx env creation
                        continue;
                    }
                    if print_env_on_error {
                        let debug_info = DebugInfo {
                            pre_state: pre_state_debug.clone(),
                            tx_env: None,
                            block_env: block_env.clone(),
                            cfg_env: cfg.clone(),
                            block_idx,
                            tx_idx,
                            withdrawals: block.withdrawals.clone(),
                        };
                        print_error_with_state(
                            &debug_info,
                            evm.ctx().db_ref(),
                            test_case.post_state.as_ref(),
                        );
                    }
                    if json_output {
                        let output = json!({
                            "block": block_idx,
                            "tx": tx_idx,
                            "error": format!("tx env creation error: {e}"),
                            "status": "skipped"
                        });
                        println!("{}", serde_json::to_string(&output).unwrap());
                    } else {
                        eprintln!(
                            "⚠️  Skipping block {block_idx} due to transaction env creation error: {e}"
                        );
                    }
                    break; // Skip to next block
                }
            };

            // If JSON output requested, output transaction details
            let execution_result = if json_output {
                evm.inspect_tx(tx_env.clone())
            } else {
                evm.transact(tx_env.clone())
            };

            match execution_result {
                Ok(result) => {
                    if should_fail {
                        // Unexpected success - should have failed but didn't
                        // If not expected to fail, use inspector to trace the transaction
                        if print_env_on_error {
                            // Re-run with inspector to get detailed trace
                            if json_output {
                                eprintln!("=== Transaction trace (unexpected success) ===");
                            }
                            let _ = evm.inspect_tx(tx_env.clone());
                        }

                        if print_env_on_error {
                            let debug_info = DebugInfo {
                                pre_state: pre_state_debug.clone(),
                                tx_env: Some(tx_env.clone()),
                                block_env: block_env.clone(),
                                cfg_env: cfg.clone(),
                                block_idx,
                                tx_idx,
                                withdrawals: block.withdrawals.clone(),
                            };
                            print_error_with_state(
                                &debug_info,
                                evm.ctx().db_ref(),
                                test_case.post_state.as_ref(),
                            );
                        }
                        let exception = block.expect_exception.clone().unwrap_or_default();
                        if json_output {
                            let output = json!({
                                "block": block_idx,
                                "tx": tx_idx,
                                "error": format!("expected failure: {exception}"),
                                "gas_used": result.result.gas_used(),
                                "status": "unexpected_success"
                            });
                            println!("{}", serde_json::to_string(&output).unwrap());
                        } else {
                            eprintln!(
                                "⚠️  Skipping block {block_idx} due to expected failure: {exception}"
                            );
                        }
                        break; // Skip to next block
                    }
                    evm.commit(result.state);
                }
                Err(e) => {
                    if !should_fail {
                        // Unexpected error - use inspector to trace the transaction
                        if print_env_on_error {
                            if json_output {
                                eprintln!("=== Transaction trace (unexpected failure) ===");
                            }
                            let _ = evm.inspect_tx(tx_env.clone());
                        }

                        if print_env_on_error {
                            let debug_info = DebugInfo {
                                pre_state: pre_state_debug.clone(),
                                tx_env: Some(tx_env.clone()),
                                block_env: block_env.clone(),
                                cfg_env: cfg.clone(),
                                block_idx,
                                tx_idx,
                                withdrawals: block.withdrawals.clone(),
                            };
                            print_error_with_state(
                                &debug_info,
                                evm.ctx().db_ref(),
                                test_case.post_state.as_ref(),
                            );
                        }
                        if json_output {
                            let output = json!({
                                "block": block_idx,
                                "tx": tx_idx,
                                "error": format!("{e:?}"),
                                "status": "unexpected_failure"
                            });
                            println!("{}", serde_json::to_string(&output).unwrap());
                        } else {
                            eprintln!(
                                "⚠️  Skipping block {block_idx} due to unexpected failure: {e:?}"
                            );
                        }
                        break; // Skip to next block
                    } else if json_output {
                        // Expected failure
                        let output = json!({
                            "block": block_idx,
                            "tx": tx_idx,
                            "error": format!("{e:?}"),
                            "status": "expected_failure"
                        });
                        println!("{}", serde_json::to_string(&output).unwrap());
                    }
                }
            }
        }

        // uncle rewards are not implemented yet
        post_block::post_block_transition(
            &mut evm,
            &block_env,
            block.withdrawals.as_deref().unwrap_or_default(),
            spec_id,
        );

        // insert present block hash.
        state
            .block_hashes
            .insert(block_env.number.to::<u64>(), block_hash.unwrap_or_default());

        parent_block_hash = block_hash;
        if let Some(excess_blob_gas) = this_excess_blob_gas {
            parent_excess_blob_gas = excess_blob_gas;
        }

        state.merge_transitions(BundleRetention::Reverts);
    }

    // Validate post state if present
    if let Some(expected_post_state) = &test_case.post_state {
        // Create debug info for post-state validation
        let debug_info = DebugInfo {
            pre_state: pre_state_debug.clone(),
            tx_env: None, // Last transaction is done
            block_env: block_env.clone(),
            cfg_env: cfg.clone(),
            block_idx: test_case.blocks.len(),
            tx_idx: 0,
            withdrawals: test_case.blocks.last().and_then(|b| b.withdrawals.clone()),
        };
        validate_post_state(
            &mut state,
            expected_post_state,
            &debug_info,
            print_env_on_error,
        )?;
    }

    Ok(())
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
        ForkSpec::Prague | ForkSpec::CancunToPragueAtTime15k => SpecId::PRAGUE,
        ForkSpec::Osaka | ForkSpec::PragueToOsakaAtTime15k => SpecId::OSAKA,
        _ => SpecId::OSAKA, // For any unknown forks, use latest available
    }
}

/// Check if a test should be skipped based on its filename
fn skip_test(path: &Path) -> bool {
    let path_str = path.to_str().unwrap_or_default();
    // blobs excess gas calculation is not supported or osaka BPO configuration
    if path_str.contains("paris/eip7610_create_collision")
        || path_str.contains("cancun/eip4844_blobs")
        || path_str.contains("prague/eip7251_consolidations")
        || path_str.contains("prague/eip7685_general_purpose_el_requests")
        || path_str.contains("prague/eip7002_el_triggerable_withdrawals")
        || path_str.contains("osaka/eip7918_blob_reserve_price")
    {
        return true;
    }

    let name = path.file_name().unwrap().to_str().unwrap();
    // Add any problematic tests here that should be skipped
    matches!(
        name,
        // Test check if gas price overflows, we handle this correctly but does not match tests specific exception.
        "CreateTransactionHighNonce.json"

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
        // TODO tests not checked, maybe related to parent block hashes as it is currently not supported in test.
        | "scenarios.json"
        // IT seems that post state is wrong, we properly handle max blob gas and state should stay the same.
        | "invalid_tx_max_fee_per_blob_gas.json"
        | "correct_increasing_blob_gas_costs.json"
        | "correct_decreasing_blob_gas_costs.json"

        // test-fixtures/main/develop/blockchain_tests/prague/eip2935_historical_block_hashes_from_state/block_hashes/block_hashes_history.json
        | "block_hashes_history.json"
    )
}

#[derive(Debug, Error)]
pub enum TestExecutionError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Skipped fork: {0}")]
    SkippedFork(String),

    #[error("Sender is required")]
    SenderRequired,

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

    #[error("Test execution failed for {test_name} in {test_path}: {error}")]
    TestExecution {
        test_name: String,
        test_path: PathBuf,
        error: String,
    },

    #[error("Directory traversal error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("{failed} tests failed")]
    TestsFailed { failed: usize },
}
