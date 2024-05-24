use revm_interpreter::analysis::{validate_raw_eof, EofError};
use revm_primitives::{Bytes, Eof};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Instant,
};
use walkdir::{DirEntry, WalkDir};

/*
Types of error: {
    FalsePossitive: 1,
    Error(
        Validation(
            OpcodeDisabled,
        ),
    ): 19,
}
*/
#[test]
fn eof_run_all_tests() {
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests");
    run_test(&eof_tests)
}

/*
Types of error: {
    FalsePossitive: 1,
}
Passed tests: 1262/1263
EOF_EofCreateWithTruncatedContainer TODO
*/
#[test]
fn eof_validation() {
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests/eof_validation");
    run_test(&eof_tests)
}

/*
Types of error: {
    OpcodeDisabled: 8,
}
Passed tests: 194/202
Probably same as below.
*/
#[test]
fn eof_validation_eip5450() {
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests/EIP5450");
    run_test(&eof_tests)
}

/*
Types of error: {
    OpcodeDisabled: 9,
}
Passed tests: 290/299
 */
// 0x60018080808080fa00 STATICCALL validInvalid - validInvalid_89
// 0x60018080808080f400 DELEGATECALL validInvalid - validInvalid_88
// 0x60018080808080f400 CALL validInvalid - validInvalid_86
// 0x38e4 CODESIZE  validInvalid - validInvalid_4
// 0x60013f00 EXTCODEHASH validInvalid - validInvalid_39
// 0x60018080803c00 EXTCODECOPY validInvalid - validInvalid_37
// 0x60013b00 EXTCODESIZE validInvalid - validInvalid_36
// 0x600180803900 CODECOPY validInvalid - validInvalid_35
// 0x5a00 GAS validInvalid - validInvalid_60
// 0xfe opcode is considered valid, should it be disabled?
#[test]
fn eof_validation_eip3670() {
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests/EIP3670");
    run_test(&eof_tests)
}

/// PASSING ALL
#[test]
fn eof_validation_eip4750() {
    let inst = Instant::now();
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests/EIP4750");
    run_test(&eof_tests);
    println!("Elapsed:{:?}", inst.elapsed())
}

/// PASSING ALL
#[test]
fn eof_validation_eip3540() {
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests/EIP3540");
    run_test(&eof_tests)
}

/// PASSING ALL
#[test]
fn eof_validation_eip4200() {
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests/EIP4200");
    run_test(&eof_tests);
}

pub fn run_test(path: &Path) {
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
                test_sum += 1;
                let res = validate_raw_eof(test_vector.code.clone());
                if res.is_ok() != test_vector.results.prague.result {
                    let eof = Eof::decode(test_vector.code.clone());
                    println!(
                        "\nTest failed: {} - {}\nresult:{:?}\nrevm err_result:{:#?}\nbytes:{:?}\n,eof:{eof:#?}",
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
                    //println!("Test passed: {} - {}", name, vector_name);
                    passed_tests += 1;
                }
            }
        }
    }
    println!("Types of error: {:#?}", types_of_error);
    println!("Passed tests: {}/{}", passed_tests, test_sum);
}

pub fn find_all_json_tests(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .map(DirEntry::into_path)
        .collect::<Vec<PathBuf>>()
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct TestSuite(pub BTreeMap<String, TestUnit>);

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestUnit {
    /// Test info is optional
    #[serde(default, rename = "_info")]
    pub info: Option<serde_json::Value>,

    pub vectors: BTreeMap<String, TestVector>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestVector {
    code: Bytes,
    results: PragueResult,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PragueResult {
    #[serde(rename = "Prague")]
    prague: Result,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Result {
    result: bool,
    exception: Option<String>,
}
