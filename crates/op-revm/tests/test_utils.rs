use std::{fs, path::PathBuf};
use anyhow::Result;
use revm::context_interface::result::ResultAndState;
use serde::Serialize;
use serde_json::Value;

// Constant for testdata directory path
const TESTDATA_DIR: &str = "tests/testdata";

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
pub(crate) fn compare_or_save_testdata<HaltReasonTy>(
    filename: &str,
    output: &ResultAndState<HaltReasonTy>,
) -> Result<()>
where
    HaltReasonTy: Serialize,
{
    let tests_dir = PathBuf::from(TESTDATA_DIR);
    let testdata_file = tests_dir.join(filename);
    
    // Create directory if it doesn't exist
    if !tests_dir.exists() {
        fs::create_dir_all(&tests_dir)?;
    }
    
    // Convert output to Value
    let output_value = serde_json::to_value(output)?;
    let output_json = serde_json::to_string_pretty(&output_value)?;
    
    // If the testdata file doesn't exist, save the output
    if !testdata_file.exists() {
        fs::write(&testdata_file, output_json)?;
        println!("Saved testdata to {}", testdata_file.display());
        return Ok(());
    }
    
    // Read the expected output from the testdata file
    let expected_json = fs::read_to_string(&testdata_file)?;
    let expected_value: Value = serde_json::from_str(&expected_json)?;
    
    // Compare the output with the expected output
    // This compares the JSON values, which is more flexible than comparing objects directly
    if output_value != expected_value {
        return Err(anyhow::anyhow!(
            "Output does not match expected output.\nExpected:\n{}\n\nActual:\n{}",
            serde_json::to_string_pretty(&expected_value)?, output_json
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
