mod merkle_trie;
mod models;
mod runner;
mod trace;

use std::{env, path::PathBuf};

use crate::trace::CustomPrintTracer;
use revm::NoOpInspector;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args:{:?}", args);
    let folder_path = if args.len() == 1 {
        "./bins/revm-ethereum-tests/temp_folder"
    } else {
        let second = &args[1];
        if second == "eth" {
            "./bins/revm-ethereum-tests/tests/GeneralStateTests"
        } else {
            second
        }
    };
    let test_files = runner::find_all_json_tests(PathBuf::from(folder_path));
    println!("Start running tests on: {:?}", folder_path);
    if args.len() == 1 {
        runner::run(test_files, NoOpInspector {})
    } else {
        runner::run(test_files, NoOpInspector())
    }
}
