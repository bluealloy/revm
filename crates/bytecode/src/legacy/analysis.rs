use super::JumpTable;
use crate::opcode;
use bitvec::{bitvec, order::Lsb0, vec::BitVec};
use primitives::Bytes;
use std::vec::Vec;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

/// Number of trailing bytes the SIMD prefix leaves to the scalar loop.
///
/// When the prefix ends on an instruction boundary, the scalar loop then sees either two
/// instructions or one PUSH, so it knows the last two instructions whenever the padding depends on
/// both. Otherwise, the instruction before the scalar loop is a PUSH, which may also run past the
/// end.
const SCALAR_TAIL: usize = 2;

/// Analyzes the bytecode to produce a jump table and potentially padded bytecode.
///
/// Prefer using [`Bytecode::new_legacy`](crate::Bytecode::new_legacy) instead.
#[inline]
pub(crate) fn analyze_legacy(bytecode: Bytes) -> (JumpTable, Bytes) {
    let mut jumps: BitVec<u8> = bitvec![u8, Lsb0; 0; bytecode.len()];
    let table = jumps.as_raw_mut_slice();
    let pc = analyze_simd(&bytecode, table);
    let padding = analyze_scalar(&bytecode, table, pc);

    let bytecode = if padding > 0 {
        let mut padded = Vec::with_capacity(bytecode.len() + padding);
        padded.extend_from_slice(&bytecode);
        padded.resize(padded.len() + padding, 0);
        Bytes::from(padded)
    } else {
        bytecode
    };

    (JumpTable::new(jumps), bytecode)
}

/// Marks the jump destinations of `code` from the instruction at `pc` on, and returns the padding
/// the bytecode needs.
///
/// `pc` must be zero, or the first instruction after a prefix that ends at least [`SCALAR_TAIL`]
/// bytes before the end of `code`.
#[inline]
fn analyze_scalar(code: &[u8], table: &mut [u8], pc: usize) -> usize {
    let range = code.as_ptr_range();
    let start = range.start;
    // A PUSH from the prefix can run past the end, so `wrapping_add` keeps this defined.
    let mut iterator = start.wrapping_add(pc);
    let end = range.end;
    let mut prev_byte: u8 = 0;
    // Stands in for the instruction before a nonzero `pc`. It is a PUSH, or the loop below sees
    // two more instructions or a PUSH, so the padding never depends on its opcode otherwise.
    let mut last_byte = if pc == 0 { opcode::STOP } else { opcode::PUSH1 };

    while iterator < end {
        prev_byte = last_byte;
        last_byte = unsafe { *iterator };
        if last_byte == opcode::JUMPDEST {
            // SAFETY: Jumps are max length of the code.
            let offset = unsafe { iterator.offset_from_unsigned(start) };
            // SAFETY: `table` has a bit for each byte of `code`.
            unsafe { *table.get_unchecked_mut(offset / 8) |= 1 << (offset % 8) };
            iterator = unsafe { iterator.add(1) };
        } else {
            let push_offset = last_byte.wrapping_sub(opcode::PUSH1);
            if push_offset < 32 {
                // A trailing PUSH can advance past the bytecode allocation.
                // `wrapping_add` keeps that offset computation defined.
                iterator = iterator.wrapping_add(push_offset as usize + 2);
            } else {
                // SAFETY: Iterator access range is checked in the while loop.
                iterator = unsafe { iterator.add(1) };
            }
        }
    }

    // Calculate padding needed:
    // push_overflow: bytes needed for incomplete PUSH immediate data
    let push_overflow = (iterator as usize) - (end as usize);
    let mut padding = push_overflow;

    if last_byte == opcode::STOP {
        // DUPN/SWAPN/EXCHANGE have 1-byte immediates that aren't handled by the loop above,
        // so we need extra padding to ensure safe execution.
        padding += is_dupn_swapn_exchange(prev_byte) as usize;
    } else {
        // Add final STOP instruction and immediate for DUPN/SWAPN/EXCHANGE
        padding += 1 + is_dupn_swapn_exchange(last_byte) as usize;
    }

    padding
}

