use std::fs;
use std::path::PathBuf;
use serde_json::{json, Value};
use revm::context_interface::result::ResultAndState;
use std::io;

/// Compare the EVM execution output with expected output from a testdata file.
/// If the testdata file doesn't exist, it will be created with the current output.
/// On mismatch, it will print both expected and actual outputs for easier debugging.
///
/// # Arguments
///
/// * `testdata_path` - Path to the testdata file relative to the tests directory
/// * `output` - The output from EVM execution
///
/// # Returns
///
/// `Ok(())` if the comparison succeeded or the testdata file was created
/// `Err(io::Error)` if there was an error reading or writing the testdata file
/// `Err(String)` if there was a mismatch between the expected and actual outputs
pub(crate) fn compare_or_save_testdata<HaltReasonTy>(
    testdata_path: &str,
    output: &ResultAndState<HaltReasonTy>,
) -> Result<(), Box<dyn std::error::Error>> 
where
    HaltReasonTy: serde::Serialize,
{
    let tests_dir = PathBuf::from("tests/testdata");
    let testdata_file = tests_dir.join(testdata_path);
    
    // Create directory if it doesn't exist
    if let Some(parent) = testdata_file.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    
    // Serialize the output to JSON
    let output_json = json!({
        "result": output.result,
        "state": output.state
    });
    
    // Check if testdata file exists
    if !testdata_file.exists() {
        // Create the testdata file with the current output
        fs::write(&testdata_file, serde_json::to_string_pretty(&output_json)?)?;
        println!("Created testdata file: {}", testdata_file.display());
        return Ok(());
    }
    
    // Read the expected output from the testdata file
    let expected_json: Value = serde_json::from_str(&fs::read_to_string(&testdata_file)?)?;
    
    // Compare the expected and actual outputs
    if expected_json != output_json {
        println!("Expected output:\n{}", serde_json::to_string_pretty(&expected_json)?);
        println!("Actual output:\n{}", serde_json::to_string_pretty(&output_json)?);
        return Err(format!("Testdata mismatch for file: {}", testdata_file.display()).into());
    }
    
    Ok(())
}

/// Regenerate the testdata file with the current output.
///
/// # Arguments
///
/// * `testdata_path` - Path to the testdata file relative to the tests directory
/// * `output` - The output from EVM execution
///
/// # Returns
///
/// `Ok(())` if the testdata file was successfully regenerated
/// `Err(io::Error)` if there was an error writing the testdata file
pub(crate) fn regenerate_testdata<HaltReasonTy>(
    testdata_path: &str,
    output: &ResultAndState<HaltReasonTy>,
) -> Result<(), io::Error> 
where
    HaltReasonTy: serde::Serialize,
{
    let tests_dir = PathBuf::from("tests/testdata");
    let testdata_file = tests_dir.join(testdata_path);
    
    // Create directory if it doesn't exist
    if let Some(parent) = testdata_file.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    
    // Serialize the output to JSON
    let output_json = json!({
        "result": output.result,
        "state": output.state
    });
    
    // Write the testdata file with the current output
    fs::write(&testdata_file, serde_json::to_string_pretty(&output_json)?)?;
    println!("Regenerated testdata file: {}", testdata_file.display());
    
    Ok(())
}

/// Convert an existing test to use the testdata comparison utility.
/// This is a helper function to assist in migrating tests to the new approach.
///
/// # Arguments
///
/// * `test_name` - Name of the test, used to generate a unique testdata file path
/// * `output` - The output from EVM execution
/// * `assertions` - A closure that contains the existing test assertions
///
/// # Returns
///
/// `Ok(())` if the migration was successful
/// `Err(Box<dyn std::error::Error>)` if there was an error 
pub(crate) fn migrate_test_to_testdata<HaltReasonTy, F>(
    test_name: &str,
    output: &ResultAndState<HaltReasonTy>,
    assertions: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(&ResultAndState<HaltReasonTy>),
    HaltReasonTy: serde::Serialize,
{
    // Run the original assertions to ensure test integrity
    assertions(output);
    
    // Generate a testdata file path based on the test name
    let testdata_path = format!("migrated/{}.json", test_name);
    
    // Compare or save the testdata
    compare_or_save_testdata(&testdata_path, output)?;
    
    println!(
        "Test '{}' successfully migrated to use testdata comparison. \
        You can now replace the assertions with: \
        compare_or_save_testdata(\"{}\", &output).unwrap();", 
        test_name, testdata_path
    );
    
    Ok(())
} 
