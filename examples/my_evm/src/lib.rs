#![doc = include_str!("../README.md")]

/// Public API for the custom EVM implementation.
/// This module provides the interface for external users to interact with MyEvm.
pub mod api;

/// Custom EVM implementation module.
/// This module contains MyEvm, which is a custom variant of the REVM that demonstrates
/// how to create your own EVM implementation by wrapping the standard EVM components.
pub mod evm;

/// Custom handler implementation for MyEvm.
/// This module contains MyHandler, which defines custom execution behavior for the EVM,
/// including how transactions are processed and how the EVM interacts with inspectors.
pub mod handler;

/// Custom frame implementation for MyEvm.
/// This module contains MyFrame, which is a custom variant of the EthFrame that demonstrates
/// how to create your own frame implementation by wrapping the standard EthFrame components.
pub mod frame;

pub use evm::*;
pub use handler::*;