/// Marks the jump destinations of a bytecode prefix that ends at least [`SCALAR_TAIL`] bytes
/// before the end, and returns the offset of the first instruction after it.
#[inline]
fn analyze_simd(code: &[u8], table: &mut [u8]) -> usize {
    if code.len() < 16 + SCALAR_TAIL {
        return 0;
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        x86::analyze(code, table)
    }
    #[cfg(target_arch = "aarch64")]
    {
        aarch64::analyze(code, table)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
    {
        let _ = table;
        0
    }
}

/// A SIMD kernel that analyzes blocks of [`Self::LEN`] bytes.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
trait Kernel {
    /// Jump destination bits of a block, one per byte.
    type Bits: Into<u128>;

    /// Offset of the first instruction in a block.
    type Entry: Entry;

    /// Block length in bytes.
    const LEN: usize = 8 * core::mem::size_of::<Self::Bits>();

    /// Returns the JUMPDEST bits of the block at `ptr` that start an instruction, and moves
    /// `entry` to the next block.
    unsafe fn block(ptr: *const u8, entry: &mut Self::Entry) -> Self::Bits;
}

/// Analyzes blocks with `K` from the block at `pc`, entered at `entry`, while they end at
/// least [`SCALAR_TAIL`] bytes before the end of `code`, and returns the next block and its entry.
///
/// # Safety
///
/// The CPU must support the kernel's target features, and `table` must have a bit for each byte
/// of `code`.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn analyze_blocks<K: Kernel>(
    code: &[u8],
    table: &mut [u8],
    (mut pc, entry): (usize, usize),
) -> (usize, usize) {
    let mut entry = unsafe { K::Entry::new(entry) };
    while pc + K::LEN + SCALAR_TAIL <= code.len() {
        // SAFETY: the block and its bits are in bounds.
        unsafe {
            let bits = K::block(code.as_ptr().add(pc), &mut entry)
                .into()
                .to_le_bytes();
            let dst = table.as_mut_ptr().add(pc / 8);
            core::ptr::copy_nonoverlapping(bits.as_ptr(), dst, K::LEN / 8);
        }
        pc += K::LEN;
    }
    (pc, unsafe { entry.offset() })
}

/// A kernel's representation of the offset of the first instruction in a block.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
trait Entry: Copy {
    /// Converts an offset, at most 32, to an entry.
    unsafe fn new(offset: usize) -> Self;

    /// Converts the entry to an offset.
    unsafe fn offset(self) -> usize;
}

/// Every opcode below this, including `0x80..` as signed bytes, is one byte long.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
const PUSHX: i8 = opcode::PUSH1 as i8 - 1;

/// Subtracted from `max(opcode, PUSH1 - 1)` to get the length of an instruction.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
const END_OFFSET: u8 = opcode::PUSH1 - 2;

/// Returns lanes counting up from `start`, restarting every `period` lanes.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
const fn lanes<const N: usize>(period: usize, start: u8) -> [u8; N] {
    let mut lanes = [0; N];
    let mut i = 0;
    while i < N {
        lanes[i] = start.wrapping_add((i % period) as u8);
        i += 1;
    }
    lanes
}

/// Returns the JUMPDEST bits of a block without PUSH opcodes, clearing those the previous block's
/// PUSH covers, and moves `entry` to the next block.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn uncarried<E: Entry>(jumpdests: u64, entry: &mut E) -> u64 {
    let carried = unsafe { entry.offset() };
    *entry = unsafe { E::new(0) };
    // At most 32, so the shifts never overflow.
    jumpdests >> carried << carried
}

