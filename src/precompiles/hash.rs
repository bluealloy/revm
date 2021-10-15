use super::{calc_linear_cost_u32, gas_quert};

use crate::precompiles::{Precompile, PrecompileOutput, PrecompileResult};
use crate::{models::CallContext};
use primitive_types::H160 as Address;
use sha2::*;

/// SHA256 precompile.
pub struct SHA256;

impl SHA256 {
    pub(super) const ADDRESS: Address = super::make_address(0, 2);
}

impl Precompile for SHA256 {
    /// See: https://ethereum.github.io/yellowpaper/paper.pdf
    /// See: https://docs.soliditylang.org/en/develop/units-and-global-variables.html#mathematical-and-cryptographic-functions
    /// See: https://etherscan.io/address/0000000000000000000000000000000000000002
    fn run(
        input: &[u8],
        gas_limit: u64,
        _context: &CallContext,
        _is_static: bool,
    ) -> PrecompileResult {
        let cost = gas_quert(calc_linear_cost_u32(input.len(), 60, 12), gas_limit)?;
        let output = sha2::Sha256::digest(input).to_vec();
        Ok(PrecompileOutput::without_logs(cost, output))
    }
}

/// RIPEMD160 precompile.
pub struct RIPEMD160;

impl RIPEMD160 {
    pub(super) const ADDRESS: Address = super::make_address(0, 3);
}

impl Precompile for RIPEMD160 {
    /// See: https://ethereum.github.io/yellowpaper/paper.pdf
    /// See: https://docs.soliditylang.org/en/develop/units-and-global-variables.html#mathematical-and-cryptographic-functions
    /// See: https://etherscan.io/address/0000000000000000000000000000000000000003
    fn run(
        input: &[u8],
        gas_limit: u64,
        _context: &CallContext,
        _is_static: bool,
    ) -> PrecompileResult {
        let gas_used = gas_quert(calc_linear_cost_u32(input.len(), 600, 120), gas_limit)?;
        let mut ret = [0u8; 32];
        ret[12..32].copy_from_slice(&ripemd160::Ripemd160::digest(input));
        Ok(PrecompileOutput::without_logs(gas_used, ret.to_vec()))
    }
}

/*

#[cfg(test)]
mod tests {
    use crate::test_utils::new_context;

    use super::*;

    #[test]
    fn test_sha256() {
        let input = b"";
        let expected =
            hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
                .unwrap();

        let res = SHA256::run(input, 60, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);
    }

    #[test]
    fn test_ripemd160() {
        let input = b"";
        let expected =
            hex::decode("0000000000000000000000009c1185a5c5e9fc54612808977ee8f548b2258d31")
                .unwrap();

        let res = RIPEMD160::run(input, 600, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);
    }
}
*/
