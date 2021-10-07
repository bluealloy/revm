mod runner;
mod models;

use std::{path::PathBuf};

use serde::de::Error;

pub fn main() {
    let folder_path = "./tests/GeneralStateTests";
    //let folder_path = "./temp_folder";
    let mut test_files = runner::find_all_json_tests(PathBuf::from(folder_path));

    //test_files.truncate(300); //for test only
    println!("Start running tests.");
    runner::run(test_files)
}
