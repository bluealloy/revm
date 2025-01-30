//! G2 MSM precopmile.

use crate::{
    Precompile, PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress,
    bls12_381_no_std::utils::{
        msm_required_gas, extract_scalar_input, SCALAR_LENGTH, extract_g2_input_subgroup_check,
        encode_g2_point, G2_INPUT_ITEM_LENGTH,
    },
};
use bls12_381::G2Projective;
use revm_primitives::{address, Address, Bytes};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G2MSM precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(ADDRESS, Precompile::Standard(g2_msm));

/// BLS12_G2MSM precompile address.
pub const ADDRESS: Address = address!("000000000000000000000000000000000000000e");

/// Base gas fee for BLS12-381 g2_mul operation.
pub const BASE_GAS_FEE: u64 = 22500;

/// Input length of g2_mul operation.
pub const INPUT_LENGTH: usize = 288;

// Discounts table for G2 MSM as a vector of pairs `[k, discount]`:
pub static DISCOUNT_TABLE: [u16; 128] = [
    1000, 1000, 923, 884, 855, 832, 812, 796, 782, 770, 759, 749, 740, 732, 724, 717, 711, 704,
    699, 693, 688, 683, 679, 674, 670, 666, 663, 659, 655, 652, 649, 646, 643, 640, 637, 634, 632,
    629, 627, 624, 622, 620, 618, 615, 613, 611, 609, 607, 606, 604, 602, 600, 598, 597, 595, 593,
    592, 590, 589, 587, 586, 584, 583, 582, 580, 579, 578, 576, 575, 574, 573, 571, 570, 569, 568,
    567, 566, 565, 563, 562, 561, 560, 559, 558, 557, 556, 555, 554, 553, 552, 552, 551, 550, 549,
    548, 547, 546, 545, 545, 544, 543, 542, 541, 541, 540, 539, 538, 537, 537, 536, 535, 535, 534,
    533, 532, 532, 531, 530, 530, 529, 528, 528, 527, 526, 526, 525, 524, 524,
];

/// Implements EIP-2537 G2MSM precompile.
/// G2 multi-scalar-multiplication call expects `288*k` bytes as an input that is interpreted
/// as byte concatenation of `k` slices each of them being a byte concatenation
/// of encoding of G2 point (`256` bytes) and encoding of a scalar value (`32`
/// bytes).
/// Output is an encoding of multi-scalar-multiplication operation result - single G2
/// point (`256` bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g2-multiexponentiation>
pub fn g2_msm(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    let input_len = input.len();
    if input_len == 0 || input_len % INPUT_LENGTH != 0 {
        return Err(PrecompileError::Other(format!(
            "G2MSM input length should be multiple of {}, was {}",
            INPUT_LENGTH, input_len
        ))
        .into());
    }

    let k = input_len / INPUT_LENGTH;
    let required_gas = msm_required_gas(k, &DISCOUNT_TABLE, BASE_GAS_FEE);
    if required_gas > gas_limit {
        return Err(PrecompileError::OutOfGas.into());
    }

    let mut points: Vec<G2Projective> = Vec::with_capacity(k * SCALAR_LENGTH);
    for i in 0..k {
        let slice = &input[i * INPUT_LENGTH..i * INPUT_LENGTH + G2_INPUT_ITEM_LENGTH];

        // BLST batch API for p2_affines blows up when you pass it a point at infinity, so we must
        // filter points at infinity (and their corresponding scalars) from the input.
        if slice.iter().all(|i| *i == 0) {
            continue;
        }

        // Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
        let p0_aff = &extract_g2_input_subgroup_check(slice)?;
        let p0: G2Projective = p0_aff.into();

        let scalar = extract_scalar_input(
            &input[i * INPUT_LENGTH + G2_INPUT_ITEM_LENGTH
                ..i * INPUT_LENGTH + G2_INPUT_ITEM_LENGTH + SCALAR_LENGTH],
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
    let acc = points.iter().fold(G2Projective::default(), |acc, p| acc + p);

    let out = encode_g2_point(acc.into());
    Ok(PrecompileOutput::new(required_gas, out))
}

