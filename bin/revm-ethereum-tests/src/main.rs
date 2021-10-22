mod models;
mod runner;
mod trace;

use std::{env, path::PathBuf};

use crate::trace::CustomPrintTracer;
use revm::NoOpInspector;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args:{:?}", args);
    let (folder_path, skip) = if args.len() == 1 {
        ("./temp_folder", 0)
    } else {
        let mut skip: usize = 0;
        if args.get(1) == Some(&String::from("skip")) {
            skip = args
                .get(2)
                .map(|t| t.clone())
                .unwrap_or_default()
                .parse()
                .unwrap();
        }
        ("./tests/GeneralStateTests", skip)
    };
    let test_files = runner::find_all_json_tests(PathBuf::from(folder_path));
    println!("Start running tests skip+{:?}", skip);
    let test_files = test_files.as_slice()[skip..].to_vec();
    if args.len() == 1 {
        runner::run(test_files, CustomPrintTracer {})
    } else {
        runner::run(test_files, NoOpInspector())
    }
}
