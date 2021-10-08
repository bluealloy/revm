mod models;
mod runner;
mod trace;

use std::{env, path::PathBuf};

use serde::de::Error;

use crate::trace::CustomPrintTracer;
use revm::NoOpInspector;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args:{:?}",args);
    let folder_path = if args.len() == 1 {
        "./temp_folder"
    } else {
        "./tests/GeneralStateTests"
    };
    //let folder_path = "./tests/GeneralStateTests";
    //let folder_path = "./temp_folder";
    let mut test_files = runner::find_all_json_tests(PathBuf::from(folder_path));

    //test_files.truncate(300); //for test only
    println!("Start running tests.");
    //let inspector = Box::new(CustomPrintTracer{});
    //let inspector = Box::new(NoOpInspector());
    if args.len() == 1 {
        runner::run(test_files, Box::new(CustomPrintTracer {}))
    } else {
        runner::run(test_files, Box::new(NoOpInspector()))
    }
}

// big nonce test. Not applicable.
// big gas limit test. Not applicable.
//      "ttNonce"
//      "TransactionWithHighNonce256.json"
