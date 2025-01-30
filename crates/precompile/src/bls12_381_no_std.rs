//! `no_std` BLS12-381 Precompiles

use crate::PrecompileWithAddress;

pub mod utils;
pub mod g1_add;
pub mod pairing;

/// Returns the `no_std` BLS12-381 precompiles with their addresses.
pub fn precompiles() -> impl Iterator<Item = PrecompileWithAddress> {
    [pairing::PRECOMPILE].into_iter()
}
