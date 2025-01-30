//! Utilities for working with big endian and fp.

use bls12_381::{Scalar, G2Affine, G1Affine};
use revm_primitives::{Bytes, PrecompileError};

/// Number of bits used in the BLS12-381 curve finite field elements.
pub const NBITS: usize = 256;

/// Scalar length.
pub const SCALAR_LENGTH: usize = 32;

/// Finite field element input length.
pub const FP_LENGTH: usize = 48;

/// Finite field element padded input length.
pub const PADDED_FP_LENGTH: usize = 64;

/// Quadratic extension of finite field element input length.
pub const PADDED_FP2_LENGTH: usize = 128;

/// Input elements padding length.
pub const PADDING_LENGTH: usize = 16;

/// Length of each of the elements in a g1 operation input.
pub const G1_INPUT_ITEM_LENGTH: usize = 128;

/// Output length of a g1 operation.
pub const G1_OUTPUT_LENGTH: usize = 128;

/// Length of each of the elements in a g2 operation input.
pub const G2_INPUT_ITEM_LENGTH: usize = 256;

/// Output length of a g2 operation.
pub const G2_OUTPUT_LENGTH: usize = 256;

/// Amount used to calculate the multi-scalar-multiplication discount.
pub const MSM_MULTIPLIER: u64 = 1000;

/// Implements the gas schedule for G1/G2 Multiscalar-multiplication assuming 30
/// MGas/second, see also: <https://eips.ethereum.org/EIPS/eip-2537#g1g2-multiexponentiation>
pub fn msm_required_gas(k: usize, discount_table: &[u16], multiplication_cost: u64) -> u64 {
    if k == 0 {
        return 0;
    }
    let index = core::cmp::min(k - 1, discount_table.len() - 1);
    let discount = discount_table[index] as u64;
    (k as u64 * discount * multiplication_cost) / MSM_MULTIPLIER
}

/// Encodes a G1 point in affine format into byte slice with padded elements.
pub fn encode_g1_point(input: G1Affine) -> Bytes {
    let uncompressed = input.to_uncompressed();
    let mut out = vec![0u8; G1_OUTPUT_LENGTH];
    out[16..64].copy_from_slice(&uncompressed[..48]);
    out[80..128].copy_from_slice(&uncompressed[48..]);
    out.into()
}

