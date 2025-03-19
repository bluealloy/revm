use super::{
    blst::map_fp_to_g1 as blst_map_fp_to_g1,
    g1::encode_g1_point,
    utils::{fp_from_bendian, remove_padding},
};
use crate::bls12_381_const::{MAP_FP_TO_G1_ADDRESS, MAP_FP_TO_G1_BASE_GAS_FEE, PADDED_FP_LENGTH};
use crate::PrecompileWithAddress;
use crate::{PrecompileError, PrecompileOutput, PrecompileResult};
use primitives::Bytes;

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_MAP_FP_TO_G1 precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(MAP_FP_TO_G1_ADDRESS, map_fp_to_g1);

/// Field-to-curve call expects 64 bytes as an input that is interpreted as an
/// element of Fp. Output of this call is 128 bytes and is an encoded G1 point.
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-mapping-fp-element-to-g1-point>
pub(super) fn map_fp_to_g1(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if MAP_FP_TO_G1_BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    if input.len() != PADDED_FP_LENGTH {
        return Err(PrecompileError::Other(format!(
            "MAP_FP_TO_G1 input should be {PADDED_FP_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let input_p0 = remove_padding(input)?;
    let fp = fp_from_bendian(input_p0)?;
    let p_aff = blst_map_fp_to_g1(&fp);

    let out = encode_g1_point(&p_aff);
    Ok(PrecompileOutput::new(MAP_FP_TO_G1_BASE_GAS_FEE, out))
}

#[cfg(test)]
mod test {
    use super::*;
    use primitives::hex;

    #[test]
    fn sanity_test() {
        let input = Bytes::from(hex!("000000000000000000000000000000006900000000000000636f6e7472616374595a603f343061cd305a03f40239f5ffff31818185c136bc2595f2aa18e08f17"));
        let fail = map_fp_to_g1(&input, MAP_FP_TO_G1_BASE_GAS_FEE);
        assert_eq!(
            fail,
            Err(PrecompileError::Other("non-canonical fp value".to_string()))
        );
    }
}
