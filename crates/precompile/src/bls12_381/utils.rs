//! BLS12-381 utilities for padding and unpadding of input.
use crate::bls12_381_const::{
    FP_LENGTH, FP_PAD_BY, G1_LENGTH, PADDED_FP_LENGTH, PADDED_G1_LENGTH, PADDED_G2_LENGTH,
};
use crate::PrecompileError;

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
pub(super) fn remove_g1_padding(input: &[u8]) -> Result<[&[u8; FP_LENGTH]; 2], PrecompileError> {
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
pub(super) fn remove_g2_padding(input: &[u8]) -> Result<[&[u8; FP_LENGTH]; 4], PrecompileError> {
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

/// Pads an unpadded G1 point (96 bytes) to the EVM-compatible format (128 bytes).
///
/// Takes a G1 point with 2 field elements of 48 bytes each and adds 16 bytes of
/// zero padding before each field element.
pub(super) fn pad_g1_point(unpadded: &[u8]) -> [u8; PADDED_G1_LENGTH] {
    assert_eq!(
        unpadded.len(),
        G1_LENGTH,
        "Invalid unpadded G1 point length"
    );

    let mut padded = [0u8; PADDED_G1_LENGTH];

    // x
    padded[FP_PAD_BY..PADDED_FP_LENGTH].copy_from_slice(&unpadded[0..FP_LENGTH]);
    // y
    padded[PADDED_FP_LENGTH + FP_PAD_BY..2 * PADDED_FP_LENGTH]
        .copy_from_slice(&unpadded[FP_LENGTH..G1_LENGTH]);

    padded
}

/// Pads an unpadded G2 point (192 bytes) to the EVM-compatible format (256 bytes).
///
/// Takes a G2 point with 4 field elements of 48 bytes each and adds 16 bytes of
/// zero padding before each field element.
pub(super) fn pad_g2_point(unpadded: &[u8]) -> [u8; PADDED_G2_LENGTH] {
    assert_eq!(
        unpadded.len(),
        4 * FP_LENGTH,
        "Invalid unpadded G2 point length"
    );

    let mut padded = [0u8; PADDED_G2_LENGTH];

    // x.c0
    padded[FP_PAD_BY..PADDED_FP_LENGTH].copy_from_slice(&unpadded[0..FP_LENGTH]);
    // x.c1
    padded[PADDED_FP_LENGTH + FP_PAD_BY..2 * PADDED_FP_LENGTH]
        .copy_from_slice(&unpadded[FP_LENGTH..2 * FP_LENGTH]);
    // y.c0
    padded[2 * PADDED_FP_LENGTH + FP_PAD_BY..3 * PADDED_FP_LENGTH]
        .copy_from_slice(&unpadded[2 * FP_LENGTH..3 * FP_LENGTH]);
    // y.c1
    padded[3 * PADDED_FP_LENGTH + FP_PAD_BY..4 * PADDED_FP_LENGTH]
        .copy_from_slice(&unpadded[3 * FP_LENGTH..4 * FP_LENGTH]);

    padded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_g1_point_roundtrip() {
        // Create test data
        let mut unpadded = [0u8; G1_LENGTH];
        for (i, byte) in unpadded.iter_mut().enumerate() {
            *byte = (i * 2 + 1) as u8;
        }

        // Pad the point
        let padded = pad_g1_point(&unpadded);

        // Remove padding
        let result = remove_g1_padding(&padded).unwrap();

        // Verify roundtrip
        assert_eq!(result[0], &unpadded[0..FP_LENGTH]);
        assert_eq!(result[1], &unpadded[FP_LENGTH..G1_LENGTH]);
    }

    #[test]
    fn test_pad_g2_point_roundtrip() {
        // Create test data for G2 point (192 bytes = 4 * 48)
        let mut unpadded = [0u8; 4 * FP_LENGTH];
        for (i, byte) in unpadded.iter_mut().enumerate() {
            *byte = (i * 2 + 1) as u8;
        }

        // Pad the point
        let padded = pad_g2_point(&unpadded);

        // Remove padding
        let result = remove_g2_padding(&padded).unwrap();

        // Verify roundtrip - G2 has 4 field elements
        assert_eq!(result[0], &unpadded[0..FP_LENGTH]);
        assert_eq!(result[1], &unpadded[FP_LENGTH..2 * FP_LENGTH]);
        assert_eq!(result[2], &unpadded[2 * FP_LENGTH..3 * FP_LENGTH]);
        assert_eq!(result[3], &unpadded[3 * FP_LENGTH..4 * FP_LENGTH]);
    }
}