/// Extracts a G1 point in Affine format from a 128 byte slice representation.
pub fn extract_g1_input(input: &[u8]) -> Result<G1Affine, PrecompileError> {
    if input.len() != G1_INPUT_ITEM_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {G1_INPUT_ITEM_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let input_p0_x = remove_padding(&input[..PADDED_FP_LENGTH])?;
    let input_p0_y = remove_padding(&input[PADDED_FP_LENGTH..G1_INPUT_ITEM_LENGTH])?;

    // Fill a new input array with the unpadded values
    let mut new_input: [u8; 96] = [0; 96];
    new_input[..48].copy_from_slice(input_p0_x);
    new_input[48..].copy_from_slice(input_p0_y);

    let g1_affine = G1Affine::from_uncompressed(&new_input).into_option().ok_or(PrecompileError::Other("Invalid G1 point".to_string()))?;

    if g1_affine.is_on_curve().unwrap_u8() == 0 {
        return Err(PrecompileError::Other("Element not on G1 Curve".to_string()));
    }

    Ok(g1_affine)
}

/// Extracts a G1 point in Affine format from a 128 byte slice representation.
/// Performs a subgroup check.
pub fn extract_g1_input_subgroup_check(input: &[u8]) -> Result<G1Affine, PrecompileError> {
    let g1_affine = extract_g1_input(input)?;

    if g1_affine.is_torsion_free().unwrap_u8() == 0 {
        return Err(PrecompileError::Other("Element not in G1".to_string()));
    }

    Ok(g1_affine)
}

/// Encodes a G2 point in affine format into byte slice with padded elements.
pub fn encode_g2_point(input: G2Affine) -> Bytes {
    let uncompressed = input.to_uncompressed();
    let mut out = vec![0u8; G2_OUTPUT_LENGTH];
    out[16..64].copy_from_slice(&uncompressed[..48]);
    out[80..128].copy_from_slice(&uncompressed[48..]);
    out[144..192].copy_from_slice(&uncompressed[96..144]);
    out[208..256].copy_from_slice(&uncompressed[144..]);
    out.into()
}

/// Extracts a G2 point in Affine format from a 256 byte slice representation.
pub fn extract_g2_input(input: &[u8]) -> Result<G2Affine, PrecompileError> {
    if input.len() != G2_INPUT_ITEM_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {G1_INPUT_ITEM_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let input_x0 = remove_padding(&input[..PADDED_FP_LENGTH])?;
    let input_x1 = remove_padding(&input[PADDED_FP_LENGTH..PADDED_FP_LENGTH*2])?;
    let input_y0 = remove_padding(&input[2*PADDED_FP_LENGTH..PADDED_FP_LENGTH*3])?;
    let input_y1 = remove_padding(&input[3*PADDED_FP_LENGTH..PADDED_FP_LENGTH*4])?;

    // Fill a new input array with the unpadded values
    let mut new_input: [u8; 192] = [0; 192];
    new_input[..48].copy_from_slice(input_x0);
    new_input[48..96].copy_from_slice(input_x1);
    new_input[96..144].copy_from_slice(input_y0);
    new_input[144..].copy_from_slice(input_y1);

    let g2_affine = G2Affine::from_uncompressed(&new_input).into_option().ok_or(PrecompileError::Other("Invalid G2 point".to_string()))?;

    if g2_affine.is_on_curve().unwrap_u8() == 0 {
        return Err(PrecompileError::Other("Element not on G2 Curve".to_string()));
    }

    Ok(g2_affine)
}

/// Extracts a G2 point in Affine format from a 256 byte slice representation.
/// Performs a subgroup check.
pub fn extract_g2_input_subgroup_check(input: &[u8]) -> Result<G2Affine, PrecompileError> {
    let g2_affine = extract_g2_input(input)?;

    if g2_affine.is_torsion_free().unwrap_u8() == 0 {
        return Err(PrecompileError::Other("Element not in G2".to_string()));
    }

    Ok(g2_affine)
}

/// Removes zeros with which the precompile inputs are left padded to 64 bytes.
pub fn remove_padding(input: &[u8]) -> Result<&[u8; FP_LENGTH], PrecompileError> {
    if input.len() != PADDED_FP_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Padded input should be {PADDED_FP_LENGTH} bytes, was {}",
            input.len()
        )));
    }
    let (padding, unpadded) = input.split_at(PADDING_LENGTH);
    if !padding.iter().all(|&x| x == 0) {
        return Err(PrecompileError::Other(format!(
            "{PADDING_LENGTH} top bytes of input are not zero",
        )));
    }
    Ok(unpadded.try_into().unwrap())
}

/// Extracts a scalar from a 32 byte slice representation, decoding the input as a big endian
/// unsigned integer. If the input is not exactly 32 bytes long, an error is returned.
///
/// From [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537):
/// * A scalar for the multiplication operation is encoded as 32 bytes by performing BigEndian
///   encoding of the corresponding (unsigned) integer.
///
/// We do not check that the scalar is a canonical Fr element, because the EIP specifies:
/// * The corresponding integer is not required to be less than or equal than main subgroup order
///   `q`.
pub(super) fn extract_scalar_input(input: &[u8]) -> Result<Scalar, PrecompileError> {
    if input.len() != SCALAR_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {SCALAR_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let mut arr = [0u8; 32];
    arr.copy_from_slice(input);
    Scalar::from_bytes(&arr).into_option().ok_or(PrecompileError::Other("Invalid scalar".to_string()))
}
