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

    let f = match input[212] {
        1 => true,
        0 => false,
        _ => return Err(PrecompileError::Blake2WrongFinalIndicatorFlag),
    };

    let mut h = [0u64; 8];
    //let mut m = [0u64; 16];

    let t;
    // Optimized parsing using ptr::read_unaligned for potentially better performance

    let m: [u8; 16 * size_of::<u64>()];
    unsafe {
        let ptr = input.as_ptr();

        // Read h values
        for (i, item) in h.iter_mut().enumerate() {
            *item = u64::from_le_bytes(core::ptr::read_unaligned(
                ptr.add(4 + i * 8) as *const [u8; 8]
            ));
        }

        m = input[68..68 + 16 * size_of::<u64>()].try_into().unwrap();

        t = [
            u64::from_le_bytes(core::ptr::read_unaligned(ptr.add(196) as *const [u8; 8])),
            u64::from_le_bytes(core::ptr::read_unaligned(ptr.add(204) as *const [u8; 8])),
        ];
    }
    algo::compress(rounds, &mut h, &m, t, f);

    let mut out = [0u8; 64];
    for (i, h) in (0..64).step_by(8).zip(h.iter()) {
        out[i..i + 8].copy_from_slice(&h.to_le_bytes());
    }

    Ok(PrecompileOutput::new(gas_used, out.into()))
}

