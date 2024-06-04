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
        println!("cycle-tracker-start: sha256");
        let output = if zk_op::contains_operation(&ZkOperation::Sha256) {
            zk_op::ZKVM_OPERATOR.get().unwrap().sha256_run(input.as_ref()).unwrap().into()
        } else {
            sha2::Sha256::digest(input).to_vec()
        };
        println!("cycle-tracker-end: sha256");

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
        let output = if zk_op::contains_operation(&ZkOperation::Ripemd160) {
            zk_op::ZKVM_OPERATOR.get().unwrap().ripemd160_run(input.as_ref()).unwrap().into()
        } else {
        // Not indented to keep the diff clean and make changes to the original code obvious

        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(input);
        let mut output = [0u8; 32];
        hasher.finalize_into((&mut output[12..]).into());
        output

        };

        Ok((gas_used, output.to_vec().into()))
    }
}
