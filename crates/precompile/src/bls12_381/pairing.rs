//! BLS12-381 pairing precompile. More details in [`pairing`]
use super::crypto_backend::{pairing_check, read_g1, read_g2};
use super::utils::{remove_g1_padding, remove_g2_padding};
use crate::bls12_381_const::{
    PADDED_G1_LENGTH, PADDED_G2_LENGTH, PAIRING_ADDRESS, PAIRING_INPUT_LENGTH,
    PAIRING_MULTIPLIER_BASE, PAIRING_OFFSET_BASE,
};
use crate::{PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress};
use primitives::B256;
use std::vec::Vec;

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_PAIRING precompile.
pub const PRECOMPILE: PrecompileWithAddress = PrecompileWithAddress(PAIRING_ADDRESS, pairing);

/// Pairing call expects 384*k (k being a positive integer) bytes as an inputs
/// that is interpreted as byte concatenation of k slices. Each slice has the
/// following structure:
///    * 128 bytes of G1 point encoding
///    * 256 bytes of G2 point encoding
///
/// Each point is expected to be in the subgroup of order q.
/// Output is 32 bytes where first 31 bytes are equal to 0x00 and the last byte
/// is 0x01 if pairing result is equal to the multiplicative identity in a pairing
/// target field and 0x00 otherwise.
///
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-pairing>
pub fn pairing(input: &[u8], gas_limit: u64) -> PrecompileResult {
    let input_len = input.len();
    if input_len == 0 || input_len % PAIRING_INPUT_LENGTH != 0 {
        return Err(PrecompileError::Other(format!(
            "Pairing input length should be multiple of {PAIRING_INPUT_LENGTH}, was {input_len}"
        )));
    }

    let k = input_len / PAIRING_INPUT_LENGTH;
    let required_gas: u64 = PAIRING_MULTIPLIER_BASE * k as u64 + PAIRING_OFFSET_BASE;
    if required_gas > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    // Collect pairs of points for the pairing check
    let mut pairs = Vec::with_capacity(k);
    for i in 0..k {
        let encoded_g1_element =
            &input[i * PAIRING_INPUT_LENGTH..i * PAIRING_INPUT_LENGTH + PADDED_G1_LENGTH];
        let encoded_g2_element = &input[i * PAIRING_INPUT_LENGTH + PADDED_G1_LENGTH
            ..i * PAIRING_INPUT_LENGTH + PADDED_G1_LENGTH + PADDED_G2_LENGTH];

        // If either the G1 or G2 element is the encoded representation
        // of the point at infinity, then these two points are no-ops
        // in the pairing computation.
        //
        // Note: we do not skip the validation of these two elements even if
        // one of them is the point at infinity because we could have G1 be
        // the point at infinity and G2 be an invalid element or vice versa.
        // In that case, the precompile should error because one of the elements
        // was invalid.
        let g1_is_zero = encoded_g1_element.iter().all(|i| *i == 0);
        let g2_is_zero = encoded_g2_element.iter().all(|i| *i == 0);

        let [a_x, a_y] = remove_g1_padding(encoded_g1_element)?;
        let [b_x_0, b_x_1, b_y_0, b_y_1] = remove_g2_padding(encoded_g2_element)?;

        // NB: Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
        // extract_g1_input and extract_g2_input perform the necessary checks
        let p1_aff = read_g1(a_x, a_y)?;
        let p2_aff = read_g2(b_x_0, b_x_1, b_y_0, b_y_1)?;

        if !g1_is_zero & !g2_is_zero {
            pairs.push((p1_aff, p2_aff));
        }
    }
    let result = if pairing_check(&pairs) { 1 } else { 0 };

    Ok(PrecompileOutput::new(
        required_gas,
        B256::with_last_byte(result).into(),
    ))
}
