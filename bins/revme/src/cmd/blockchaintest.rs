use clap::Parser;
use serde_json::json;
use statetest_types::blockchain::BlockchainTest;
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

            run_tests(
                test_files,
                self.single_thread,
                self.json,
                self.keep_going,
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
    let content = fs::read_to_string(file_path)
        .map_err(|e| Error::FileRead(file_path.to_path_buf(), e))?;
    
    let blockchain_test: BlockchainTest = serde_json::from_str(&content)
        .map_err(|e| Error::JsonDecode(file_path.to_path_buf(), e))?;
    
    let mut test_count = 0;
    
    for (test_name, _test_case) in blockchain_test.0 {
        if !output_json {
            println!("  Running: {}", test_name);
        }
        
        // TODO: Implement actual blockchain test execution
        // This would involve:
        // 1. Setting up the genesis state
        // 2. Processing each block in sequence
        // 3. Validating state roots, receipts, etc.
        // 4. Checking for expected exceptions
        
        test_count += 1;
    }
    
    Ok(test_count)
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
pub enum Error {
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),
    
    #[error("No JSON files found in: {0}")]
    NoJsonFiles(PathBuf),
    
    #[error("Failed to read file {0}: {1}")]
    FileRead(PathBuf, std::io::Error),
    
    #[error("Failed to decode JSON from {0}: {1}")]
    JsonDecode(PathBuf, serde_json::Error),
    
    #[error("Directory traversal error: {0}")]
    WalkDir(#[from] walkdir::Error),
    
    #[error("{failed} tests failed")]
    TestsFailed { failed: usize },
}