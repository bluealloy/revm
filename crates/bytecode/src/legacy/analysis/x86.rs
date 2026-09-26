//! Jump destination analysis with SSE4.1, AVX2, and AVX-512 VBMI.
//!
//! For every byte of a block, the successor map holds the offset of the next instruction if one
//! started at that byte. Squaring
//! the map repeatedly yields the maps for 2, 4, 8, ... instructions, and the widest one gives the
//! offset where the path from the block's entry leaves the block, which is the entry of the next
//! block. For each byte, a binary search down the maps then finds the furthest instruction on the
//! path that does not pass the byte; the byte starts an instruction if that is the byte itself.
//!
//! The 16-byte kernels add `0x70` to offsets inside a block, so offsets past the block have bit 7
//! set. `pshufb` returns zero for those indices, and since paths only move forward, an unsigned
//! max with the index recovers the offset.

use super::{analyze_blocks, lanes, uncarried, Entry, Kernel, END_OFFSET, PUSHX};
use crate::opcode;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

const JUMPDEST: i8 = opcode::JUMPDEST as i8;
const BIAS: i8 = 0x70;

// SAFETY: the vector types are plain bytes.
const LANES16: __m128i = unsafe { core::mem::transmute(lanes::<16>(16, BIAS as u8)) };
const END_OFFSETS16: __m128i =
    unsafe { core::mem::transmute(lanes::<16>(16, (BIAS as u8).wrapping_sub(END_OFFSET))) };
const LANES16X2: __m256i = unsafe { core::mem::transmute(lanes::<32>(16, BIAS as u8)) };
const END_OFFSETS16X2: __m256i =
    unsafe { core::mem::transmute(lanes::<32>(16, (BIAS as u8).wrapping_sub(END_OFFSET))) };
const LANES64: __m512i = unsafe { core::mem::transmute(lanes::<64>(64, 0)) };
const END_OFFSETS64: __m512i =
    unsafe { core::mem::transmute(lanes::<64>(64, 0u8.wrapping_sub(END_OFFSET))) };
/// Second table of `vpermt2b`, which keeps offsets past the block.
const EXITS64: __m512i = unsafe { core::mem::transmute(lanes::<64>(64, 64)) };

/// Returns whether the CPU supports all of the given target features.
macro_rules! detected {
    ($($feature:tt),+ $(,)?) => {
        cfg!(all($(target_feature = $feature),+)) || {
            #[cfg(feature = "std")]
            let detected = $(std::is_x86_feature_detected!($feature))&&+;
            #[cfg(not(feature = "std"))]
            let detected = false;
            detected
        }
    };
}

pub(super) fn analyze(code: &[u8], table: &mut [u8]) -> usize {
    // SAFETY: the required target features are available.
    unsafe {
        if detected!("avx512f", "avx512bw", "avx512vbmi") {
            analyze_avx512vbmi(code, table)
        } else if detected!("avx2") {
            analyze_avx2(code, table)
        } else if detected!("sse4.1") {
            analyze_sse41(code, table)
        } else {
            0
        }
    }
}

#[target_feature(enable = "sse4.1")]
unsafe fn analyze_sse41(code: &[u8], table: &mut [u8]) -> usize {
    let state = unsafe { analyze_blocks::<Sse41x4>(code, table, (0, 0)) };
    let (pc, entry) = unsafe { analyze_blocks::<Sse41>(code, table, state) };
    pc + entry
}

#[target_feature(enable = "avx2")]
unsafe fn analyze_avx2(code: &[u8], table: &mut [u8]) -> usize {
    let state = unsafe { analyze_blocks::<Avx2>(code, table, (0, 0)) };
    let (pc, entry) = unsafe { analyze_blocks::<Sse41>(code, table, state) };
    pc + entry
}

#[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
unsafe fn analyze_avx512vbmi(code: &[u8], table: &mut [u8]) -> usize {
    let state = unsafe { analyze_blocks::<Avx512Vbmix2>(code, table, (0, 0)) };
    let state = unsafe { analyze_blocks::<Avx512Vbmi>(code, table, state) };
    let (pc, entry) = unsafe { analyze_blocks::<Sse41>(code, table, state) };
    pc + entry
}

/// 16-byte blocks with SSE4.1.
struct Sse41;

impl Kernel for Sse41 {
    type Bits = u16;

