pub mod post_block;
pub mod pre_block;

use clap::Parser;
use context::ContextTr;
use database::states::bundle_state::BundleRetention;
use database::{EmptyDB, State};
use inspector::inspectors::TracerEip3155;
use primitives::B256;
use primitives::{hardfork::SpecId, hex, Address, HashMap, U256};
use revm::{
    context::cfg::CfgEnv, context_interface::result::HaltReason, Context, ExecuteCommitEvm,
    MainBuilder, MainContext,
};
use revm::{Database, InspectEvm};
use serde_json::json;
use state::AccountInfo;
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

            run_tests(
                test_files,
                self.omit_progress,
                self.keep_going,
                self.print_env_on_error,
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
) -> Result<(), Error> {
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    let start_time = Instant::now();
    let total_files = test_files.len();

    for (file_index, file_path) in test_files.into_iter().enumerate() {
        let current_file = file_index + 1;
        if skip_test(&file_path) {
            skipped += 1;
            if !omit_progress {
                println!(
                    "Skipping ({}/{}): {}",
                    current_file,
                    total_files,
                    file_path.display()
                );
            }
            continue;
        }

        let result = run_test_file(&file_path, omit_progress, print_env_on_error);

        match result {
            Ok(test_count) => {
                passed += test_count;
                if !omit_progress {
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
                if omit_progress {
                    let output = json!({
                        "file": file_path.display().to_string(),
                        "error": e.to_string(),
                        "status": "failed"
                    });
                    println!("{}", serde_json::to_string(&output).unwrap());
                } else {
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

    println!("\nTest results:");
    println!("  Passed:  {passed}");
    println!("  Failed:  {failed}");
    println!("  Skipped: {skipped}");
    println!("  Time:    {:.2}s", duration.as_secs_f64());

    if failed > 0 {
        Err(Error::TestsFailed { failed })
    } else {
        Ok(())
    }
}

/// Run tests from a single file
fn run_test_file(
    file_path: &Path,
    output_json: bool,
    print_env_on_error: bool,
) -> Result<usize, Error> {
    let content =
        fs::read_to_string(file_path).map_err(|e| Error::FileRead(file_path.to_path_buf(), e))?;

    let blockchain_test: BlockchainTest = serde_json::from_str(&content)
        .map_err(|e| Error::JsonDecode(file_path.to_path_buf(), e))?;

    let mut test_count = 0;

    for (test_name, test_case) in blockchain_test.0 {
        if !output_json {
            println!("  Running: {test_name}");
        }

        // Execute the blockchain test
        execute_blockchain_test(&test_case, print_env_on_error).map_err(|e| {
            Error::TestExecution {
                test_name: test_name.clone(),
                test_path: file_path.to_path_buf(),
                error: e.to_string(),
            }
        })?;

        test_count += 1;
    }

    Ok(test_count)
}

/// Debug information captured during test execution
#[derive(Debug, Clone)]
struct DebugInfo {
    /// Initial pre-state before any execution
    pre_state: HashMap<Address, (AccountInfo, HashMap<U256, U256>)>,
    /// Current committed state
    committed_state: HashMap<Address, (AccountInfo, HashMap<U256, U256>)>,
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

    fn print(&self) {
        eprintln!("\n========== DEBUG INFORMATION ==========");
        eprintln!(
            "\n📍 Error occurred at block {} transaction {}",
            self.block_idx, self.tx_idx
        );

        eprintln!("\n📋 Configuration Environment:");
        eprintln!("  Spec ID: {:?}", self.cfg_env.spec);
        eprintln!("  Chain ID: {}", self.cfg_env.chain_id);
        eprintln!(
            "  Limit contract code size: {:?}",
            self.cfg_env.limit_contract_code_size
        );
        eprintln!(
            "  Limit contract initcode size: {:?}",
            self.cfg_env.limit_contract_initcode_size
        );

        eprintln!("\n🔨 Block Environment:");
        eprintln!("  Number: {}", self.block_env.number);
        eprintln!("  Timestamp: {}", self.block_env.timestamp);
        eprintln!("  Gas limit: {}", self.block_env.gas_limit);
        eprintln!("  Base fee: {:?}", self.block_env.basefee);
        eprintln!("  Difficulty: {}", self.block_env.difficulty);
        eprintln!("  Prevrandao: {:?}", self.block_env.prevrandao);
        eprintln!("  Beneficiary: {:?}", self.block_env.beneficiary);
        eprintln!(
            "  Blob excess gas: {:?}",
            self.block_env.blob_excess_gas_and_price
        );

        // Add withdrawals to block environment
        if let Some(withdrawals) = &self.withdrawals {
            eprintln!("  Withdrawals: {} items", withdrawals.len());
            if withdrawals.is_empty() {
                eprintln!("    (No withdrawals in this block)");
            } else {
                for (i, withdrawal) in withdrawals.iter().enumerate() {
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
            }
        } else {
            eprintln!("  Withdrawals: Not available (pre-Shanghai fork)");
        }

        if let Some(tx_env) = &self.tx_env {
            eprintln!("\n📄 Transaction Environment:");
            eprintln!("  Caller: {:?}", tx_env.caller);
            eprintln!("  Gas limit: {}", tx_env.gas_limit);
            eprintln!("  Gas price: {:?}", tx_env.gas_price);
            eprintln!("  Transaction kind: {:?}", tx_env.kind);
            eprintln!("  Value: {}", tx_env.value);
            eprintln!("  Data length: {} bytes", tx_env.data.len());
            eprintln!("  Nonce: {:?}", tx_env.nonce);
            eprintln!("  Chain ID: {:?}", tx_env.chain_id);
            eprintln!("  Access list: {} entries", tx_env.access_list.len());
            eprintln!("  Blob hashes: {} blobs", tx_env.blob_hashes.len());
            eprintln!("  Max fee per blob gas: {:?}", tx_env.max_fee_per_blob_gas);
        }

        eprintln!("\n💾 Pre-State (Initial):");
        for (address, (info, storage)) in &self.pre_state {
            eprintln!("  Account {address:?}:");
            eprintln!("    Balance: {}", info.balance);
            eprintln!("    Nonce: {}", info.nonce);
            eprintln!("    Code hash: {:?}", info.code_hash);
            eprintln!(
                "    Code size: {} bytes",
                info.code.as_ref().map_or(0, |c| c.bytecode().len())
            );
            if !storage.is_empty() {
                eprintln!("    Storage ({} slots):", storage.len());
                for (key, value) in storage.iter().take(10) {
                    eprintln!("      {key:?} => {value:?}");
                }
                if storage.len() > 10 {
                    eprintln!("      ... and {} more slots", storage.len() - 10);
                }
            }
        }

        eprintln!("\n📝 Committed State (Current):");
        for (address, (info, storage)) in &self.committed_state {
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
                for (key, value) in storage.iter().take(10) {
                    eprintln!("      {key:?} => {value:?}");
                }
                if storage.len() > 10 {
                    eprintln!("      ... and {} more slots", storage.len() - 10);
                }
            }
        }

        eprintln!("\n========================================\n");
    }
}

/// Validate post state against expected values
fn validate_post_state(
    state: &mut State<EmptyDB>,
    expected_post_state: &BTreeMap<Address, Account>,
    pre_state_debug: &HashMap<Address, (AccountInfo, HashMap<U256, U256>)>,
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
                print_state_comparison(pre_state_debug, state, expected_post_state);
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
                print_state_comparison(pre_state_debug, state, expected_post_state);
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
                        print_state_comparison(pre_state_debug, state, expected_post_state);
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
                    print_state_comparison(pre_state_debug, state, expected_post_state);
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
                    print_state_comparison(pre_state_debug, state, expected_post_state);
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
            println!("TWO ddress {address:?} storage[{slot}] = {expected_value}");
            let actual_value = state.storage(*address, *slot);
            println!("TWO actual_value {actual_value:?}");
            let actual_value = actual_value.unwrap_or_default();

            if actual_value != *expected_value {
                if print_env_on_error {
                    print_state_comparison(pre_state_debug, state, expected_post_state);
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

/// Print state comparison for debugging
fn print_state_comparison(
    pre_state: &HashMap<Address, (AccountInfo, HashMap<U256, U256>)>,
    current_state: &State<EmptyDB>,
    expected_post_state: &BTreeMap<Address, Account>,
) {
    eprintln!("\n========== STATE VALIDATION FAILURE ==========");

    eprintln!("\n💾 Pre-State (Initial):");
    for (address, (info, storage)) in pre_state {
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

    eprintln!("\n==============================================\n");
}

/// Execute a single blockchain test case
fn execute_blockchain_test(
    test_case: &BlockchainTestCase,
    print_env_on_error: bool,
) -> Result<(), TestExecutionError> {
    // Skip certain forks for now
    if test_case.network == ForkSpec::ByzantiumToConstantinopleAt5 {
        return Err(TestExecutionError::SkippedFork(format!(
            "{:?}",
            test_case.network
        )));
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
            code_hash: primitives::keccak256(&account.code),
            code: Some(bytecode::Bytecode::new_raw(account.code.clone())),
        };

        // Store for debug info
        if print_env_on_error {
            pre_state_debug.insert(address, (account_info.clone(), account.storage.clone()));
        }

        state.insert_account_with_storage(address, account_info, account.storage);
    }

    // Setup configuration based on fork
    let spec_id = fork_to_spec_id(test_case.network);
    let mut cfg = CfgEnv::default();
    cfg.spec = spec_id;

    // Genesis block is not used yet.
    let mut block_env = test_case.genesis_block_env();
    let mut parent_block_hash = Some(test_case.genesis_block_header.hash);

    // Process each block in the test
    for (block_idx, block) in test_case.blocks.iter().enumerate() {
        // Check if this block should fail
        let should_fail = block.expect_exception.is_some();

        let transactions = block.transactions.as_deref().unwrap_or_default();

        let mut block_hash = B256::ZERO;
        let mut beacon_root = None;
        // Update block environment for this block
        if let Some(header) = &block.block_header {
            block_hash = header.hash;
            beacon_root = header.parent_beacon_block_root;
            block_env = header.to_block_env();
        }

        println!("STATE BEFORE EXECUTE: {:?}", state.cache.accounts);

        // Create EVM context for each transaction to ensure fresh state access
        let evm_context = Context::mainnet()
            .with_block(&block_env)
            .with_cfg(&cfg)
            .with_db(&mut state);

        // Build and execute with EVM
        let mut evm = evm_context.build_mainnet_with_inspector(TracerEip3155::new_stdout());

        // Pre block system calls
        pre_block::pre_block_transition(
            &mut evm,
            spec_id,
            parent_block_hash,
            beacon_root,
            block.withdrawals.as_deref().unwrap_or_default(),
        );

        // Execute each transaction in the block
        for (tx_idx, tx) in transactions.iter().enumerate() {
            if tx.sender.is_none() {
                if print_env_on_error {
                    let debug_info = DebugInfo {
                        pre_state: pre_state_debug.clone(),
                        committed_state: DebugInfo::capture_committed_state(&state),
                        tx_env: None,
                        block_env: block_env.clone(),
                        cfg_env: cfg.clone(),
                        block_idx,
                        tx_idx,
                        withdrawals: block.withdrawals.clone(),
                    };
                    debug_info.print();
                }
                eprintln!("⚠️  Skipping block {block_idx} due to missing sender");
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
                            committed_state: DebugInfo::capture_committed_state(&state),
                            tx_env: None,
                            block_env: block_env.clone(),
                            cfg_env: cfg.clone(),
                            block_idx,
                            tx_idx,
                            withdrawals: block.withdrawals.clone(),
                        };
                        debug_info.print();
                    }
                    eprintln!(
                        "⚠️  Skipping block {block_idx} due to transaction env creation error: {e}"
                    );
                    break; // Skip to next block
                }
            };

            let execution_result = evm.inspect_tx(tx_env.clone());

            match execution_result {
                Ok(exec_res) => {
                    println!("\nSTATE BEFORE COMMIT: {:?}\n", evm.ctx.db().cache.accounts);
                    println!("COMMIT: {:?}\n", exec_res.state);
                    evm.commit(exec_res.state);
                    println!("STATE AFTER EXECUTE: {:?}\n", evm.ctx.db().cache.accounts);
                    if should_fail {
                        if print_env_on_error {
                            let debug_info = DebugInfo {
                                pre_state: pre_state_debug.clone(),
                                committed_state: DebugInfo::capture_committed_state(&state),
                                tx_env: Some(tx_env.clone()),
                                block_env: block_env.clone(),
                                cfg_env: cfg.clone(),
                                block_idx,
                                tx_idx,
                                withdrawals: block.withdrawals.clone(),
                            };
                            debug_info.print();
                        }
                        let exception = block.expect_exception.clone().unwrap_or_default();
                        eprintln!(
                            "⚠️  Skipping block {block_idx} due to expected failure: {exception}"
                        );
                        break; // Skip to next block
                    }
                }
                Err(e) => {
                    if !should_fail {
                        if print_env_on_error {
                            let debug_info = DebugInfo {
                                pre_state: pre_state_debug.clone(),
                                committed_state: DebugInfo::capture_committed_state(&state),
                                tx_env: Some(tx_env.clone()),
                                block_env: block_env.clone(),
                                cfg_env: cfg.clone(),
                                block_idx,
                                tx_idx,
                                withdrawals: block.withdrawals.clone(),
                            };
                            debug_info.print();
                        }
                        eprintln!(
                            "⚠️  Skipping block {block_idx} due to unexpected failure: {e:?}"
                        );
                        break; // Skip to next block
                    }
                    // Expected failure
                }
            }
        }

        // uncle rewards are not implemented yet
        post_block::post_block_transition(&mut state, &block_env, spec_id);

        parent_block_hash = Some(block_hash);

        state.merge_transitions(BundleRetention::Reverts);
    }

    // Validate post state if present
    if let Some(expected_post_state) = &test_case.post_state {
        validate_post_state(
            &mut state,
            expected_post_state,
            &pre_state_debug,
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
    let name = path.file_name().unwrap().to_str().unwrap();

    // Add any problematic tests here that should be skipped
    matches!(
        name, // Example: Skip tests that are known to be problematic
        ""
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
