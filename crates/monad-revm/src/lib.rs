//! Monad-specific EVM implementation.
//!
//! This crate provides Monad-specific customizations for REVM:
//! - Gas limit charging (no refunds)
//! - Custom precompiles
//! - Custom gas costs

pub mod handler;
pub mod instructions;
pub mod precompiles;
pub mod spec;

pub use spec::*;
