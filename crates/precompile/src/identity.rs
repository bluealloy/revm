use super::calc_linear_cost_u32;
use crate::{Error, Precompile, PrecompileResult, PrecompileWithAddress};

pub const FUN: PrecompileWithAddress =
    PrecompileWithAddress(crate::u64_to_address(4), Precompile::Standard(identity_run));

/// The base cost of the operation.
const IDENTITY_BASE: u64 = 15;
/// The cost per word.
const IDENTITY_PER_WORD: u64 = 3;

/// Takes the input bytes, copies them, and returns it as the output.
///
/// See: <https://ethereum.github.io/yellowpaper/paper.pdf>
/// See: <https://etherscan.io/address/0000000000000000000000000000000000000004>
fn identity_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    let gas_used = calc_linear_cost_u32(input.len(), IDENTITY_BASE, IDENTITY_PER_WORD);
    if gas_used > gas_limit {
        return Err(Error::OutOfGas);
    }
    Ok((gas_used, input.to_vec()))
}
