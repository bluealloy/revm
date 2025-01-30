//! BLS12-381 precompile with input size limits for Optimism.

use crate::primitives::{address, Address, Bytes, B256};
use bls12_381::{G1Affine, G2Affine, G2Prepared, Gt, MillerLoopResult};
use crate::{
    Precompile, PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress,
};

pub(crate) mod pair {
    use super::*;

    /// BLS12_PAIRING precompile address.
    const ADDRESS: Address = address!("000000000000000000000000000000000000000f");

    /// Multiplier gas fee for BLS12-381 pairing operation.
    const PAIRING_MULTIPLIER_BASE: u64 = 32600;

    /// Offset gas fee for BLS12-381 pairing operation.
    const PAIRING_OFFSET_BASE: u64 = 37700;

    /// Input length of pairing operation.
    const INPUT_LENGTH: usize = 384;

    /// The maximum input size for isthmus.
    const ISTHMUS_MAX_INPUT_SIZE: usize = 235008;

    /// The isthmus precompile for BLS12-381 pairing check.
    pub(crate) const ISTHMUS: PrecompileWithAddress = PrecompileWithAddress(
        ADDRESS,
        Precompile::Standard(|input, gas_limit| run_pair(input, gas_limit)),
    );

    /// Runs the pairing for the given input, limiting the input size.
    fn run_pair(input: &[u8], gas_limit: u64) -> PrecompileResult {
        if input.len() > ISTHMUS_MAX_INPUT_SIZE {
            return Err(
                PrecompileError::Other("BLS12-381 pairing input is too large".into()).into(),
            );
        }
        let input = Bytes::copy_from_slice(input);
        pairing(&input, gas_limit)
    }

    /// Pairing call expects 384*k (k being a positive integer) bytes as an inputs
    /// that is interpreted as byte concatenation of k slices. Each slice has the
    /// following structure:
    ///    * 128 bytes of G1 point encoding
    ///    * 256 bytes of G2 point encoding
    ///
    /// Each point is expected to be in the subgroup of order q.
    /// Output is 32 bytes where first 31 bytes are equal to 0x00 and the last byte
    /// is 0x01 if pairing result is equal to the multiplicative identity in a pairing
    /// target field and 0x00 otherwise.
    ///
    /// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-pairing>
    fn pairing(input: &Bytes, gas_limit: u64) -> PrecompileResult {
        let input_len = input.len();
        if input_len == 0 || input_len % INPUT_LENGTH != 0 {
            return Err(PrecompileError::Other(format!(
                "Pairing input length should be multiple of {INPUT_LENGTH}, was {input_len}"
            ))
            .into());
        }

        let k = input_len / INPUT_LENGTH;
        let required_gas: u64 = PAIRING_MULTIPLIER_BASE * k as u64 + PAIRING_OFFSET_BASE;
        if required_gas > gas_limit {
            return Err(PrecompileError::OutOfGas.into());
        }

        // Accumulator for the Fp12 multiplications of the miller loops.
        let mut acc = MillerLoopResult::default();
        for i in 0..k {
            // construct an array of len 96 and copy the input into it
            let start = i * INPUT_LENGTH;
            let end = start + 96;
            let input_arr: [u8; 96] = input[start..end].try_into().unwrap();
            let Some(g1_aff) = G1Affine::from_uncompressed(&input_arr).into_option() else {
                return Err(PrecompileError::Other("Failed to parse G1 point".into()).into());
            };
            let input_arr: [u8; 192] = input[end..end + 192].try_into().unwrap();
            let Some(g2_aff) = G2Affine::from_uncompressed(&input_arr).into_option() else {
                return Err(PrecompileError::Other("Failed to parse G2 point".into()).into());
            };
            let g2_prep = G2Prepared::from(g2_aff);
            let res = bls12_381::multi_miller_loop(&[(&g1_aff, &g2_prep)]);
            acc += res;
        }

        let res = acc.final_exponentiation();

        let mut result: u8 = 0;
        if res == Gt::identity() {
            result = 1;
        }
        Ok(PrecompileOutput::new(
            required_gas,
            B256::with_last_byte(result).into(),
        ))
    }
}