/// Blake2 algorithm
pub mod algo {
    /// SIGMA from spec: <https://datatracker.ietf.org/doc/html/rfc7693#section-2.7>
    pub const SIGMA: [[usize; 16]; 10] = [
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

    /// got IV from: <https://en.wikipedia.org/wiki/BLAKE_(hash_function)>
    pub const IV: [u64; 8] = [
        0x6a09e667f3bcc908,
        0xbb67ae8584caa73b,
        0x3c6ef372fe94f82b,
        0xa54ff53a5f1d36f1,
        0x510e527fade682d1,
        0x9b05688c2b3e6c1f,
        0x1f83d9abfb41bd6b,
        0x5be0cd19137e2179,
    ];

    #[inline(always)]
    #[allow(clippy::many_single_char_names)]
    /// G function: <https://tools.ietf.org/html/rfc7693#section-3.1>
    pub fn g(v: &mut [u64], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
        v[a] = v[a].wrapping_add(v[b]);
        v[a] = v[a].wrapping_add(x);
        v[d] ^= v[a];
        v[d] = v[d].rotate_right(32);
        v[c] = v[c].wrapping_add(v[d]);
        v[b] ^= v[c];
        v[b] = v[b].rotate_right(24);

        v[a] = v[a].wrapping_add(v[b]);
        v[a] = v[a].wrapping_add(y);
        v[d] ^= v[a];
        v[d] = v[d].rotate_right(16);
        v[c] = v[c].wrapping_add(v[d]);
        v[b] ^= v[c];
        v[b] = v[b].rotate_right(63);
    }

    /// Compression function F takes as an argument the state vector "h",
    /// message block vector "m" (last block is padded with zeros to full
    /// block size, if required), 2w-bit offset counter "t", and final block
    /// indicator flag "f".  Local vector v[0..15] is used in processing.  F
    /// returns a new state vector.  The number of rounds, "r", is 12 for
    /// BLAKE2b and 10 for BLAKE2s.  Rounds are numbered from 0 to r - 1.
    #[allow(clippy::many_single_char_names)]
    pub fn compress(
        rounds: usize,
        h: &mut [u64; 8],
        m_slice: &[u8; 16 * size_of::<u64>()],
        t: [u64; 2],
        f: bool,
    ) {
        assert!(m_slice.len() == 16 * size_of::<u64>());
        #[cfg(target_feature = "avx2")]
        {
            // avx2 gives 40% performance boost over portable implementation
            let block = m_slice;
            let words = h;
            let count = ((t[1] as u128) << 64) | (t[0] as u128);
            let last_block = if f { !0 } else { 0 };
            let last_node = 0;

            unsafe {
                super::avx2::compress_block(rounds, block, words, count, last_block, last_node);
            }
            return;
        }
        // if avx2 is not available, use the fallback portable implementation

        // Read m values
        let mut m = [0u64; 16];
        for (i, item) in m.iter_mut().enumerate() {
            *item = u64::from_le_bytes(unsafe {
                core::ptr::read_unaligned(m_slice.as_ptr().add(i * 8) as *const [u8; 8])
            });
        }

        let mut v = [0u64; 16];
        v[..h.len()].copy_from_slice(h); // First half from state.
        v[h.len()..].copy_from_slice(&IV); // Second half from IV.

        v[12] ^= t[0];
        v[13] ^= t[1];

        if f {
            v[14] = !v[14] // Invert all bits if the last-block-flag is set.
        }
        for i in 0..rounds {
            round(&mut v, &m, i);
        }

        for i in 0..8 {
            h[i] ^= v[i] ^ v[i + 8];
        }
    }

    #[inline(always)]
    fn round(v: &mut [u64; 16], m: &[u64; 16], r: usize) {
        // Message word selection permutation for this round.
        let s = &SIGMA[r % 10];
        // g1
        g(v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
        g(v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
        g(v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
        g(v, 3, 7, 11, 15, m[s[6]], m[s[7]]);

        // g2
        g(v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
        g(v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
        g(v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
        g(v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
    }
}

// Adapted from https://github.com/rust-lang-nursery/stdsimd/pull/479.
macro_rules! _MM_SHUFFLE {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

/// Code adapted from https://github.com/oconnor663/blake2_simd/blob/82b3e2aee4d2384aabbeb146058301ff0dbd453f/blake2b/src/avx2.rs
#[cfg(target_feature = "avx2")]
mod avx2 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    use super::algo::IV;
    use arrayref::{array_refs, mut_array_refs};

    type Word = u64;
    type Count = u128;
    /// The number input bytes passed to each call to the compression function. Small benchmarks need
    /// to use an even multiple of `BLOCKBYTES`, or else their apparent throughput will be low.
    const BLOCKBYTES: usize = 16 * size_of::<Word>();

    const DEGREE: usize = 4;

    /// Compress a block of data using the BLAKE2 algorithm.
    #[inline(always)]
    pub(crate) unsafe fn compress_block(
        mut rounds: usize,
        block: &[u8; BLOCKBYTES],
        words: &mut [Word; 8],
        count: Count,
        last_block: Word,
        last_node: Word,
    ) {
        let (words_low, words_high) = mut_array_refs!(words, DEGREE, DEGREE);
        let (iv_low, iv_high) = array_refs!(&IV, DEGREE, DEGREE);
        let mut a = loadu(words_low);
        let mut b = loadu(words_high);
        let mut c = loadu(iv_low);
        let flags = set4(count_low(count), count_high(count), last_block, last_node);
        let mut d = xor(loadu(iv_high), flags);

        let msg_chunks = array_refs!(block, 16, 16, 16, 16, 16, 16, 16, 16);
        let m0 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.0));
        let m1 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.1));
        let m2 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.2));
        let m3 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.3));
        let m4 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.4));
        let m5 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.5));
        let m6 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.6));
        let m7 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.7));

        let iv0 = a;
        let iv1 = b;
        let mut t0;
        let mut t1;
        let mut b0;

        loop {
            if rounds == 0 {
                break;
            }
            rounds -= 1;

            // round 1
            t0 = _mm256_unpacklo_epi64(m0, m1);
            t1 = _mm256_unpacklo_epi64(m2, m3);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpackhi_epi64(m0, m1);
            t1 = _mm256_unpackhi_epi64(m2, m3);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            diagonalize(&mut a, &mut b, &mut c, &mut d);
            t0 = _mm256_unpacklo_epi64(m7, m4);
            t1 = _mm256_unpacklo_epi64(m5, m6);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpackhi_epi64(m7, m4);
            t1 = _mm256_unpackhi_epi64(m5, m6);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            undiagonalize(&mut a, &mut b, &mut c, &mut d);

            if rounds == 0 {
                break;
            }
            rounds -= 1;

            // round 2
            t0 = _mm256_unpacklo_epi64(m7, m2);
            t1 = _mm256_unpackhi_epi64(m4, m6);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpacklo_epi64(m5, m4);
            t1 = _mm256_alignr_epi8(m3, m7, 8);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            diagonalize(&mut a, &mut b, &mut c, &mut d);
            t0 = _mm256_unpackhi_epi64(m2, m0);
            t1 = _mm256_blend_epi32(m5, m0, 0x33);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_alignr_epi8(m6, m1, 8);
            t1 = _mm256_blend_epi32(m3, m1, 0x33);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            undiagonalize(&mut a, &mut b, &mut c, &mut d);

            if rounds == 0 {
                break;
            }
            rounds -= 1;

            // round 3
            t0 = _mm256_alignr_epi8(m6, m5, 8);
            t1 = _mm256_unpackhi_epi64(m2, m7);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpacklo_epi64(m4, m0);
            t1 = _mm256_blend_epi32(m6, m1, 0x33);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            diagonalize(&mut a, &mut b, &mut c, &mut d);
            t0 = _mm256_alignr_epi8(m5, m4, 8);
            t1 = _mm256_unpackhi_epi64(m1, m3);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpacklo_epi64(m2, m7);
            t1 = _mm256_blend_epi32(m0, m3, 0x33);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            undiagonalize(&mut a, &mut b, &mut c, &mut d);

            if rounds == 0 {
                break;
            }
            rounds -= 1;

            // round 4
            t0 = _mm256_unpackhi_epi64(m3, m1);
            t1 = _mm256_unpackhi_epi64(m6, m5);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpackhi_epi64(m4, m0);
            t1 = _mm256_unpacklo_epi64(m6, m7);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            diagonalize(&mut a, &mut b, &mut c, &mut d);
            t0 = _mm256_alignr_epi8(m1, m7, 8);
            t1 = _mm256_shuffle_epi32(m2, _MM_SHUFFLE!(1, 0, 3, 2));
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpacklo_epi64(m4, m3);
            t1 = _mm256_unpacklo_epi64(m5, m0);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            undiagonalize(&mut a, &mut b, &mut c, &mut d);

            if rounds == 0 {
                break;
            }
            rounds -= 1;

            // round 5
            t0 = _mm256_unpackhi_epi64(m4, m2);
            t1 = _mm256_unpacklo_epi64(m1, m5);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_blend_epi32(m3, m0, 0x33);
            t1 = _mm256_blend_epi32(m7, m2, 0x33);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            diagonalize(&mut a, &mut b, &mut c, &mut d);
            t0 = _mm256_alignr_epi8(m7, m1, 8);
            t1 = _mm256_alignr_epi8(m3, m5, 8);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpackhi_epi64(m6, m0);
            t1 = _mm256_unpacklo_epi64(m6, m4);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            undiagonalize(&mut a, &mut b, &mut c, &mut d);

            if rounds == 0 {
                break;
            }
            rounds -= 1;

            // round 6
            t0 = _mm256_unpacklo_epi64(m1, m3);
            t1 = _mm256_unpacklo_epi64(m0, m4);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpacklo_epi64(m6, m5);
            t1 = _mm256_unpackhi_epi64(m5, m1);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            diagonalize(&mut a, &mut b, &mut c, &mut d);
            t0 = _mm256_alignr_epi8(m2, m0, 8);
            t1 = _mm256_unpackhi_epi64(m3, m7);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpackhi_epi64(m4, m6);
            t1 = _mm256_alignr_epi8(m7, m2, 8);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            undiagonalize(&mut a, &mut b, &mut c, &mut d);

            if rounds == 0 {
                break;
            }
            rounds -= 1;

            // round 7
            t0 = _mm256_blend_epi32(m0, m6, 0x33);
            t1 = _mm256_unpacklo_epi64(m7, m2);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpackhi_epi64(m2, m7);
            t1 = _mm256_alignr_epi8(m5, m6, 8);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            diagonalize(&mut a, &mut b, &mut c, &mut d);
            t0 = _mm256_unpacklo_epi64(m4, m0);
            t1 = _mm256_blend_epi32(m4, m3, 0x33);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpackhi_epi64(m5, m3);
            t1 = _mm256_shuffle_epi32(m1, _MM_SHUFFLE!(1, 0, 3, 2));
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            undiagonalize(&mut a, &mut b, &mut c, &mut d);

            if rounds == 0 {
                break;
            }
            rounds -= 1;
            // round 8
            t0 = _mm256_unpackhi_epi64(m6, m3);
            t1 = _mm256_blend_epi32(m1, m6, 0x33);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_alignr_epi8(m7, m5, 8);
            t1 = _mm256_unpackhi_epi64(m0, m4);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            diagonalize(&mut a, &mut b, &mut c, &mut d);
            t0 = _mm256_blend_epi32(m2, m1, 0x33);
            t1 = _mm256_alignr_epi8(m4, m7, 8);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpacklo_epi64(m5, m0);
            t1 = _mm256_unpacklo_epi64(m2, m3);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            undiagonalize(&mut a, &mut b, &mut c, &mut d);

            if rounds == 0 {
                break;
            }
            rounds -= 1;

            // round 9
            t0 = _mm256_unpacklo_epi64(m3, m7);
            t1 = _mm256_alignr_epi8(m0, m5, 8);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpackhi_epi64(m7, m4);
            t1 = _mm256_alignr_epi8(m4, m1, 8);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            diagonalize(&mut a, &mut b, &mut c, &mut d);
            t0 = _mm256_unpacklo_epi64(m5, m6);
            t1 = _mm256_unpackhi_epi64(m6, m0);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_alignr_epi8(m1, m2, 8);
            t1 = _mm256_alignr_epi8(m2, m3, 8);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            undiagonalize(&mut a, &mut b, &mut c, &mut d);

            if rounds == 0 {
                break;
            }
            rounds -= 1;

            // round 10
            t0 = _mm256_unpacklo_epi64(m5, m4);
            t1 = _mm256_unpackhi_epi64(m3, m0);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_unpacklo_epi64(m1, m2);
            t1 = _mm256_blend_epi32(m2, m3, 0x33);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            diagonalize(&mut a, &mut b, &mut c, &mut d);
            t0 = _mm256_unpackhi_epi64(m6, m7);
            t1 = _mm256_unpackhi_epi64(m4, m1);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
            t0 = _mm256_blend_epi32(m5, m0, 0x33);
            t1 = _mm256_unpacklo_epi64(m7, m6);
            b0 = _mm256_blend_epi32(t0, t1, 0xF0);
            g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
            undiagonalize(&mut a, &mut b, &mut c, &mut d);

            // last two rounds are removed
        }
        a = xor(a, c);
        b = xor(b, d);
        a = xor(a, iv0);
        b = xor(b, iv1);

        storeu(a, words_low);
        storeu(b, words_high);
    }

    #[inline(always)]
    pub(crate) fn count_low(count: Count) -> Word {
        count as Word
    }

    #[inline(always)]
    pub(crate) fn count_high(count: Count) -> Word {
        (count >> 8 * size_of::<Word>()) as Word
    }

    #[inline(always)]
    unsafe fn loadu(src: *const [Word; DEGREE]) -> __m256i {
        // This is an unaligned load, so the pointer cast is allowed.
        _mm256_loadu_si256(src as *const __m256i)
    }

    #[inline(always)]
    unsafe fn storeu(src: __m256i, dest: *mut [Word; DEGREE]) {
        // This is an unaligned store, so the pointer cast is allowed.
        _mm256_storeu_si256(dest as *mut __m256i, src)
    }

    #[inline(always)]
    unsafe fn loadu_128(mem_addr: &[u8; 16]) -> __m128i {
        _mm_loadu_si128(mem_addr.as_ptr() as *const __m128i)
    }

    #[inline(always)]
    unsafe fn add(a: __m256i, b: __m256i) -> __m256i {
        _mm256_add_epi64(a, b)
    }

    #[inline(always)]
    unsafe fn xor(a: __m256i, b: __m256i) -> __m256i {
        _mm256_xor_si256(a, b)
    }

    #[inline(always)]
    unsafe fn set4(a: u64, b: u64, c: u64, d: u64) -> __m256i {
        _mm256_setr_epi64x(a as i64, b as i64, c as i64, d as i64)
    }

    // These rotations are the "simple version". For the "complicated version", see
    // https://github.com/sneves/blake2-avx2/blob/b3723921f668df09ece52dcd225a36d4a4eea1d9/blake2b-common.h#L43-L46.
    // For a discussion of the tradeoffs, see
    // https://github.com/sneves/blake2-avx2/pull/5. In short:
    // - Due to an LLVM bug (https://bugs.llvm.org/show_bug.cgi?id=44379), this
    //   version performs better on recent x86 chips.
    // - LLVM is able to optimize this version to AVX-512 rotation instructions
    //   when those are enabled.
    #[inline(always)]
    unsafe fn rot32(x: __m256i) -> __m256i {
        _mm256_or_si256(_mm256_srli_epi64(x, 32), _mm256_slli_epi64(x, 64 - 32))
    }

    #[inline(always)]
    unsafe fn rot24(x: __m256i) -> __m256i {
        _mm256_or_si256(_mm256_srli_epi64(x, 24), _mm256_slli_epi64(x, 64 - 24))
    }

    #[inline(always)]
    unsafe fn rot16(x: __m256i) -> __m256i {
        _mm256_or_si256(_mm256_srli_epi64(x, 16), _mm256_slli_epi64(x, 64 - 16))
    }

    #[inline(always)]
    unsafe fn rot63(x: __m256i) -> __m256i {
        _mm256_or_si256(_mm256_srli_epi64(x, 63), _mm256_slli_epi64(x, 64 - 63))
    }

    #[inline(always)]
    unsafe fn g1(
        a: &mut __m256i,
        b: &mut __m256i,
        c: &mut __m256i,
        d: &mut __m256i,
        m: &mut __m256i,
    ) {
        *a = add(*a, *m);
        *a = add(*a, *b);
        *d = xor(*d, *a);
        *d = rot32(*d);
        *c = add(*c, *d);
        *b = xor(*b, *c);
        *b = rot24(*b);
    }

    #[inline(always)]
    unsafe fn g2(
        a: &mut __m256i,
        b: &mut __m256i,
        c: &mut __m256i,
        d: &mut __m256i,
        m: &mut __m256i,
    ) {
        *a = add(*a, *m);
        *a = add(*a, *b);
        *d = xor(*d, *a);
        *d = rot16(*d);
        *c = add(*c, *d);
        *b = xor(*b, *c);
        *b = rot63(*b);
    }

    // Note the optimization here of leaving b as the unrotated row, rather than a.
    // All the message loads below are adjusted to compensate for this. See
    // discussion at https://github.com/sneves/blake2-avx2/pull/4
    #[inline(always)]
    unsafe fn diagonalize(a: &mut __m256i, _b: &mut __m256i, c: &mut __m256i, d: &mut __m256i) {
        *a = _mm256_permute4x64_epi64(*a, _MM_SHUFFLE!(2, 1, 0, 3));
        *d = _mm256_permute4x64_epi64(*d, _MM_SHUFFLE!(1, 0, 3, 2));
        *c = _mm256_permute4x64_epi64(*c, _MM_SHUFFLE!(0, 3, 2, 1));
    }

    #[inline(always)]
    unsafe fn undiagonalize(a: &mut __m256i, _b: &mut __m256i, c: &mut __m256i, d: &mut __m256i) {
        *a = _mm256_permute4x64_epi64(*a, _MM_SHUFFLE!(0, 3, 2, 1));
        *d = _mm256_permute4x64_epi64(*d, _MM_SHUFFLE!(1, 0, 3, 2));
        *c = _mm256_permute4x64_epi64(*c, _MM_SHUFFLE!(2, 1, 0, 3));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::hex;
    use std::time::Instant;

    #[test]
    fn perfblake2() {
        let input = [hex!("0000040048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b616162636465666768696a6b6c6d6e6f700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001")
        ,hex!("0000020048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001")
        ,hex!("0000004048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001")];

        /*
        "bc112be5618b20d24be64c9e1c6efd63fea38cc79d53692fad6568b16e953eb6128c1ec8ffaf9a2d69e3cb043d6e11e1c7afd48573311052b6e7ec0960371186":
        "a2c1eb780a6e1249156fe0751e5d4687ea9357b0651c78df660ab004cb4773636298bbbc683e4a0261574b6d857a6a99e06b2eea50b16f86343d2625ff222b98":
        "74097ae7b16ffd18c742aee5c55dc89d54b6f1a8a19e6139ccfb38afba56b6b02cc35c441c19c21194fefb6841e72202f7c9d05eb9c3cfd8f94c67aa77d473c1"

        "bc112be5618b20d24be64c9e1c6efd63fea38cc79d53692fad6568b16e953eb6128c1ec8ffaf9a2d69e3cb043d6e11e1c7afd48573311052b6e7ec0960371186":
        "a2c1eb780a6e1249156fe0751e5d4687ea9357b0651c78df660ab004cb4773636298bbbc683e4a0261574b6d857a6a99e06b2eea50b16f86343d2625ff222b98":
        "74097ae7b16ffd18c742aee5c55dc89d54b6f1a8a19e6139ccfb38afba56b6b02cc35c441c19c21194fefb6841e72202f7c9d05eb9c3cfd8f94c67aa77d473c1":
        */

        println!(
            "{:?}:",
            hex::encode(&run(&input[0], u64::MAX).unwrap().bytes)
        );
        println!(
            "{:?}:",
            hex::encode(&run(&input[1], u64::MAX).unwrap().bytes)
        );
        println!(
            "{:?}:",
            hex::encode(&run(&input[2], u64::MAX).unwrap().bytes)
        );

        let time = Instant::now();
        for i in 0..3000 {
            let _ = run(&input[i % 3], u64::MAX).unwrap();
        }
        let duration = time.elapsed();
        println!("{:?}", duration);

        /*
        cargo test --package revm-precompile --lib -- blake2::tests::perfblake2 --exact --show-output
         */

        // let time = Instant::now();
        // for i in 0..50000 {
        //     let mut hasher = blake2b_simd::Params::new()
        //         .hash_length(32)
        //         .personal(b"test")
        //         .to_state();
        //     hasher.update(&input[i % 3]);
        //     let _ = hasher.finalize();
        // }

        // for i in 0..3 {
        //     let mut hasher = blake2::Blake2b::new();
        //     hasher.update(&input[i]);
        //     let out1: [u8; 32] = hasher.finalize().into();
        //     let out2 = run(&input[i], u64::MAX).unwrap();
        //     assert_eq!(out1, &out2.bytes[..32]);
        // }

        // let duration = time.elapsed();
        // println!("time: {:?}", duration);
    }
}
