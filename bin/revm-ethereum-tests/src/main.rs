mod runner;
mod models;
mod trace;

use std::{path::PathBuf};

use serde::de::Error;

use crate::trace::CustomPrintTracer;
use revm::NoOpInspector;

pub fn main() {
    //let folder_path = "./tests/GeneralStateTests";
    let folder_path = "./temp_folder";
    let mut test_files = runner::find_all_json_tests(PathBuf::from(folder_path));

    //test_files.truncate(300); //for test only
    println!("Start running tests.");
    let inspector = Box::new(CustomPrintTracer{});
    //let inspector = Box::new(NoOpInspector());
    runner::run(test_files,inspector)
}


// big nonce test. Not applicable.
// big gas limit test. Not applicable.
//      "ttNonce"
//      "TransactionWithHighNonce256.json"
        