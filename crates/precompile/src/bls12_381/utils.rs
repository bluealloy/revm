use crate::bls12_381_const::{
    FP_LENGTH, FP_PAD_BY, MODULUS_REPR, PADDED_FP_LENGTH, PADDED_G1_LENGTH, PADDED_G2_LENGTH,
};
use crate::PrecompileError;
use core::cmp::Ordering;

/// Removes zeros with which the precompile inputs are left padded to 64 bytes.
pub(super) fn remove_fp_padding(input: &[u8]) -> Result<&[u8; FP_LENGTH], PrecompileError> {
    if input.len() != PADDED_FP_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Padded input should be {PADDED_FP_LENGTH} bytes, was {}",
            input.len()
        )));
    }
    let (padding, unpadded) = input.split_at(FP_PAD_BY);
    if !padding.iter().all(|&x| x == 0) {
        return Err(PrecompileError::Other(format!(
            "{FP_PAD_BY} top bytes of input are not zero",
        )));
    }
    Ok(unpadded.try_into().unwrap())
}
/// remove_g1_padding removes the padding applied to the Fp elements that constitute the
/// encoded G1 element.
pub(super) fn remove_g1_padding(input: &[u8]) -> Result<[&[u8; 48]; 2], PrecompileError> {
    if input.len() != PADDED_G1_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {PADDED_G1_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let x = remove_fp_padding(&input[..PADDED_FP_LENGTH])?;
    let y = remove_fp_padding(&input[PADDED_FP_LENGTH..PADDED_G1_LENGTH])?;
    Ok([x, y])
}

/// remove_g2_padding removes the padding applied to the Fp elements that constitute the
/// encoded G2 element.
pub(super) fn remove_g2_padding(input: &[u8]) -> Result<[&[u8; 48]; 4], PrecompileError> {
    if input.len() != PADDED_G2_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {PADDED_G2_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let mut input_fps = [&[0; FP_LENGTH]; 4];
    for i in 0..4 {
        input_fps[i] = remove_fp_padding(&input[i * PADDED_FP_LENGTH..(i + 1) * PADDED_FP_LENGTH])?;
    }
    Ok(input_fps)
}

/// Checks if the input is a valid big-endian representation of a field element.
pub(super) fn is_valid_be(input: &[u8; 48]) -> bool {
    for (i, modulo) in input.iter().zip(MODULUS_REPR.iter()) {
        match i.cmp(modulo) {
            Ordering::Greater => return false,
            Ordering::Less => return true,
            Ordering::Equal => continue,
        }
    }
    // Return false if matching the modulus
    false
}
