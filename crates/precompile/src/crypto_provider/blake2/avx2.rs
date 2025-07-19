/// Code adapted from https://github.com/oconnor663/blake2_simd/blob/82b3e2aee4d2384aabbeb146058301ff0dbd453f/blake2b/src/avx2.rs
use super::algo::IV;

use arrayref::{array_refs, mut_array_refs};

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

// Adapted from https://github.com/rust-lang-nursery/stdsimd/pull/479.
macro_rules! _MM_SHUFFLE {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

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
unsafe fn g1(a: &mut __m256i, b: &mut __m256i, c: &mut __m256i, d: &mut __m256i, m: &mut __m256i) {
    *a = add(*a, *m);
    *a = add(*a, *b);
    *d = xor(*d, *a);
    *d = rot32(*d);
    *c = add(*c, *d);
    *b = xor(*b, *c);
    *b = rot24(*b);
}

#[inline(always)]
unsafe fn g2(a: &mut __m256i, b: &mut __m256i, c: &mut __m256i, d: &mut __m256i, m: &mut __m256i) {
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
