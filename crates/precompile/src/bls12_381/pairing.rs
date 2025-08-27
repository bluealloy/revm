//! BLS12-381 pairing precompile. More details in [`pairing`]
use super::utils::{remove_g1_padding, remove_g2_padding};
use super::PairingPair;
use crate::bls12_381_const::{
    PADDED_G1_LENGTH, PADDED_G2_LENGTH, PAIRING_ADDRESS, PAIRING_INPUT_LENGTH,
    PAIRING_MULTIPLIER_BASE, PAIRING_OFFSET_BASE,
};
use crate::{
    crypto, Precompile, PrecompileError, PrecompileId, PrecompileOutput, PrecompileResult,
};
use primitives::B256;
use std::vec::Vec;

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_PAIRING precompile.
pub const PRECOMPILE: Precompile =
    Precompile::new(PrecompileId::Bls12Pairing, PAIRING_ADDRESS, pairing);

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
    if input_len == 0 || !input_len.is_multiple_of(PAIRING_INPUT_LENGTH) {
        return Err(PrecompileError::Bls12381PairingInputLength);
    }

    let k = input_len / PAIRING_INPUT_LENGTH;
    let required_gas: u64 = PAIRING_MULTIPLIER_BASE * k as u64 + PAIRING_OFFSET_BASE;
    if required_gas > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    // Collect pairs of points for the pairing check
    let mut pairs: Vec<PairingPair> = Vec::with_capacity(k);
    for i in 0..k {
        let encoded_g1_element =
            &input[i * PAIRING_INPUT_LENGTH..i * PAIRING_INPUT_LENGTH + PADDED_G1_LENGTH];
        let encoded_g2_element = &input[i * PAIRING_INPUT_LENGTH + PADDED_G1_LENGTH
            ..i * PAIRING_INPUT_LENGTH + PADDED_G1_LENGTH + PADDED_G2_LENGTH];

        let [a_x, a_y] = remove_g1_padding(encoded_g1_element)?;
        let [b_x_0, b_x_1, b_y_0, b_y_1] = remove_g2_padding(encoded_g2_element)?;

        pairs.push(((*a_x, *a_y), (*b_x_0, *b_x_1, *b_y_0, *b_y_1)));
    }

    let result = crypto().bls12_381_pairing_check(&pairs)?;
    let result = if result { 1 } else { 0 };

    Ok(PrecompileOutput::new(
        required_gas,
        B256::with_last_byte(result).into(),
    ))
}
