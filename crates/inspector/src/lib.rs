//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

// #[cfg(feature = "std")]
// pub mod customprinter;
// #[cfg(all(feature = "std", feature = "serde-json"))]
// mod eip3155;
//mod gas;
mod handler_register;
mod inspector;
// mod noop;

pub use inspector::*;

// /// [Inspector] implementations.
// pub mod inspectors {
//     #[cfg(feature = "std")]
//     pub use super::customprinter::CustomPrintTracer;
//     #[cfg(all(feature = "std", feature = "serde-json"))]
//     pub use super::eip3155::TracerEip3155;
//     pub use super::gas::GasInspector;
//     pub use super::noop::NoOpInspector;
// }
