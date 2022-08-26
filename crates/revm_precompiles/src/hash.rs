use super::{calc_linear_cost_u32, gas_query};

use crate::{Precompile, PrecompileOutput, PrecompileResult, StandardPrecompileFn};
use primitive_types::H160 as Address;
use sha2::*;

pub const SHA256: (Address, Precompile) = (
    super::make_address(0, 2),
    Precompile::Standard(sha256_run as StandardPrecompileFn),
);
pub const RIPEMD160: (Address, Precompile) = (
    super::make_address(0, 3),
    Precompile::Standard(ripemd160_run as StandardPrecompileFn),
);

/// See: https://ethereum.github.io/yellowpaper/paper.pdf
/// See: https://docs.soliditylang.org/en/develop/units-and-global-variables.html#mathematical-and-cryptographic-functions
/// See: https://etherscan.io/address/0000000000000000000000000000000000000002
fn sha256_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    let cost = gas_query(calc_linear_cost_u32(input.len(), 60, 12), gas_limit)?;
    let output = sha2::Sha256::digest(input).to_vec();
    Ok(PrecompileOutput::without_logs(cost, output))
}

/// See: https://ethereum.github.io/yellowpaper/paper.pdf
/// See: https://docs.soliditylang.org/en/develop/units-and-global-variables.html#mathematical-and-cryptographic-functions
/// See: https://etherscan.io/address/0000000000000000000000000000000000000003
fn ripemd160_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    let gas_used = gas_query(calc_linear_cost_u32(input.len(), 600, 120), gas_limit)?;
    let mut ret = [0u8; 32];
    ret[12..32].copy_from_slice(&ripemd::Ripemd160::digest(input));
    Ok(PrecompileOutput::without_logs(gas_used, ret.to_vec()))
}
