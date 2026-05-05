//! Integration tests for REVM crates.
//!
//! This crate hosts integration-style tests that exercise multiple REVM
//! crates together. Snapshot assertions use [`insta`].

#[cfg(test)]
mod revm_tests;

#[cfg(test)]
mod eip8037;
