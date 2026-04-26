//! Blake2 precompile. More details in [`run`].
//!
//! The compression function is vendored from
//! [`blake2b_simd`](https://github.com/oconnor663/blake2_simd) (MIT license),
//! with modifications for EIP-152 variable round counts.

use crate::{
    crypto, eth_precompile_fn, EthPrecompileOutput, EthPrecompileResult, Precompile,
    PrecompileHalt, PrecompileId,
};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
mod portable;

type Word = u64;

const F_ROUND: u64 = 1;
const INPUT_LENGTH: usize = 213;

const IV: [Word; 8] = [
    0x6A09E667F3BCC908,
    0xBB67AE8584CAA73B,
    0x3C6EF372FE94F82B,
    0xA54FF53A5F1D36F1,
    0x510E527FADE682D1,
    0x9B05688C2B3E6C1F,
    0x1F83D9ABFB41BD6B,
    0x5BE0CD19137E2179,
];

// SIGMA has spec period 10 (RFC 7693 §2.7). BLAKE2b runs 12 rounds by reusing
// SIGMA[0]/SIGMA[1] for rounds 10/11; for EIP-152's variable round count we
// must index with `r % 10`, not `r % 12`.
const SIGMA: [[u8; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

/// BLAKE2b compression function F (EIP-152).
///
/// Dispatches to the best available implementation (AVX2 or portable).
pub fn compress(rounds: u32, h: &mut [Word; 8], m: &[Word; 16], t: &[Word; 2], f: bool) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(target_feature = "avx2")]
        {
            unsafe { avx2::compress(rounds, h, m, t, f) };
            return;
        }
        #[cfg(all(not(target_feature = "avx2"), feature = "std"))]
        {
            if std::is_x86_feature_detected!("avx2") {
                unsafe { avx2::compress(rounds, h, m, t, f) };
                return;
            }
        }
    }
    portable::compress(rounds, h, m, t, f);
}

eth_precompile_fn!(blake2_precompile, run);

/// Blake2 precompile
pub const FUN: Precompile = Precompile::new(
    PrecompileId::Blake2F,
    crate::u64_to_address(9),
    blake2_precompile,
);

/// reference: <https://eips.ethereum.org/EIPS/eip-152>
/// input format:
/// [4 bytes for rounds][64 bytes for h][128 bytes for m][8 bytes for t_0][8 bytes for t_1][1 byte for f]
pub fn run(input: &[u8], gas_limit: u64) -> EthPrecompileResult {
    if input.len() != INPUT_LENGTH {
        return Err(PrecompileHalt::Blake2WrongLength);
    }

    // Parse number of rounds (4 bytes)
    let rounds = u32::from_be_bytes(input[..4].try_into().unwrap());
    let gas_used = rounds as u64 * F_ROUND;
    if gas_used > gas_limit {
        return Err(PrecompileHalt::OutOfGas);
    }

    // Parse final block flag
    let f = match input[212] {
        0 => false,
        1 => true,
        _ => return Err(PrecompileHalt::Blake2WrongFinalIndicatorFlag),
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

    crypto().blake2_compress(rounds, &mut h, &m, &[t_0, t_1], f);

    let mut out = [0u8; 64];
    for (i, h) in (0..64).step_by(8).zip(h.iter()) {
        out[i..i + 8].copy_from_slice(&h.to_le_bytes());
    }

    Ok(EthPrecompileOutput::new(gas_used, out.into()))
}
