pub mod merkle_trie;
mod runner;
pub mod utils;

pub use runner::TestError as Error;

use clap::Parser;
use runner::{find_all_json_tests, run, TestError};
use std::path::PathBuf;

/// `statetest` subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    /// Path to folder or file containing the tests. If multiple paths are specified
    /// they will be run in sequence.
    ///
    /// Folders will be searched recursively for files with the extension `.json`.
    #[clap(required = true, num_args = 1..)]
    paths: Vec<PathBuf>,
    /// Run tests in a single thread.
    #[clap(short = 's', long)]
    single_thread: bool,
    /// Output results in JSON format.
    /// It will stop second run of evm on failure.
    #[clap(long)]
    json: bool,
    /// Output outcome in JSON format. If `--json` is true, this is implied.
    /// It will stop second run of EVM on failure.
    #[clap(short = 'o', long)]
    json_outcome: bool,
    /// Keep going after a test failure.
    #[clap(long, alias = "no-fail-fast")]
    keep_going: bool,
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) -> Result<(), TestError> {
        for path in &self.paths {
            println!("\nRunning tests in {}...", path.display());
            let test_files = find_all_json_tests(path);
            run(
                test_files,
                self.single_thread,
                self.json,
                self.json_outcome,
                self.keep_going,
            )?
        }
        Ok(())
    }
}
