//! Integration tests for REVM crates.
//!
//! This crate hosts integration-style tests that exercise multiple REVM
//! crates together. Snapshot assertions use the `insta` crate.

/// Asserts a JSON snapshot with map keys sorted, so the snapshot is stable
/// across `HashMap` hashers (e.g. with the `map-foldhash` feature enabled).
#[cfg(test)]
#[macro_export]
macro_rules! assert_sorted_json_snapshot {
    ($value:expr $(,)?) => {
        ::insta::with_settings!({sort_maps => true}, {
            ::insta::assert_json_snapshot!($value);
        })
    };
}

#[cfg(test)]
mod revm_tests;

#[cfg(test)]
mod eip8037;