/// Returns true if the opcode is DUPN, SWAPN, or EXCHANGE.
const fn is_dupn_swapn_exchange(opcode: u8) -> bool {
    opcode.wrapping_sub(opcode::DUPN) < 3
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcode::OpCode;
    use rand::{rngs::StdRng, RngExt, SeedableRng};
    use std::vec;

    /// Returns the jump table and padding of `code`, one instruction at a time.
    fn reference(code: &[u8]) -> (Vec<u8>, usize) {
        let mut table = vec![0; code.len().div_ceil(8)];
        let (mut pc, mut prev, mut last) = (0, 0, 0);
        while pc < code.len() {
            (prev, last) = (last, code[pc]);
            if last == opcode::JUMPDEST {
                table[pc / 8] |= 1 << (pc % 8);
            }
            let opcode = OpCode::new_or_unknown(last);
            pc += 1 + if opcode.is_push() {
                opcode.info().immediate_size() as usize
            } else {
                0
            };
        }
        let padding = pc - code.len()
            + if last == opcode::STOP {
                is_dupn_swapn_exchange(prev) as usize
            } else {
                1 + is_dupn_swapn_exchange(last) as usize
            };
        (table, padding)
    }

    /// Returns random bytecode of `len` bytes, drawn from one of several opcode mixes.
    fn random_code(rng: &mut StdRng, len: usize) -> Vec<u8> {
        const DENSE: &[u8] = &[
            opcode::PUSH1,
            opcode::PUSH1,
            opcode::PUSH2,
            opcode::PUSH4,
            opcode::PUSH32,
            opcode::JUMPDEST,
            opcode::JUMPDEST,
            opcode::STOP,
            opcode::ADD,
            opcode::DUPN,
            0x80,
            0xff,
        ];
        let kind = rng.random_range(0..6);
        let mut code = Vec::with_capacity(len);
        while code.len() < len {
            let byte = match kind {
                0 => rng.random(),
                1 => DENSE[rng.random_range(0..DENSE.len())],
                // No PUSH opcodes.
                2 => match rng.random() {
                    opcode::PUSH1..=opcode::PUSH32 => opcode::JUMPDEST,
                    byte => byte,
                },
                3 => [opcode::PUSH1, opcode::JUMPDEST, opcode::ADD][rng.random_range(0..3)],
                // Rare PUSH opcodes.
                4 => match rng.random_range(0..64) {
                    0 => rng.random_range(opcode::PUSH1..=opcode::PUSH32),
                    1..8 => opcode::JUMPDEST,
                    _ => opcode::ADD,
                },
                // Instructions with random immediates.
                _ => {
                    let byte = rng.random();
                    code.push(byte);
                    if (opcode::PUSH1..=opcode::PUSH32).contains(&byte) {
                        for _ in 0..=byte - opcode::PUSH1 {
                            code.push(rng.random());
                        }
                    }
                    continue;
                }
            };
            code.push(byte);
        }
        code.truncate(len);
        code
    }

    /// Checks that `simd`, followed by the scalar loop, matches [`reference`] on bytecode ending
    /// in PUSH instructions, and on random bytecode.
    pub(super) fn check_simd(simd: impl Fn(&[u8], &mut [u8]) -> usize) {
        let check = |code: &[u8]| {
            let mut table = vec![0; code.len().div_ceil(8)];
            let pc = simd(code, &mut table);
            let padding = analyze_scalar(code, &mut table, pc);
            assert_eq!(
                (table, padding),
                reference(code),
                "{}",
                primitives::hex::encode(code)
            );
        };

        // Every PUSH near the end, followed by the endings the padding depends on.
        let endings: [&[u8]; 4] = [
            &[],
            &[opcode::STOP],
            &[opcode::DUPN],
            &[opcode::DUPN, opcode::STOP],
        ];
        for len in 16usize..160 {
            for at in len.saturating_sub(40)..len {
                for push in opcode::PUSH1..=opcode::PUSH32 {
                    for ending in endings.into_iter().filter(|ending| at + ending.len() < len) {
                        let mut code = vec![opcode::JUMPDEST; len];
                        code[at] = push;
                        code[len - ending.len()..].copy_from_slice(ending);
                        check(&code);
                    }
                }
            }
        }

        let mut rng = StdRng::seed_from_u64(0);
        for i in 0..600 {
            let len = if i < 400 {
                i
            } else {
                rng.random_range(400..5000)
            };
            for _ in 0..8 {
                check(&random_code(&mut rng, len));
            }
        }
    }

    #[test]
    fn test_simd_matches_reference() {
        check_simd(analyze_simd);
    }

    #[test]
    fn test_scalar_matches_reference() {
        check_simd(|_, _| 0);
    }

    #[test]
    fn test_bytecode_ends_with_stop_no_padding_needed() {
        let bytecode = vec![
            opcode::PUSH1,
            0x01,
            opcode::PUSH1,
            0x02,
            opcode::ADD,
            opcode::STOP,
        ];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len());
    }

    #[test]
    fn test_bytecode_ends_without_stop_requires_padding() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH1, 0x02, opcode::ADD];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 1);
    }

    #[test]
    fn test_bytecode_ends_with_push16_requires_17_bytes_padding() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH16];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 17);
    }

    #[test]
    fn test_bytecode_ends_with_push2_requires_2_bytes_padding() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH2, 0x02];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 2);
    }

    #[test]
    fn test_bytecode_with_jumpdest_at_start() {
        let bytecode = vec![opcode::JUMPDEST, opcode::PUSH1, 0x01, opcode::STOP];
        let (jump_table, _) = analyze_legacy(bytecode.into());
        assert!(jump_table.is_valid(0)); // First byte should be a valid jumpdest
    }

    #[test]
    fn test_bytecode_with_jumpdest_after_push() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::JUMPDEST, opcode::STOP];
        let (jump_table, _) = analyze_legacy(bytecode.into());
        assert!(jump_table.is_valid(2)); // JUMPDEST should be at position 2
    }

    #[test]
    fn test_bytecode_with_multiple_jumpdests() {
        let bytecode = vec![
            opcode::JUMPDEST,
            opcode::PUSH1,
            0x01,
            opcode::JUMPDEST,
            opcode::STOP,
        ];
        let (jump_table, _) = analyze_legacy(bytecode.into());
        assert!(jump_table.is_valid(0)); // First JUMPDEST
        assert!(jump_table.is_valid(3)); // Second JUMPDEST
    }

    #[test]
    fn test_bytecode_with_max_push32() {
        let bytecode = vec![opcode::PUSH32];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33); // PUSH32 + 32 bytes + STOP
    }

    #[test]
    fn test_truncated_pushes_are_padded_without_inbounds_pointer_advance() {
        for push in opcode::PUSH1..=opcode::PUSH32 {
            let bytecode = vec![push];
            let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
            let push_immediate_len = (push - opcode::PUSH1 + 1) as usize;
            assert_eq!(
                padded_bytecode.len(),
                bytecode.len() + push_immediate_len + 1
            );
        }
    }

    #[test]
    fn test_bytecode_with_invalid_opcode() {
        let bytecode = vec![0xFF, opcode::STOP]; // 0xFF is an invalid opcode
        let (jump_table, _) = analyze_legacy(bytecode.into());
        assert!(!jump_table.is_valid(0)); // Invalid opcode should not be a jumpdest
    }

    #[test]
    fn test_bytecode_with_sequential_pushes() {
        let bytecode = vec![
            opcode::PUSH1,
            0x01,
            opcode::PUSH2,
            0x02,
            0x03,
            opcode::PUSH4,
            0x04,
            0x05,
            0x06,
            0x07,
            opcode::STOP,
        ];
        let (jump_table, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len());
        assert!(!jump_table.is_valid(0)); // PUSH1
        assert!(!jump_table.is_valid(2)); // PUSH2
        assert!(!jump_table.is_valid(5)); // PUSH4
    }

    #[test]
    fn test_bytecode_with_jumpdest_in_push_data() {
        let bytecode = vec![
            opcode::PUSH2,
            opcode::JUMPDEST, // This should not be treated as a JUMPDEST
            0x02,
            opcode::STOP,
        ];
        let (jump_table, _) = analyze_legacy(bytecode.into());
        assert!(!jump_table.is_valid(1)); // JUMPDEST in push data should not be valid
    }

    #[test]
    fn test_bytecode_ends_with_immediate_opcode_and_stop_requires_padding() {
        // For SWAPN/DUPN/EXCHANGE, the STOP (0x00) is consumed as the immediate operand,
        // not as an actual STOP instruction, so padding is needed.
        // [OPCODE]       -> [OPCODE, STOP, STOP] (3 bytes)
        // [OPCODE, STOP] -> [OPCODE, STOP, STOP] (3 bytes)
        for op in [opcode::SWAPN, opcode::DUPN, opcode::EXCHANGE] {
            for bytecode in [vec![op], vec![op, opcode::STOP]] {
                let (_, padded_bytecode) = analyze_legacy(bytecode.into());
                assert_eq!(padded_bytecode.len(), 3);
                assert_eq!(padded_bytecode[0], op);
                assert_eq!(padded_bytecode[1], opcode::STOP);
                assert_eq!(padded_bytecode[2], opcode::STOP);
            }
        }
    }
}
