//! Map an element of Fp to a point on G1 curve.

use crate::{
    Precompile, PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress,
    bls12_381_no_std::utils::{PADDED_FP_LENGTH, encode_g1_point, remove_padding},
};
use bls12_381::G1Affine;
use revm_primitives::{address, Address, Bytes};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_MAP_FP_TO_G1 precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(ADDRESS, Precompile::Standard(map_fp_to_g1));

/// BLS12_MAP_FP_TO_G1 precompile address.
pub const ADDRESS: Address = address!("0000000000000000000000000000000000000010");

/// Base gas fee for BLS12-381 map_fp_to_g1 operation.
const MAP_FP_TO_G1_BASE: u64 = 5500;

/// Field-to-curve call expects 64 bytes as an input that is interpreted as an
/// element of Fp. Output of this call is 128 bytes and is an encoded G1 point.
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-mapping-fp-element-to-g1-point>
pub fn map_fp_to_g1(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if MAP_FP_TO_G1_BASE > gas_limit {
        return Err(PrecompileError::OutOfGas.into());
    }

    if input.len() != PADDED_FP_LENGTH {
        return Err(PrecompileError::Other(format!(
            "MAP_FP_TO_G1 input should be {PADDED_FP_LENGTH} bytes, was {}",
            input.len()
        ))
        .into());
    }

    let input_p0 = remove_padding(input)?;
    let aff = G1Affine::from_compressed(&input_p0).into_option().ok_or_else(|| {
        PrecompileError::Other("non-canonical fp value".to_string())
    })?;

    let out = encode_g1_point(aff);
    Ok(PrecompileOutput::new(MAP_FP_TO_G1_BASE, out))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::primitives::hex;

    #[test]
    fn sanity_test() {
        let input = Bytes::from(hex!("000000000000000000000000000000006900000000000000636f6e7472616374595a603f343061cd305a03f40239f5ffff31818185c136bc2595f2aa18e08f17"));
        let fail = map_fp_to_g1(&input, MAP_FP_TO_G1_BASE);
        assert_eq!(
            fail,
            Err(PrecompileError::Other("non-canonical fp value".to_string()).into())
        );
    }
}
