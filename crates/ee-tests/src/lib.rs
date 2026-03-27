//! Common test utilities for REVM crates.
//!
//! This crate provides shared test utilities that are used across different REVM crates.
//! Snapshot testing is powered by the `insta` crate.

#[cfg(test)]
mod tests {
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestData {
        value: u32,
        message: String,
    }

    #[test]
    fn test_snapshot() {
        let test_data = TestData {
            value: 42,
            message: "test message".to_string(),
        };

        insta::assert_json_snapshot!(test_data);
    }
}

#[cfg(test)]
mod op_revm_tests;

#[cfg(test)]
mod revm_tests;
