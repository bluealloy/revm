//! Identity precompile returns
use super::calc_linear_cost;
use crate::{
    call_eth_precompile, Precompile, PrecompileEthResult, PrecompileHalt, PrecompileId,
    PrecompileOutput, PrecompileOutputEth,
};
use primitives::Bytes;

/// Address of the identity precompile.
pub const FUN: Precompile = Precompile::new(
    PrecompileId::Identity,
    crate::u64_to_address(4),
    identity_precompile,
);

fn identity_precompile(input: &[u8], gas_limit: u64, reservoir: u64) -> PrecompileOutput {
    call_eth_precompile(identity_run, input, gas_limit, reservoir)
}

/// The base cost of the operation
pub const IDENTITY_BASE: u64 = 15;
/// The cost per word
pub const IDENTITY_PER_WORD: u64 = 3;

/// Takes the input bytes, copies them, and returns it as the output.
///
/// See: <https://ethereum.github.io/yellowpaper/paper.pdf>
///
/// See: <https://etherscan.io/address/0000000000000000000000000000000000000004>
pub fn identity_run(input: &[u8], gas_limit: u64) -> PrecompileEthResult {
    let gas_used = calc_linear_cost(input.len(), IDENTITY_BASE, IDENTITY_PER_WORD);
    if gas_used > gas_limit {
        return Err(PrecompileHalt::OutOfGas);
    }
    Ok(PrecompileOutputEth::new(
        gas_used,
        Bytes::copy_from_slice(input),
    ))
}
