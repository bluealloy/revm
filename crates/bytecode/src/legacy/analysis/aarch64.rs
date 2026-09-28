//! Jump destination analysis with NEON.
//!
//! Uses the same algorithm as the x86 16-byte kernels. `tbx` leaves a lane unchanged when its
//! index is out of range, so offsets past the block need no bias.

use super::{analyze_blocks, lanes, uncarried, Entry, Kernel, END_OFFSET, PUSHX};
use crate::opcode;
use core::arch::aarch64::*;

// SAFETY: the vector types are plain bytes.
const LANES: uint8x16_t = unsafe { core::mem::transmute(lanes::<16>(16, 0)) };
const END_OFFSETS: uint8x16_t =
    unsafe { core::mem::transmute(lanes::<16>(16, 0u8.wrapping_sub(END_OFFSET))) };
const WEIGHTS: uint8x16_t = unsafe {
    core::mem::transmute::<[u8; 16], _>([1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128])
};

pub(super) fn analyze(code: &[u8], table: &mut [u8]) -> usize {
    // SAFETY: NEON is always available on aarch64.
    unsafe { analyze_neon(code, table) }
}

#[target_feature(enable = "neon")]
unsafe fn analyze_neon(code: &[u8], table: &mut [u8]) -> usize {
    let state = unsafe { analyze_blocks::<Neonx4>(code, table, (0, 0)) };
    let (pc, entry) = unsafe { analyze_blocks::<Neon>(code, table, state) };
    pc + entry
}

impl Entry for uint8x16_t {
    #[inline]
    #[target_feature(enable = "neon")]
    unsafe fn new(offset: usize) -> Self {
        vdupq_n_u8(offset as u8)
    }

    #[inline]
    #[target_feature(enable = "neon")]
    unsafe fn offset(self) -> usize {
        vgetq_lane_u8::<0>(self) as usize
    }
}

/// Looks up each position in the successor map, keeping positions past the block.
#[inline]
#[target_feature(enable = "neon")]
fn advance16(successors: uint8x16_t, positions: uint8x16_t) -> uint8x16_t {
    vqtbx1q_u8(positions, successors, positions)
}

/// Returns the successor maps of `ops` for 1, 2, 4, 8 and 16 instructions.
#[inline]
#[target_feature(enable = "neon")]
fn successors16(ops: uint8x16_t) -> [uint8x16_t; 5] {
    let n0 = vaddq_u8(
        vreinterpretq_u8_s8(vmaxq_s8(vreinterpretq_s8_u8(ops), vdupq_n_s8(PUSHX))),
        END_OFFSETS,
    );
    let n1 = advance16(n0, n0);
    let n2 = advance16(n1, n1);
    let n3 = advance16(n2, n2);
    [n0, n1, n2, n3, advance16(n3, n3)]
}

/// Returns the entry into the block after the one with successor maps `n`, given the entry into
/// it.
#[inline]
#[target_feature(enable = "neon")]
fn next_entry16(n: &[uint8x16_t; 5], entry: uint8x16_t) -> uint8x16_t {
    vsubq_u8(advance16(n[4], entry), vdupq_n_u8(16))
}

/// Moves each lane's position along the successor map if the result does not pass the lane.
#[inline]
#[target_feature(enable = "neon")]
fn descend16(successors: uint8x16_t, positions: uint8x16_t) -> uint8x16_t {
    let next = advance16(successors, positions);
    vbslq_u8(vcleq_u8(next, LANES), next, positions)
}

/// Returns a mask of the lanes that start an instruction when the block is entered at `entry`.
#[inline]
#[target_feature(enable = "neon")]
fn starts16(n: &[uint8x16_t; 5], entry: uint8x16_t) -> uint8x16_t {
    let mut position = entry;
    for i in (0..4).rev() {
        position = descend16(n[i], position);
    }
    vceqq_u8(position, LANES)
}

/// Returns the bits of four lane masks, with lane 0 of `s[0]` in bit 0.
///
/// Masking each lane with its bit value and summing groups of eight lanes gives the bitmap bytes;
/// the bits are disjoint, so the sums are ORs.
#[inline]
#[target_feature(enable = "neon")]
fn to_bitmap(s: [uint8x16_t; 4]) -> u64 {
    let s = s.map(|s| vandq_u8(s, WEIGHTS));
    let quads = vpaddq_u8(vpaddq_u8(s[0], s[1]), vpaddq_u8(s[2], s[3]));
    vgetq_lane_u64::<0>(vreinterpretq_u64_u8(vpaddq_u8(quads, quads)))
}

/// 16-byte blocks with NEON.
struct Neon;

impl Kernel for Neon {
    type Bits = u16;

    type Entry = uint8x16_t;

    #[inline]
    #[target_feature(enable = "neon")]
    unsafe fn block(ptr: *const u8, entry: &mut uint8x16_t) -> u16 {
        // SAFETY: the caller guarantees 16 bytes at `ptr`.
        let ops = unsafe { vld1q_u8(ptr) };
        let n = successors16(ops);
        let starts = vandq_u8(
            starts16(&n, *entry),
            vceqq_u8(ops, vdupq_n_u8(opcode::JUMPDEST)),
        );
        *entry = next_entry16(&n, *entry);
        let zero = vdupq_n_u8(0);
        to_bitmap([starts, zero, zero, zero]) as u16
    }
}

/// 64-byte blocks of four 16-byte blocks with NEON, skipping work for blocks without PUSH or
/// JUMPDEST opcodes.
struct Neonx4;

impl Kernel for Neonx4 {
    type Bits = u64;

    type Entry = uint8x16_t;

    #[inline]
    #[target_feature(enable = "neon")]
    unsafe fn block(ptr: *const u8, entry: &mut uint8x16_t) -> u64 {
        // SAFETY: the caller guarantees 64 bytes at `ptr`.
        let ops = unsafe { core::array::from_fn::<_, 4, _>(|i| vld1q_u8(ptr.add(i * 16))) };
        let jumpdests = ops.map(|ops| vceqq_u8(ops, vdupq_n_u8(opcode::JUMPDEST)));
        let any_jumpdest = vorrq_u8(
            vorrq_u8(jumpdests[0], jumpdests[1]),
            vorrq_u8(jumpdests[2], jumpdests[3]),
        );
        let has_jumpdest = vmaxvq_u8(any_jumpdest) != 0;
        let ops_s = ops.map(|ops| vreinterpretq_s8_u8(ops));
        let max = vmaxq_s8(vmaxq_s8(ops_s[0], ops_s[1]), vmaxq_s8(ops_s[2], ops_s[3]));
        if vmaxvq_s8(max) <= PUSHX {
            return unsafe { uncarried(to_bitmap(jumpdests), entry) };
        }

        let n = ops.map(|ops| successors16(ops));
        let mut entries = [*entry; 4];
        for i in 1..4 {
            entries[i] = next_entry16(&n[i - 1], entries[i - 1]);
        }
        *entry = next_entry16(&n[3], entries[3]);
        if !has_jumpdest {
            return 0;
        }
        let starts = core::array::from_fn(|i| vandq_u8(starts16(&n[i], entries[i]), jumpdests[i]));
        to_bitmap(starts)
    }
}
