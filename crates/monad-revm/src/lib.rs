//! Monad-specific EVM implementation.
//!
//! This crate provides Monad-specific customizations for REVM:
//! - Gas limit charging (no refunds)
//! - Custom precompiles
//! - Custom gas costs
//! - Custom code size limits (64KB max code, 128KB max initcode)

pub mod api;
pub mod cfg;
pub mod evm;
pub mod handler;
pub mod instructions;
pub mod precompiles;
pub mod spec;

pub use api::*;
pub use cfg::{MonadCfgEnv, MONAD_MAX_CODE_SIZE, MONAD_MAX_INITCODE_SIZE};
pub use evm::MonadEvm;
pub use spec::*;
