//! EVM gasometer.

//#![deny(warnings)]
//#![forbid(unsafe_code, unused_variables)]
//#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
mod calc;
mod constants;
mod utils;

pub use calc::*;
pub use constants::*;
