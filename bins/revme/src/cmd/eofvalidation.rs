mod test_suite;

pub use test_suite::{TestResult, TestSuite, TestUnit, TestVector};

use crate::{cmd::Error, dir_utils::find_all_json_tests};
use clap::Parser;
use revm::bytecode::eof::{validate_raw_eof_inner, CodeType, EofError};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `eof-validation` subcommand
#[derive(Parser, Debug)]
pub struct Cmd {
    /// Input paths to EOF validation tests
    #[arg(required = true, num_args = 1..)]
    paths: Vec<PathBuf>,
}

impl Cmd {
    /// Runs statetest command.
    pub fn run(&self) -> Result<(), Error> {
        // Check if path exists.
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
    // Embedded containers rules changed
    if name.starts_with("EOF1_embedded_container") {
        return true;
    }
    matches!(
        name,
        "EOF1_undefined_opcodes_186"
        | ""
        // Truncated data is only allowed in embedded containers
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
        let s = std::fs::read_to_string(&test_file).unwrap();
        let suite: TestSuite = serde_json::from_str(&s).unwrap();
        for (name, test_unit) in suite.0 {
            for (vector_name, test_vector) in test_unit.vectors {
                if skip_test(&vector_name) {
                    continue;
                }
                test_sum += 1;
                let kind = match test_vector.container_kind.as_deref() {
                    Some("RUNTIME") => CodeType::Runtime,
                    Some("INITCODE") => CodeType::Initcode,
                    None => CodeType::Runtime,
                    _ => return Err(Error::Custom("Invalid container kind")),
                };
                // In future this can be generalized to cover multiple forks, Not just Osaka.
                let Some(test_result) = test_vector.results.get("Osaka") else {
                    // if test does not have a result that we can compare to, we skip it
                    println!("Test without result: {} - {}", name, vector_name);
                    continue;
                };
                let res = validate_raw_eof_inner(test_vector.code.clone(), Some(kind));
                if test_result.result != res.is_ok() {
                    println!(
                        "\nTest failed: {} - {}\nPath:{:?}\nresult:{:?}\nrevm err_result:{:#?}\nExpected exception:{:?}\nbytes:{:?}\n",
                        name,
                        vector_name,
                        test_file,
                        test_result.result,
                        res.as_ref().err(),
                        test_result.exception,
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
