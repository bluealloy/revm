use super::{blst::pairing_check, g1::extract_g1_input, g2::extract_g2_input};
use crate::bls12_381_const::{
    PADDED_G1_LENGTH, PADDED_G2_LENGTH, PAIRING_ADDRESS, PAIRING_INPUT_LENGTH,
    PAIRING_MULTIPLIER_BASE, PAIRING_OFFSET_BASE,
};
use crate::{PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress};
use primitives::{Bytes, B256};

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
pub(super) fn pairing(input: &Bytes, gas_limit: u64) -> PrecompileResult {
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
        // NB: Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
        // extract_g1_input and extract_g2_input perform the necessary checks
        let p1_aff = extract_g1_input(
            &input[i * PAIRING_INPUT_LENGTH..i * PAIRING_INPUT_LENGTH + PADDED_G1_LENGTH],
        )?;

        // NB: Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
        let p2_aff = extract_g2_input(
            &input[i * PAIRING_INPUT_LENGTH + PADDED_G1_LENGTH
                ..i * PAIRING_INPUT_LENGTH + PADDED_G1_LENGTH + PADDED_G2_LENGTH],
        )?;

        pairs.push((p1_aff, p2_aff));
    }

    let result = if pairing_check(&pairs) { 1 } else { 0 };

    Ok(PrecompileOutput::new(
        required_gas,
        B256::with_last_byte(result).into(),
    ))
}
