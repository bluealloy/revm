mod runner;
mod models;

use std::{path::PathBuf};

use serde::de::Error;

pub fn main() {
    let mut test_files = runner::find_all_json_tests(PathBuf::from("./tests/GeneralStateTests"));

    test_files.truncate(300); //for test only
    println!("Start running tests.");
    runner::run(test_files)
}
