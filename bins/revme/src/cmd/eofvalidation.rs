mod test_suite;

pub use test_suite::{PragueTestResult, TestResult, TestSuite, TestUnit, TestVector};

use crate::{cmd::Error, dir_utils::find_all_json_tests};
use clap::Parser;
use revm::bytecode::eof::{validate_raw_eof_inner, CodeType, EofError};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `eof-validation` subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    /// Input paths to EOF validation tests.
    #[arg(required = true, num_args = 1..)]
    paths: Vec<PathBuf>,
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) -> Result<(), Error> {
        // check if path exists.
        for path in &self.paths {
            if !path.exists() {
                return Err(Error::Custom("The specified path does not exist"));
            }
            run_test(path)?
        }
        Ok(())
    }
}

fn skip_test(name: &str) -> bool {
    // embedded containers rules changed
    if name.starts_with("EOF1_embedded_container") {
        return true;
    }
    matches!(
        name,
        "EOF1_undefined_opcodes_186"
        | ""
        // truncated data is only allowed in embedded containers
        | "validInvalid_48"
        | "validInvalid_1"
        | "EOF1_truncated_section_3"
        | "EOF1_truncated_section_4"
        | "validInvalid_2"
        | "validInvalid_3"
        // Orphan containers are no longer allowed
        | "EOF1_returncontract_valid_0"
        | "EOF1_returncontract_valid_1"
        | "EOF1_returncontract_valid_2"
        | "EOF1_eofcreate_valid_1"
        | "EOF1_eofcreate_valid_2"
        | "EOF1_section_order_6"
    )
}

pub fn run_test(path: &Path) -> Result<(), Error> {
    let test_files = find_all_json_tests(path);
    let mut test_sum = 0;
    let mut passed_tests = 0;

    #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
    enum ErrorType {
        FalsePositive,
        Error(EofError),
    }
    let mut types_of_error: BTreeMap<ErrorType, usize> = BTreeMap::new();
    for test_file in test_files {
        let s = std::fs::read_to_string(test_file).unwrap();
        let suite: TestSuite = serde_json::from_str(&s).unwrap();
        for (name, test_unit) in suite.0 {
            for (vector_name, test_vector) in test_unit.vectors {
                if skip_test(&vector_name) {
                    continue;
                }
                test_sum += 1;
                let kind = if test_vector.container_kind.is_some() {
                    Some(CodeType::ReturnContract)
                } else {
                    Some(CodeType::ReturnOrStop)
                };
                let res = validate_raw_eof_inner(test_vector.code.clone(), kind);
                if res.is_ok() != test_vector.results.prague.result {
                    println!(
                        "\nTest failed: {} - {}\nresult:{:?}\nrevm err_result:{:#?}\nbytes:{:?}\n",
                        name,
                        vector_name,
                        test_vector.results.prague,
                        res.as_ref().err(),
                        test_vector.code
                    );
                    *types_of_error
                        .entry(
                            res.err()
                                .map(ErrorType::Error)
                                .unwrap_or(ErrorType::FalsePositive),
                        )
                        .or_default() += 1;
                } else {
                    passed_tests += 1;
                }
            }
        }
    }
    println!("Passed tests: {}/{}", passed_tests, test_sum);
    if passed_tests != test_sum {
        println!("Types of error: {:#?}", types_of_error);
        Err(Error::EofValidation {
            failed_test: test_sum - passed_tests,
            total_tests: test_sum,
        })
    } else {
        Ok(())
    }
}
