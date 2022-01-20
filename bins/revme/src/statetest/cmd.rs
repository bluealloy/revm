use std::path::PathBuf;

use super::runner::{find_all_json_tests, run, TestError};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Cmd {
    #[structopt(required = true)]
    path: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> Result<(), TestError> {
        let test_files = find_all_json_tests(&self.path);
        println!("Start running tests on: {:?}", self.path);
        run(test_files)
    }
}
