//! BLS12-381 map fp to g1 precompile. More details in [`map_fp_to_g1`]
use super::utils::{pad_g1_point, remove_fp_padding};
use crate::bls12_381_const::{MAP_FP_TO_G1_ADDRESS, MAP_FP_TO_G1_BASE_GAS_FEE, PADDED_FP_LENGTH};
use crate::{
    crypto, Precompile, PrecompileError, PrecompileId, PrecompileOutput, PrecompileResult,
};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_MAP_FP_TO_G1 precompile.
pub const PRECOMPILE: Precompile = Precompile::new(
    PrecompileId::Bls12MapFpToGp1,
    MAP_FP_TO_G1_ADDRESS,
    map_fp_to_g1,
);

/// Field-to-curve call expects 64 bytes as an input that is interpreted as an
/// element of Fp. Output of this call is 128 bytes and is an encoded G1 point.
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-mapping-fp-element-to-g1-point>
pub fn map_fp_to_g1(input: &[u8], gas_limit: u64) -> PrecompileResult {
    if MAP_FP_TO_G1_BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    if input.len() != PADDED_FP_LENGTH {
        return Err(PrecompileError::Bls12381MapFpToG1InputLength);
    }

    let input_p0 = remove_fp_padding(input)?;

    let unpadded_result = crypto().bls12_381_fp_to_g1(input_p0)?;

    // Pad the result for EVM compatibility
    let padded_result = pad_g1_point(&unpadded_result);

    Ok(PrecompileOutput::new(
        MAP_FP_TO_G1_BASE_GAS_FEE,
        padded_result.into(),
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use primitives::{hex, Bytes};

    #[test]
    fn sanity_test() {
        let input = Bytes::from(hex!("000000000000000000000000000000006900000000000000636f6e7472616374595a603f343061cd305a03f40239f5ffff31818185c136bc2595f2aa18e08f17"));
        let fail = map_fp_to_g1(&input, MAP_FP_TO_G1_BASE_GAS_FEE);
        assert_eq!(fail, Err(PrecompileError::NonCanonicalFp));
    }
}
