//! Blake2 precompile. More details in [`run`]

use crate::{PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress};

const F_ROUND: u64 = 1;
const INPUT_LENGTH: usize = 213;

/// Blake2 precompile
pub const FUN: PrecompileWithAddress = PrecompileWithAddress(crate::u64_to_address(9), run);

/// reference: <https://eips.ethereum.org/EIPS/eip-152>
/// input format:
/// [4 bytes for rounds][64 bytes for h][128 bytes for m][8 bytes for t_0][8 bytes for t_1][1 byte for f]
pub fn run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    if input.len() != INPUT_LENGTH {
        return Err(PrecompileError::Blake2WrongLength);
    }

    // Parse number of rounds (4 bytes)
    let rounds = u32::from_be_bytes(input[..4].try_into().unwrap()) as usize;

    let gas_used = rounds as u64 * F_ROUND;
    if gas_used > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    // Parse final block flag
    let f = match input[212] {
        0 => false,
        1 => true,
        _ => return Err(PrecompileError::Blake2WrongFinalIndicatorFlag),
    };

    // Parse state vector h (8 × u64)
    let mut h = [0u64; 8];
    input[4..68]
        .chunks_exact(8)
        .enumerate()
        .for_each(|(i, chunk)| {
            h[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        });

    // Parse message block m (16 × u64)
    let mut m = [0u64; 16];
    input[68..196]
        .chunks_exact(8)
        .enumerate()
        .for_each(|(i, chunk)| {
            m[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        });

    // Parse offset counters
    let t_0 = u64::from_le_bytes(input[196..204].try_into().unwrap());
    let t_1 = u64::from_le_bytes(input[204..212].try_into().unwrap());

    crate::crypto::blake2::compress(rounds, &mut h, m, [t_0, t_1], f);

    let mut out = [0u8; 64];
    for (i, h) in (0..64).step_by(8).zip(h.iter()) {
        out[i..i + 8].copy_from_slice(&h.to_le_bytes());
    }

    Ok(PrecompileOutput::new(gas_used, out.into()))
}
