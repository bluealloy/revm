//! Blake2 precompile. More details in [`run`].
//!
//! The compression function is vendored from
//! [`blake2b_simd`](https://github.com/oconnor663/blake2_simd) (MIT license),
//! with modifications for EIP-152 variable round counts.

use crate::{
    crypto, eth_precompile_fn, EthPrecompileOutput, EthPrecompileResult, Precompile,
    PrecompileHalt, PrecompileId,
};

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(target_feature = "avx2", feature = "std")
))]
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
// On targets with no SIMD path the cfgs below collapse to the single `portable::compress` call,
// which is `const`, so clippy suggests making this `const` too. It cannot be: on x86 this performs
// runtime AVX2 feature detection and calls an `unsafe` intrinsic implementation.
#[allow(clippy::missing_const_for_fn)]
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

/// The portable (non-SIMD) compression function, reachable directly.
///
/// Not public API -- use [`compress`], which always selects the fastest implementation for the
/// target. This exists so benchmarks can measure the portable path on any host: [`compress`]
/// detects AVX2 at runtime, so on an x86_64 CI runner a benchmark going through it measures the
/// AVX2 path only, and the portable code that `no_std`, zkVM and non-x86 builds actually run is
/// never timed.
#[doc(hidden)]
#[inline]
pub fn compress_portable(rounds: u32, h: &mut [Word; 8], m: &[Word; 16], t: &[Word; 2], f: bool) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference implementation: the compression function written with a runtime round index,
    /// as it stood before the rounds were unrolled with a constant schedule. Deliberately kept
    /// naive so that it is easy to check against RFC 7693 §3.2 by eye.
    fn reference_compress(
        rounds: u32,
        words: &mut [Word; 8],
        m: &[Word; 16],
        t: &[Word; 2],
        f: bool,
    ) {
        fn g(v: &mut [Word; 16], a: usize, b: usize, c: usize, d: usize, x: Word, y: Word) {
            v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
            v[d] = (v[d] ^ v[a]).rotate_right(32);
            v[c] = v[c].wrapping_add(v[d]);
            v[b] = (v[b] ^ v[c]).rotate_right(24);
            v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
            v[d] = (v[d] ^ v[a]).rotate_right(16);
            v[c] = v[c].wrapping_add(v[d]);
            v[b] = (v[b] ^ v[c]).rotate_right(63);
        }

        let mut v = [
            words[0],
            words[1],
            words[2],
            words[3],
            words[4],
            words[5],
            words[6],
            words[7],
            IV[0],
            IV[1],
            IV[2],
            IV[3],
            IV[4] ^ t[0],
            IV[5] ^ t[1],
            IV[6] ^ if f { !0 } else { 0 },
            IV[7],
        ];

        for r in 0..rounds as usize {
            let s = SIGMA[r % 10];
            g(&mut v, 0, 4, 8, 12, m[s[0] as usize], m[s[1] as usize]);
            g(&mut v, 1, 5, 9, 13, m[s[2] as usize], m[s[3] as usize]);
            g(&mut v, 2, 6, 10, 14, m[s[4] as usize], m[s[5] as usize]);
            g(&mut v, 3, 7, 11, 15, m[s[6] as usize], m[s[7] as usize]);
            g(&mut v, 0, 5, 10, 15, m[s[8] as usize], m[s[9] as usize]);
            g(&mut v, 1, 6, 11, 12, m[s[10] as usize], m[s[11] as usize]);
            g(&mut v, 2, 7, 8, 13, m[s[12] as usize], m[s[13] as usize]);
            g(&mut v, 3, 4, 9, 14, m[s[14] as usize], m[s[15] as usize]);
        }

        for i in 0..8 {
            words[i] ^= v[i] ^ v[i + 8];
        }
    }

    /// splitmix64, so the inputs are varied but the failures are reproducible.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
    }

    /// The unrolled portable implementation must agree with the runtime-index reference for
    /// every round count, not just multiples of ten. Round counts 0..=40 cover an empty run, a
    /// partial first tile, the wraparound at ten, and a full second tile; the larger counts
    /// check that nothing drifts after many tiles.
    ///
    /// Note this calls `portable::compress` directly rather than the `compress` dispatcher, so
    /// the portable path is exercised on every target, including those that would otherwise
    /// dispatch to AVX2.
    #[test]
    fn portable_matches_runtime_index_reference() {
        let mut rng = Rng(0x0DDB_1A5E_5BAD_5EED);

        for rounds in (0u32..=40).chain([100, 101, 109, 110, 111, 1000, 4096]) {
            for _ in 0..64 {
                let mut h = [0u64; 8];
                for w in h.iter_mut() {
                    *w = rng.next();
                }
                let mut m = [0u64; 16];
                for w in m.iter_mut() {
                    *w = rng.next();
                }
                let t = [rng.next(), rng.next()];
                let f = rng.next() & 1 == 0;

                let mut got = h;
                let mut want = h;
                portable::compress(rounds, &mut got, &m, &t, f);
                reference_compress(rounds, &mut want, &m, &t, f);

                assert_eq!(
                    got, want,
                    "mismatch at rounds={rounds} f={f} h={h:?} m={m:?} t={t:?}"
                );
            }
        }
    }

    /// EIP-152 test vector 4: an oracle independent of both implementations above. Twelve
    /// rounds over "abc", the standard BLAKE2b parameters.
    #[test]
    fn eip152_vector_4() {
        let h_in: [u64; 8] = [
            0x6a09_e667_f2bd_c948,
            0xbb67_ae85_84ca_a73b,
            0x3c6e_f372_fe94_f82b,
            0xa54f_f53a_5f1d_36f1,
            0x510e_527f_ade6_82d1,
            0x9b05_688c_2b3e_6c1f,
            0x1f83_d9ab_fb41_bd6b,
            0x5be0_cd19_137e_2179,
        ];
        let mut m = [0u64; 16];
        m[0] = 0x0000_0000_0063_6261; // "abc"
        let t = [3u64, 0u64];

        let expected: [u64; 8] = [
            0x0d4d_1c98_3fa5_80ba,
            0xe9f6_129f_b697_276a,
            0xb7c4_5a68_142f_214c,
            0xd1a2_ffdb_6fbb_124b,
            0x2d79_ab2a_39c5_877d,
            0x95cc_3345_ded5_52c2,
            0x5a92_f1db_a88a_d318,
            0x2399_00d4_ed86_23b9,
        ];

        let mut h = h_in;
        portable::compress(12, &mut h, &m, &t, true);
        assert_eq!(h, expected, "portable");

        // And through the dispatcher, so whichever implementation this target selects is
        // checked against the same vector.
        let mut h = h_in;
        compress(12, &mut h, &m, &t, true);
        assert_eq!(h, expected, "dispatched");
    }
}