    type Entry = __m128i;

    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn block(ptr: *const u8, entry: &mut __m128i) -> u16 {
        // SAFETY: the caller guarantees 16 bytes at `ptr`.
        let ops = unsafe { _mm_loadu_si128(ptr.cast()) };
        starts16(ops, entry) & jumpdests16(ops)
    }
}

/// Biased by [`BIAS`], so an entry past the block has bit 7 set.
impl Entry for __m128i {
    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn new(offset: usize) -> Self {
        _mm_set1_epi8((BIAS as u8).wrapping_add(offset as u8) as i8)
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn offset(self) -> usize {
        (_mm_cvtsi128_si32(self) as u8).wrapping_sub(BIAS as u8) as usize
    }
}

/// Squares the successor map, keeping offsets past the block.
#[inline]
#[target_feature(enable = "sse4.1")]
fn compose16(successors: __m128i) -> __m128i {
    _mm_max_epu8(_mm_shuffle_epi8(successors, successors), successors)
}

/// Returns the successor maps of `ops` for 1, 2, 4, 8 and 16 instructions.
#[inline]
#[target_feature(enable = "sse4.1")]
fn successors16(ops: __m128i) -> [__m128i; 5] {
    let n0 = _mm_add_epi8(_mm_max_epi8(ops, _mm_set1_epi8(PUSHX)), END_OFFSETS16);
    let n1 = compose16(n0);
    let n2 = compose16(n1);
    let n3 = compose16(n2);
    [n0, n1, n2, n3, compose16(n3)]
}

/// Returns the entry of the next block, given the entry of the block that `exits` leaves.
///
/// 16 instructions always leave a 16-byte block, so every exit is at least `0x80` before
/// rebasing. When the entry is itself past the block, the lookup is zero and the max rebases the
/// entry instead.
#[inline]
#[target_feature(enable = "sse4.1")]
fn next_entry16(exits: __m128i, entry: __m128i) -> __m128i {
    let block = _mm_set1_epi8(16);
    _mm_max_epu8(
        _mm_shuffle_epi8(_mm_sub_epi8(exits, block), entry),
        _mm_sub_epi8(entry, block),
    )
}

/// Moves each lane's position along the successor map if the result does not pass the lane.
#[inline]
#[target_feature(enable = "sse4.1")]
fn descend16(successors: __m128i, positions: __m128i) -> __m128i {
    let next = _mm_shuffle_epi8(successors, positions);
    let stays = _mm_cmpeq_epi8(_mm_min_epu8(next, LANES16), next);
    _mm_blendv_epi8(positions, next, stays)
}

/// Returns the lanes of the 16 bytes in `ops` that start an instruction, and moves `entry` past
/// them.
#[inline]
#[target_feature(enable = "sse4.1")]
fn starts16(ops: __m128i, entry: &mut __m128i) -> u16 {
    let [n0, n1, n2, n3, n4] = successors16(ops);
    let block_entry = *entry;
    *entry = next_entry16(n4, block_entry);
    let mut position = descend16(n3, block_entry);
    position = descend16(n2, position);
    position = descend16(n1, position);
    // One step at most remains, so a lane starts an instruction if it is the position or the
    // position's successor.
    let starts = _mm_or_si128(
        _mm_cmpeq_epi8(position, LANES16),
        _mm_cmpeq_epi8(_mm_shuffle_epi8(n0, position), LANES16),
    );
    // No lane starts an instruction if the entry is past the block.
    (_mm_movemask_epi8(starts) & !_mm_movemask_epi8(block_entry)) as u16
}

/// Returns the JUMPDEST lanes of the 16 bytes in `ops`.
#[inline]
#[target_feature(enable = "sse4.1")]
fn jumpdests16(ops: __m128i) -> u16 {
    _mm_movemask_epi8(_mm_cmpeq_epi8(ops, _mm_set1_epi8(JUMPDEST))) as u16
}

/// 64-byte blocks of four 16-byte blocks with SSE4.1, skipping work for blocks without PUSH or
/// JUMPDEST opcodes.
struct Sse41x4;

impl Kernel for Sse41x4 {
    type Bits = u64;

