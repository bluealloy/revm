use super::{
    g2::{encode_g2_point, extract_g2_input, G2_INPUT_ITEM_LENGTH},
    g2_mul,
    msm::msm_required_gas,
    utils::{extract_scalar_input, NBITS, SCALAR_LENGTH},
};
use crate::{u64_to_address, PrecompileWithAddress};
use crate::{Precompile, PrecompileError, PrecompileOutput, PrecompileResult};
use blst::{blst_p2, blst_p2_affine, blst_p2_from_affine, blst_p2_to_affine, p2_affines};
use primitives::Bytes;

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G2MSM precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(ADDRESS), Precompile::Standard(g2_msm));

/// BLS12_G2MSM precompile address.
pub const ADDRESS: u64 = 0x10;

/// Implements EIP-2537 G2MSM precompile.
/// G2 multi-scalar-multiplication call expects `288*k` bytes as an input that is interpreted
/// as byte concatenation of `k` slices each of them being a byte concatenation
/// of encoding of G2 point (`256` bytes) and encoding of a scalar value (`32`
/// bytes).
/// Output is an encoding of multi-scalar-multiplication operation result - single G2
/// point (`256` bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g2-multiexponentiation>
pub(super) fn g2_msm(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    let input_len = input.len();
    if input_len == 0 || input_len % g2_mul::INPUT_LENGTH != 0 {
        return Err(PrecompileError::Other(format!(
            "G2MSM input length should be multiple of {}, was {}",
            g2_mul::INPUT_LENGTH,
            input_len
        ))
        .into());
    }

    let k = input_len / g2_mul::INPUT_LENGTH;
    let required_gas = msm_required_gas(k, g2_mul::BASE_GAS_FEE);
    if required_gas > gas_limit {
        return Err(PrecompileError::OutOfGas.into());
    }

    let mut g2_points: Vec<blst_p2> = Vec::with_capacity(k);
    let mut scalars: Vec<u8> = Vec::with_capacity(k * SCALAR_LENGTH);
    for i in 0..k {
        let slice =
            &input[i * g2_mul::INPUT_LENGTH..i * g2_mul::INPUT_LENGTH + G2_INPUT_ITEM_LENGTH];
        // BLST batch API for p2_affines blows up when you pass it a point at infinity, so we must
        // filter points at infinity (and their corresponding scalars) from the input.
        if slice.iter().all(|i| *i == 0) {
            continue;
        }

        // NB: Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
        //
        // So we set the subgroup_check flag to `true`
        let p0_aff = &extract_g2_input(slice, true)?;

        let mut p0 = blst_p2::default();
        // SAFETY: p0 and p0_aff are blst values.
        unsafe { blst_p2_from_affine(&mut p0, p0_aff) };

        g2_points.push(p0);

        scalars.extend_from_slice(
            &extract_scalar_input(
                &input[i * g2_mul::INPUT_LENGTH + G2_INPUT_ITEM_LENGTH
                    ..i * g2_mul::INPUT_LENGTH + G2_INPUT_ITEM_LENGTH + SCALAR_LENGTH],
            )?
            .b,
        );
    }

    // return infinity point if all points are infinity
    if g2_points.is_empty() {
        return Ok(PrecompileOutput::new(required_gas, [0; 256].into()));
    }

    let points = p2_affines::from(&g2_points);
    let multiexp = points.mult(&scalars, NBITS);

    let mut multiexp_aff = blst_p2_affine::default();
    // SAFETY: multiexp_aff and multiexp are blst values.
    unsafe { blst_p2_to_affine(&mut multiexp_aff, &multiexp) };

    let out = encode_g2_point(&multiexp_aff);
    Ok(PrecompileOutput::new(required_gas, out))
}
