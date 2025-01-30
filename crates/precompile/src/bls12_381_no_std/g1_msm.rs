//! G1 MSM precopmile.

use crate::{
    Precompile, PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress,
    bls12_381_no_std::utils::{msm_required_gas, extract_scalar_input, SCALAR_LENGTH, extract_g1_input_subgroup_check, encode_g1_point, G1_INPUT_ITEM_LENGTH},
};
use bls12_381::G1Projective;
use revm_primitives::{address, Address, Bytes};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G1MSM precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(ADDRESS, Precompile::Standard(g1_msm));

/// BLS12_G1MSM precompile address.
pub const ADDRESS: Address = address!("000000000000000000000000000000000000000c");

/// Base gas fee for BLS12-381 g1_mul operation.
pub const BASE_GAS_FEE: u64 = 12000;

/// Input length of g1_mul operation.
pub const INPUT_LENGTH: usize = 160;

/// Discounts table for G1 MSM as a vector of pairs `[k, discount]`.
pub static DISCOUNT_TABLE: [u16; 128] = [
    1000, 949, 848, 797, 764, 750, 738, 728, 719, 712, 705, 698, 692, 687, 682, 677, 673, 669, 665,
    661, 658, 654, 651, 648, 645, 642, 640, 637, 635, 632, 630, 627, 625, 623, 621, 619, 617, 615,
    613, 611, 609, 608, 606, 604, 603, 601, 599, 598, 596, 595, 593, 592, 591, 589, 588, 586, 585,
    584, 582, 581, 580, 579, 577, 576, 575, 574, 573, 572, 570, 569, 568, 567, 566, 565, 564, 563,
    562, 561, 560, 559, 558, 557, 556, 555, 554, 553, 552, 551, 550, 549, 548, 547, 547, 546, 545,
    544, 543, 542, 541, 540, 540, 539, 538, 537, 536, 536, 535, 534, 533, 532, 532, 531, 530, 529,
    528, 528, 527, 526, 525, 525, 524, 523, 522, 522, 521, 520, 520, 519,
];

/// Implements EIP-2537 G1MSM precompile.
/// G1 multi-scalar-multiplication call expects `160*k` bytes as an input that is interpreted
/// as byte concatenation of `k` slices each of them being a byte concatenation
/// of encoding of G1 point (`128` bytes) and encoding of a scalar value (`32`
/// bytes).
/// Output is an encoding of multi-scalar-multiplication operation result - single G1
/// point (`128` bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g1-multiexponentiation>
pub fn g1_msm(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    let input_len = input.len();
    if input_len == 0 || input_len % INPUT_LENGTH != 0 {
        return Err(PrecompileError::Other(format!(
            "G1MSM input length should be multiple of {}, was {}",
            INPUT_LENGTH, input_len
        ))
        .into());
    }

    let k = input_len / INPUT_LENGTH;
    let required_gas = msm_required_gas(k, &DISCOUNT_TABLE, BASE_GAS_FEE);
    if required_gas > gas_limit {
        return Err(PrecompileError::OutOfGas.into());
    }

    let mut points: Vec<G1Projective> = Vec::with_capacity(k * SCALAR_LENGTH);
    for i in 0..k {
        let slice = &input[i * INPUT_LENGTH..i * INPUT_LENGTH + G1_INPUT_ITEM_LENGTH];

        // BLST batch API for p1_affines blows up when you pass it a point at infinity, so we must
        // filter points at infinity (and their corresponding scalars) from the input.
        if slice.iter().all(|i| *i == 0) {
            continue;
        }

        // Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
        let p0_aff = &extract_g1_input_subgroup_check(slice)?;
        let p0: G1Projective = p0_aff.into();

        let scalar = extract_scalar_input(
            &input[i * INPUT_LENGTH + G1_INPUT_ITEM_LENGTH
                ..i * INPUT_LENGTH + G1_INPUT_ITEM_LENGTH + SCALAR_LENGTH],
        )?;

        // TODO: actually use pippenger's algorithm here.
        // EIP-2537 requires pippenger's algorithm to be used for MSM speedup that results in a discount.
        // Multiply the affine by the scalar.
        let projective = p0 * scalar;
        points.push(projective);
    }

    // return infinity point if all points are infinity
    if points.is_empty() {
        return Ok(PrecompileOutput::new(required_gas, [0; 128].into()));
    }

    // Accumulate all the points.
    let acc = points.iter().fold(G1Projective::default(), |acc, p| acc + p);

    let out = encode_g1_point(acc.into());
    Ok(PrecompileOutput::new(required_gas, out))
}
