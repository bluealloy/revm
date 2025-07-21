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

    // Rounds 4 bytes
    let rounds = u32::from_be_bytes(input[..4].try_into().unwrap()) as usize;
    
    let gas_used = rounds as u64 * F_ROUND;
    if gas_used > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    // Parse inputs
    let mut h = [0u64; 8];
    let f: bool = input[212] != 0;

    // state vector h
    let h_be = &input[4..68];

    for (i, item) in h.iter_mut().enumerate() {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&h_be[i * 8..(i + 1) * 8]);
        *item = u64::from_le_bytes(buf);
    }

    // message block vector m
    let m: [u8; 128] = input[68..196].try_into().unwrap();

    // 2w-bit offset counter t
    let t_be = &input[196..212];
    let mut buf: [u8; 8] = t_be[..8].try_into().unwrap();
    let t0 = u64::from_le_bytes(buf);
    buf = t_be[8..].try_into().unwrap();
    let t1 = u64::from_le_bytes(buf);
    let t = [t0, t1];

    crate::crypto::blake2::compress(rounds, &mut h, &m, t, f);

    let mut out = [0u8; 64];
    for (i, h) in (0..64).step_by(8).zip(h.iter()) {
        out[i..i + 8].copy_from_slice(&h.to_le_bytes());
    }

    Ok(PrecompileOutput::new(gas_used, out.into()))
}