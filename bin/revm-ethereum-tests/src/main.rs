mod runner;
mod models;

use std::{path::PathBuf};

use serde::de::Error;
use tokio;

#[tokio::main]
pub async fn main() {
    let mut test_files = runner::find_all_json_tests(PathBuf::from("./tests/GeneralStateTests")).await;

    println!("found {}:\n{:?}\n{:?}",test_files.len(),test_files.get(0),test_files.get(1));
    //test_files.truncate(1); //for test only
    println!("Start running tests.");
    runner::run(test_files).await
}
