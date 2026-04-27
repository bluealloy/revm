pub mod merkle_trie;
mod runner;
pub mod utils;

pub use runner::{TestError as Error, TestErrorKind};

use crate::dir_utils::find_all_json_tests;
use clap::Parser;
use regex::Regex;
use runner::{run, TestError};
use std::path::PathBuf;

/// `statetest` subcommand
#[derive(Parser, Debug)]
pub struct Cmd {
    /// Path to folder or file containing the tests
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
    ///
    /// It will stop second run of evm on failure.
    #[arg(long)]
    json: bool,
    /// Output outcome in JSON format
    ///
    /// If `--json` is true, this is implied.
    ///
    /// It will stop second run of EVM on failure.
    #[arg(short = 'o', long)]
    json_outcome: bool,
    /// Output results as a JSON array with standard schema to stdout
    ///
    /// Fields: name, pass, fork, stateRoot, error
    #[arg(long)]
    json_array: bool,
    /// Omit progress output
    #[arg(long)]
    omit_progress: bool,
    /// Keep going after a test failure
    #[arg(long, alias = "no-fail-fast")]
    keep_going: bool,
    /// Only run tests whose name matches this regex
    #[arg(long)]
    run: Option<String>,
}

impl Cmd {
    /// Runs `statetest` command.
    pub fn run(&self) -> Result<(), TestError> {
        let run_filter = self
            .run
            .as_deref()
            .map(Regex::new)
            .transpose()
            .map_err(|e| TestError {
                name: "Regex compilation".to_string(),
                path: String::new(),
                kind: TestErrorKind::RegexError(e.to_string()),
            })?;

        for path in &self.paths {
            if !path.exists() {
                return Err(TestError {
                    name: "Path validation".to_string(),
                    path: path.display().to_string(),
                    kind: TestErrorKind::InvalidPath,
                });
            }

            if !self.json_array {
                println!("\nRunning tests in {}...", path.display());
            }
            let test_files = find_all_json_tests(path);

            if test_files.is_empty() {
                return Err(TestError {
                    name: "Path validation".to_string(),
                    path: path.display().to_string(),
                    kind: TestErrorKind::NoJsonFiles,
                });
            }

            run(
                test_files,
                self.single_thread,
                self.json,
                self.json_outcome,
                self.json_array,
                self.keep_going,
                self.omit_progress,
                run_filter.clone(),
            )?
        }
        Ok(())
    }
}
