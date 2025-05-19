use std::{fs, path::PathBuf};
use anyhow::Result;
use revm::context_interface::result::ResultAndState;
use serde::Serialize;
use serde::de::DeserializeOwned;

// Constant for testdata directory path
const TESTS_TESTDATA: &str = "tests/testdata";

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
pub(crate) fn compare_or_save_testdata<HaltReasonTy>(
    filename: &str,
    output: &ResultAndState<HaltReasonTy>,
) -> Result<()>
where
    HaltReasonTy: Serialize + DeserializeOwned + PartialEq,
{
    let tests_dir = PathBuf::from(TESTS_TESTDATA);
    let testdata_file = tests_dir.join(filename);
    
    // Create directory if it doesn't exist
    if !tests_dir.exists() {
        fs::create_dir_all(&tests_dir)?;
    }
    
    // Serialize the output to JSON for saving
    let output_json = serde_json::to_string_pretty(output)?;
    
    // If the testdata file doesn't exist, save the output
    if !testdata_file.exists() {
        fs::write(&testdata_file, &output_json)?;
        println!("Saved testdata to {}", testdata_file.display());
        return Ok(());
    }
    
    // Read the expected output from the testdata file
    let expected_json = fs::read_to_string(&testdata_file)?;
    
    // Deserialize to actual ResultAndState object for proper comparison
    let expected: ResultAndState<HaltReasonTy> = serde_json::from_str(&expected_json)?;
    
    // Compare the output objects directly
    if output != &expected {
        // If they don't match, generate a nicer error by pretty-printing both as JSON
        // This helps with debugging by showing the exact differences
        let expected_pretty = serde_json::to_string_pretty(&expected)?;
        
        return Err(anyhow::anyhow!(
            "Output does not match expected output.\nExpected:\n{}\n\nActual:\n{}",
            expected_pretty, output_json
        ));
    }
    
    Ok(())
}

/// Regenerates all testdata files.
///
/// This is useful when making intentional changes to the EVM that affect output.
///
/// # Returns
///
/// `Ok(())` if the regeneration was successful
/// `Err(anyhow::Error)` if there was an error
#[allow(dead_code)]
pub(crate) fn regenerate_testdata() -> Result<()> {
    // Implementation would go here
    // This would walk through the testdata directory and re-run each test
    todo!("Implement testdata regeneration")
} 
