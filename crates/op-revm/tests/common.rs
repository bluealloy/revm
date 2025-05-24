#![allow(dead_code)]

use revm::{
    context::result::ResultAndState,
    context_interface::result::{ExecutionResult, HaltReason, Output, SuccessReason},
    primitives::Bytes,
    state::EvmState,
};

// Constant for testdata directory path
pub(crate) const TESTS_TESTDATA: &str = "tests/testdata";

#[cfg(not(feature = "serde"))]
pub(crate) fn compare_or_save_testdata<HaltReasonTy>(
    _filename: &str,
    _output: &ResultAndState<ExecutionResult<HaltReasonTy>, EvmState>,
) {
    // serde needs to be enabled to use this function
}

/// Compares or saves the execution output to a testdata file.
///
/// This utility helps maintain consistent test behavior by comparing
/// execution results against known-good outputs stored in JSON files.
///
/// # Arguments
///
/// * `filename` - The name of the testdata file, relative to tests/testdata/
/// * `output` - The execution output to compare or save
///
/// # Returns
///
/// `Ok(())` if the comparison or save was successful
/// `Err(anyhow::Error)` if there was an error
///
/// # Note
///
/// Tests using this function require the `serde` feature to be enabled:
/// ```bash
/// cargo test --features serde
/// ```
#[cfg(feature = "serde")]
pub(crate) fn compare_or_save_testdata<HaltReasonTy>(
    filename: &str,
    output: &ResultAndState<ExecutionResult<HaltReasonTy>, EvmState>,
) where
    HaltReasonTy: serde::Serialize + for<'a> serde::Deserialize<'a> + PartialEq,
{
    use std::{fs, path::PathBuf};

    let tests_dir = PathBuf::from(TESTS_TESTDATA);
    let testdata_file = tests_dir.join(filename);

    // Create directory if it doesn't exist
    if !tests_dir.exists() {
        fs::create_dir_all(&tests_dir).unwrap();
    }

    // Serialize the output to JSON for saving
    let output_json = serde_json::to_string_pretty(output).unwrap();

    // If the testdata file doesn't exist, save the output
    if !testdata_file.exists() {
        fs::write(&testdata_file, &output_json).unwrap();
        println!("Saved testdata to {}", testdata_file.display());
        return;
    }

    // Read the expected output from the testdata file
    let expected_json = fs::read_to_string(&testdata_file).unwrap();

    // Deserialize to actual ResultAndState object for proper comparison
    let expected = serde_json::from_str(&expected_json).unwrap();

    // Compare the output objects directly
    if output != &expected {
        // If they don't match, generate a nicer error by pretty-printing both as JSON
        // This helps with debugging by showing the exact differences
        let expected_pretty = serde_json::to_string_pretty(&expected).unwrap();

        panic!(
            "Value does not match testdata.\nExpected:\n{}\n\nActual:\n{}",
            expected_pretty, output_json
        );
    }
}

/// Example showing how to migrate an existing test to use the testdata comparison.
///
/// This example consists of:
/// 1. The "original" test with standard assertions
/// 2. The migration approach - running assertions and saving testdata
/// 3. The final migrated test that only uses testdata comparison
#[test]
fn template_test() {
    // Create a minimal result and state
    let result = ResultAndState::new(
        ExecutionResult::Success {
            reason: SuccessReason::Stop,
            gas_used: 1000,
            gas_refunded: 0,
            logs: vec![],
            output: Output::Call(Bytes::from(vec![4, 5, 6])),
        },
        EvmState::default(),
    );

    // Simply use the testdata comparison utility
    // No assertions needed - full validation is done by comparing with testdata
    compare_or_save_testdata::<HaltReason>("template_test.json", &result);
}
