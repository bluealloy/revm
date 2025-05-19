# op-revm Testdata Utilities

This directory contains utilities for testing EVM execution output against known-good testdata files.

## Usage

### Running Tests

To run tests that use the testdata comparison utility, you need to enable the `serde` feature:

```bash
# Run all tests with serde feature enabled
cargo test --features serde

# Run a specific test
cargo test --features serde -- test_basic_testdata_comparison
```

### Using the Testdata Comparison Utility

The `compare_or_save_testdata` function allows you to:

1. Save execution results to a testdata file (if it doesn't exist yet)
2. Compare execution results against an existing testdata file

Example:

```rust
use crate::test_utils::compare_or_save_testdata;

#[test]
fn my_test() {
    // Set up your EVM execution
    let result = /* execute EVM code */;
    
    // Compare or save the result
    compare_or_save_testdata("my_test.json", &result).unwrap();
}
```

### Migration from Existing Tests

If you have existing tests with assertions, you can migrate them to use the testdata comparison approach:

```rust
// Original test with assertions
assert!(result.result.is_success());
assert_eq!(result.result.gas_used(), 100);

// Run original assertions and save testdata for future comparison
assert!(result.result.is_success());
assert_eq!(result.result.gas_used(), 100);
compare_or_save_testdata("my_test.json", &result).unwrap();

// After migration, you can simplify to just:
compare_or_save_testdata("my_test.json", &result).unwrap();
```

## Benefits

Using testdata comparison provides several advantages:

1. Complete validation of all execution results, including state changes
2. Detection of unintended changes in EVM behavior between PRs
3. Easy way to update expected outputs when intentional changes are made
4. Simplifies test code by removing manual assertions 
