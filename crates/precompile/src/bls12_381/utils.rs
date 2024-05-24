use blst::{blst_bendian_from_fp, blst_fp, blst_scalar, blst_scalar_from_bendian};
use revm_primitives::PrecompileError;

/// Number of bits used in the BLS12-381 curve finite field elements.
pub(super) const NBITS: usize = 256;
/// Finite field element input length.
pub(super) const FP_LENGTH: usize = 48;
/// Finite field element padded input length.
pub(super) const PADDED_FP_LENGTH: usize = 64;
/// Quadratic extension of finite field element input length.
pub(super) const PADDED_FP2_LENGTH: usize = 128;
/// Input elements padding length.
pub(super) const PADDING_LENGTH: usize = 16;
/// Scalar length.
pub(super) const SCALAR_LENGTH: usize = 32;

/// Encodes a single finite field element into a byte slice with padding.
pub(super) fn fp_to_bytes(out: &mut [u8], input: *const blst_fp) {
    if out.len() != PADDED_FP_LENGTH {
        return;
    }
    let (padding, rest) = out.split_at_mut(PADDING_LENGTH);
    padding.fill(0);
    // SAFETY: out length is checked previously, input is a blst value.
    unsafe { blst_bendian_from_fp(rest.as_mut_ptr(), input) };
}

/// Removes zeros with which the precompile inputs are left padded to 64 bytes.
pub(super) fn remove_padding(input: &[u8]) -> Result<&[u8; FP_LENGTH], PrecompileError> {
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
/// encoding of the corresponding (unsigned) integer.
///
/// We do not check that the scalar is a canonical Fr element, because the EIP specifies:
/// * The corresponding integer is not required to be less than or equal than main subgroup order
/// `q`.
pub(super) fn extract_scalar_input(input: &[u8]) -> Result<blst_scalar, PrecompileError> {
    if input.len() != SCALAR_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {SCALAR_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let mut out = blst_scalar::default();
    // SAFETY: input length is checked previously, out is a blst value.
    unsafe {
        // NOTE: we do not use `blst_scalar_fr_check` here because, from EIP-2537:
        //
        // * The corresponding integer is not required to be less than or equal than main subgroup
        // order `q`.
        blst_scalar_from_bendian(&mut out, input.as_ptr())
    };

    Ok(out)
}
