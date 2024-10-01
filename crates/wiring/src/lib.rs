//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod block;
pub mod default;
pub mod evm_wiring;
pub mod precompile;
pub mod result;

pub use block::Block;
pub use evm_wiring::{DefaultEthereumWiring, EthereumWiring, EvmWiring, HaltReasonTrait};
pub use transaction::{Transaction, TransactionType};

// KZG

#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub mod kzg;

#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub use kzg::{EnvKzgSettings, KzgSettings};

// silence kzg-rs lint as c-kzg will be used as default if both are enabled.

#[cfg(all(feature = "c-kzg", feature = "kzg-rs"))]
use kzg_rs as _;
#[cfg(all(feature = "c-kzg", feature = "kzg-rs"))]
use once_cell as _;
