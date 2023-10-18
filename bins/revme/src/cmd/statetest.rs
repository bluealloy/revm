pub mod merkle_trie;
pub mod models;
mod runner;

pub use runner::TestError as Error;

use runner::{find_all_json_tests, run, TestError};
use std::path::PathBuf;
use structopt::StructOpt;

/// Statetest command
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// Path to folder or file containing the tests. If multiple paths are specified
    /// they will be run in sequence.
    ///
    /// Folders will be searched recursively for files with the extension `.json`.
    #[structopt(required = true)]
    path: Vec<PathBuf>,
    /// Run tests in a single thread.
    #[structopt(short = "s", long)]
    single_thread: bool,
    /// Output results in JSON format.
    #[structopt(long)]
    json: bool,
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) -> Result<(), TestError> {
        for path in &self.path {
            println!("\nRunning tests in {}...", path.display());
            let test_files = find_all_json_tests(path);
            run(test_files, self.single_thread, self.json)?
        }
        Ok(())
    }
}
