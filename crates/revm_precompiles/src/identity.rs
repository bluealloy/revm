use crate::{
    Precompile, PrecompileAddress, PrecompileOutput, PrecompileResult, StandardPrecompileFn,
};
use ruint::uint;

use super::{calc_linear_cost_u32, gas_query};

pub const FUN: PrecompileAddress = PrecompileAddress(
    uint!(4_B160),
    Precompile::Standard(identity_run as StandardPrecompileFn),
);

/// The base cost of the operation.
const IDENTITY_BASE: u64 = 15;
/// The cost per word.
const IDENTITY_PER_WORD: u64 = 3;

/// Takes the input bytes, copies them, and returns it as the output.
///
/// See: https://ethereum.github.io/yellowpaper/paper.pdf
/// See: https://etherscan.io/address/0000000000000000000000000000000000000004
fn identity_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    let gas_used = gas_query(
        calc_linear_cost_u32(input.len(), IDENTITY_BASE, IDENTITY_PER_WORD),
        gas_limit,
    )?;
    Ok(PrecompileOutput::without_logs(gas_used, input.to_vec()))
}
