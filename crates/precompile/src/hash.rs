use super::calc_linear_cost_u32;
use crate::zk_op::{self, ZkOperation};
use crate::{Error, Precompile, PrecompileResult, PrecompileWithAddress};
use revm_primitives::Bytes;
use sha2::Digest;

pub const SHA256: PrecompileWithAddress =
    PrecompileWithAddress(crate::u64_to_address(2), Precompile::Standard(sha256_run));

pub const RIPEMD160: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(3),
    Precompile::Standard(ripemd160_run),
);

/// See: <https://ethereum.github.io/yellowpaper/paper.pdf>
/// See: <https://docs.soliditylang.org/en/develop/units-and-global-variables.html#mathematical-and-cryptographic-functions>
/// See: <https://etherscan.io/address/0000000000000000000000000000000000000002>
pub fn sha256_run(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    let cost = calc_linear_cost_u32(input.len(), 60, 12);
    if cost > gas_limit {
        Err(Error::OutOfGas)
    } else {
        if zk_op::contains_operation(&ZkOperation::Sha256) {
            if let Some(op) = zk_op::ZKVM_OPERATOR.get() {
                return op.sha256_run(input.as_ref()).map(|out| (cost, out.into()));
            }
        }
        let output = sha2::Sha256::digest(input).to_vec();
        Ok((cost, output.into()))
    }
}

/// See: <https://ethereum.github.io/yellowpaper/paper.pdf>
/// See: <https://docs.soliditylang.org/en/develop/units-and-global-variables.html#mathematical-and-cryptographic-functions>
/// See: <https://etherscan.io/address/0000000000000000000000000000000000000003>
pub fn ripemd160_run(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    let gas_used = calc_linear_cost_u32(input.len(), 600, 120);
    if gas_used > gas_limit {
        Err(Error::OutOfGas)
    } else {
        if zk_op::contains_operation(&zk_op::ZkOperation::Ripemd160) {
            if let Some(op) = zk_op::ZKVM_OPERATOR.get() {
                return op.ripemd160_run(input.as_ref()).map(|out| (gas_used, out.into()));
            }
        }
        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(input);
        let mut output = [0u8; 32];
        hasher.finalize_into((&mut output[12..]).into());
        Ok((gas_used, output.to_vec().into()))
    }
}
