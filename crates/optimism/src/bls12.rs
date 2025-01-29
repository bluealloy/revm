//! BLS12-381 precompile with input size limits for Optimism.

use precompile::{
    bls12_381, {PrecompileError, PrecompileResult, PrecompileWithAddress},
};
use primitives::Bytes;

pub mod pair {
    use super::*;

    pub const ISTHMUS_MAX_INPUT_SIZE: usize = 235008;
    pub const ISTHMUS: PrecompileWithAddress = PrecompileWithAddress(
        precompile::u64_to_address(bls12_381::pairing::ADDRESS),
        |input, gas_limit| run_pair(input, gas_limit),
    );

    pub fn run_pair(input: &[u8], gas_limit: u64) -> PrecompileResult {
        if input.len() > ISTHMUS_MAX_INPUT_SIZE {
            return Err(
                PrecompileError::Other("BLS12-381 pairing input is too large".into()).into(),
            );
        }
        let input = Bytes::copy_from_slice(input);
        bls12_381::pairing::pairing(&input, gas_limit)
    }
}