    type Entry = __m128i;

    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn block(ptr: *const u8, entry: &mut __m128i) -> u64 {
        // SAFETY: the caller guarantees 64 bytes at `ptr`.
        let ops =
            unsafe { core::array::from_fn::<_, 4, _>(|i| _mm_loadu_si128(ptr.add(i * 16).cast())) };
        let jumpdests = (0..4).fold(0, |acc, i| acc | (jumpdests16(ops[i]) as u64) << (i * 16));
        let max = _mm_max_epi8(_mm_max_epi8(ops[0], ops[1]), _mm_max_epi8(ops[2], ops[3]));
        if _mm_movemask_epi8(_mm_cmpgt_epi8(max, _mm_set1_epi8(PUSHX))) == 0 {
            return unsafe { uncarried(jumpdests, entry) };
        }
        if jumpdests == 0 {
            for ops in ops {
                *entry = next_entry16(successors16(ops)[4], *entry);
            }
            return 0;
        }
        let starts = (0..4).fold(0, |acc, i| {
            acc | (starts16(ops[i], entry) as u64) << (i * 16)
        });
        starts & jumpdests
    }
}

/// 32-byte blocks of two 16-byte blocks with AVX2.
struct Avx2;

impl Kernel for Avx2 {
    type Bits = u32;

    type Entry = __m128i;

    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn block(ptr: *const u8, entry: &mut __m128i) -> u32 {
        // SAFETY: the caller guarantees 32 bytes at `ptr`.
        let ops = unsafe { _mm256_loadu_si256(ptr.cast()) };
        let jumpdests =
            _mm256_movemask_epi8(_mm256_cmpeq_epi8(ops, _mm256_set1_epi8(JUMPDEST))) as u32;
        if _mm256_movemask_epi8(_mm256_cmpgt_epi8(ops, _mm256_set1_epi8(PUSHX))) == 0 {
            return unsafe { uncarried(jumpdests.into(), entry) } as u32;
        }

        let n0 = _mm256_add_epi8(
            _mm256_max_epi8(ops, _mm256_set1_epi8(PUSHX)),
            END_OFFSETS16X2,
        );
        let n1 = compose16x2(n0);
        let n2 = compose16x2(n1);
        let n3 = compose16x2(n2);
        let exits = compose16x2(n3);
        let entry_lo = *entry;
        let entry_hi = next_entry16(_mm256_castsi256_si128(exits), entry_lo);
        *entry = next_entry16(_mm256_extracti128_si256::<1>(exits), entry_hi);
        if jumpdests == 0 {
            return 0;
        }

        let entries = _mm256_set_m128i(entry_hi, entry_lo);
        let mut position = descend16x2(n3, entries);
        position = descend16x2(n2, position);
        position = descend16x2(n1, position);
        // One step at most remains, so a lane starts an instruction if it is the position or the
        // position's successor.
        let starts = _mm256_or_si256(
            _mm256_cmpeq_epi8(position, LANES16X2),
            _mm256_cmpeq_epi8(_mm256_shuffle_epi8(n0, position), LANES16X2),
        );
        // No lane of a half starts an instruction if its entry is past the half.
        let past = _mm256_movemask_epi8(entries) as u32;
        jumpdests & _mm256_movemask_epi8(starts) as u32 & !past
    }
}

/// Squares the successor map of each 16-byte half, keeping offsets past the half.
#[inline]
#[target_feature(enable = "avx2")]
fn compose16x2(successors: __m256i) -> __m256i {
    _mm256_max_epu8(_mm256_shuffle_epi8(successors, successors), successors)
}

/// Moves each lane's position along the successor map if the result does not pass the lane.
#[inline]
#[target_feature(enable = "avx2")]
fn descend16x2(successors: __m256i, positions: __m256i) -> __m256i {
    let next = _mm256_shuffle_epi8(successors, positions);
    let stays = _mm256_cmpeq_epi8(_mm256_min_epu8(next, LANES16X2), next);
    _mm256_blendv_epi8(positions, next, stays)
}

/// 64-byte blocks with AVX-512 VBMI.
struct Avx512Vbmi;

impl Kernel for Avx512Vbmi {
    type Bits = u64;

    type Entry = __m512i;

    #[inline]
    #[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
    unsafe fn block(ptr: *const u8, entry: &mut __m512i) -> u64 {
        // SAFETY: the caller guarantees 64 bytes at `ptr`.
        let ops = unsafe { _mm512_loadu_si512(ptr.cast()) };
        let n = successors64(ops);
        let block_entry = *entry;
        *entry = next_entry64(&n, block_entry);
        _mm512_cmpeq_epi8_mask(ops, _mm512_set1_epi8(JUMPDEST)) & starts64(&n, block_entry)
    }
}

impl Entry for __m512i {
    #[inline]
    #[target_feature(enable = "avx512f")]
    unsafe fn new(offset: usize) -> Self {
        _mm512_set1_epi8(offset as i8)
    }

