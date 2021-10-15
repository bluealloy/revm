use crate::{
    models::CallContext,
    precompiles::{Precompile, PrecompileOutput, PrecompileResult},
    ExitError,
};

use primitive_types::H160 as Address;

use super::{calc_linear_cost_u32, gas_quert};

/// Identity precompile costs.
mod costs {
    /// The base cost of the operation.
    pub(super) const IDENTITY_BASE: u64 = 15;

    /// The cost per word.
    pub(super) const IDENTITY_PER_WORD: u64 = 3;
}

mod consts {
    /// Length of the identity word.
    pub(super) const IDENTITY_WORD_LEN: u64 = 32;
}

pub struct Identity;

impl Identity {
    pub(super) const ADDRESS: Address = super::make_address(0, 4);
    fn required_gas(input: &[u8]) -> Result<u64, ExitError> {
        Ok(
            (input.len() as u64 + consts::IDENTITY_WORD_LEN - 1) / consts::IDENTITY_WORD_LEN
                * costs::IDENTITY_PER_WORD
                + costs::IDENTITY_BASE,
        )
    }
}

impl Precompile for Identity {
    /// Takes the input bytes, copies them, and returns it as the output.
    ///
    /// See: https://ethereum.github.io/yellowpaper/paper.pdf
    /// See: https://etherscan.io/address/0000000000000000000000000000000000000004
    fn run(
        input: &[u8],
        gas_limit: u64,
        _context: &CallContext,
        _is_static: bool,
    ) -> PrecompileResult {
        let gas_used = gas_quert(calc_linear_cost_u32(input.len(), 15, 3), gas_limit)?;
        Ok(PrecompileOutput::without_logs(gas_used, input.to_vec()))
    }
}
/*

#[cfg(test)]
mod tests {
    use evm::ExitError;

    use crate::test_utils::new_context;

    use super::*;

    #[test]
    fn test_identity() {
        let input = [0u8, 1, 2, 3];

        let expected = input[0..2].to_vec();
        let res = Identity::run(&input[0..2], 18, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        let expected = input.to_vec();
        let res = Identity::run(&input, 18, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // gas fail
        let res = Identity::run(&input[0..2], 17, &new_context(), false);

        assert!(matches!(res, Err(ExitError::OutOfGas)));

        // larger input
        let input = [
            0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let res = Identity::run(&input, 21, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, input.to_vec());
    }
}
*/
