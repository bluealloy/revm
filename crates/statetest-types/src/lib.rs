//! # revm-statetest-types
//!
//! This crate provides type definitions and utilities for Ethereum state tests,
//! specifically tailored for use with REVM.
//!
//! It includes structures for representing account information, environment settings,
//! test cases, and transaction data used in Ethereum state tests.

mod account_info;
mod deserializer;
mod env;
mod spec;
mod test;
mod test_authorization;
mod test_suite;
mod test_unit;
mod transaction;

pub use account_info::*;
pub use deserializer::*;
pub use env::*;
pub use spec::*;
pub use test::*;
pub use test_authorization::*;
pub use test_suite::*;
pub use test_unit::*;
pub use transaction::*;