    #[inline]
    #[target_feature(enable = "avx512f")]
    unsafe fn offset(self) -> usize {
        _mm512_cvtsi512_si32(self) as u8 as usize
    }
}

/// Returns the successor maps of `ops` for 1, 2, 4, ..., 64 instructions.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
fn successors64(ops: __m512i) -> [__m512i; 7] {
    let mut n = [_mm512_add_epi8(_mm512_max_epi8(ops, _mm512_set1_epi8(PUSHX)), END_OFFSETS64); 7];
    for i in 1..7 {
        n[i] = _mm512_permutex2var_epi8(n[i - 1], n[i - 1], EXITS64);
    }
    n
}

/// Returns the lanes of the block with successor maps `n` that start an instruction.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
fn starts64(n: &[__m512i; 7], entry: __m512i) -> u64 {
    let mut position = entry;
    for i in (1..6).rev() {
        position = descend64(n[i], position);
    }
    // One step at most remains, so a lane starts an instruction if it is the position or the
    // position's successor.
    _mm512_cmpeq_epi8_mask(position, LANES64)
        | _mm512_cmpeq_epi8_mask(_mm512_permutexvar_epi8(position, n[0]), LANES64)
}

/// Returns the entry of the next block, given the entry of the block with successor maps `n`.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
fn next_entry64(n: &[__m512i; 7], entry: __m512i) -> __m512i {
    _mm512_sub_epi8(_mm512_permutexvar_epi8(entry, n[6]), _mm512_set1_epi8(64))
}

/// Pairs of 64-byte blocks with AVX-512 VBMI, skipping work for pairs without PUSH or JUMPDEST
/// opcodes.
struct Avx512Vbmix2;

impl Kernel for Avx512Vbmix2 {
    type Bits = u128;

    type Entry = __m512i;

    #[inline]
    #[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
    unsafe fn block(ptr: *const u8, entry: &mut __m512i) -> u128 {
        // SAFETY: the caller guarantees 128 bytes at `ptr`.
        let (ops_a, ops_b) = unsafe {
            (
                _mm512_loadu_si512(ptr.cast()),
                _mm512_loadu_si512(ptr.add(64).cast()),
            )
        };
        let jumpdests_a = _mm512_cmpeq_epi8_mask(ops_a, _mm512_set1_epi8(JUMPDEST));
        let jumpdests_b = _mm512_cmpeq_epi8_mask(ops_b, _mm512_set1_epi8(JUMPDEST));
        if _mm512_cmpgt_epi8_mask(_mm512_max_epi8(ops_a, ops_b), _mm512_set1_epi8(PUSHX)) == 0 {
            let jumpdests_a = unsafe { uncarried(jumpdests_a, entry) };
            return jumpdests_a as u128 | (jumpdests_b as u128) << 64;
        }

        let a = successors64(ops_a);
        let b = successors64(ops_b);
        let entry_a = *entry;
        let entry_b = next_entry64(&a, entry_a);
        *entry = next_entry64(&b, entry_b);
        if (jumpdests_a | jumpdests_b) == 0 {
            return 0;
        }
        let starts_a = jumpdests_a & starts64(&a, entry_a);
        let starts_b = jumpdests_b & starts64(&b, entry_b);
        starts_a as u128 | (starts_b as u128) << 64
    }
}

/// Moves each lane's position along the successor map if the result does not pass the lane.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
fn descend64(successors: __m512i, positions: __m512i) -> __m512i {
    let next = _mm512_permutexvar_epi8(positions, successors);
    _mm512_mask_mov_epi8(positions, _mm512_cmple_epu8_mask(next, LANES64), next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy::analysis::tests::check_simd;

    #[test]
    fn test_sse41() {
        if detected!("sse4.1") {
            check_simd(|code, table| unsafe { analyze_sse41(code, table) });
        }
    }

    #[test]
    fn test_avx2() {
        if detected!("avx2") {
            check_simd(|code, table| unsafe { analyze_avx2(code, table) });
        }
    }

    #[test]
    fn test_avx512vbmi() {
        if detected!("avx512f", "avx512bw", "avx512vbmi") {
            check_simd(|code, table| unsafe { analyze_avx512vbmi(code, table) });
        }
    }
}
