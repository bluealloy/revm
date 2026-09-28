use super::JumpTable;
use crate::opcode;
use bitvec::{bitvec, order::Lsb0, vec::BitVec};
use primitives::Bytes;
use std::vec::Vec;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

/// Analyzes the original bytecode to produce a jump table.
#[inline]
pub(crate) fn analyze_legacy(bytecode: &[u8]) -> JumpTable {
    let mut jumps: BitVec<u8> = bitvec![u8, Lsb0; 0; bytecode.len()];
    let table = jumps.as_raw_mut_slice();
    let pc = analyze_simd(bytecode, table);
    analyze_scalar(bytecode, table, pc);
    JumpTable::new(jumps)
}

/// Marks the jump destinations of `code` from the instruction at `pc` on.
///
/// `pc` must be zero or the first instruction after an already analyzed prefix.
#[inline]
fn analyze_scalar(code: &[u8], table: &mut [u8], pc: usize) {
    let range = code.as_ptr_range();
    let start = range.start;
    // A PUSH from the prefix can run past the end, so `wrapping_add` keeps this defined.
    let mut iterator = start.wrapping_add(pc);
    let end = range.end;

    while iterator < end {
        let last_byte = unsafe { *iterator };
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
}

/// Marks the jump destinations of complete SIMD blocks and returns the offset of the first
/// instruction after them.
#[inline]
fn analyze_simd(code: &[u8], table: &mut [u8]) -> usize {
    if code.len() < 16 {
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

/// Analyzes complete blocks with `K` from the block at `pc`, entered at `entry`, and returns
/// the next block and its entry.
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
    while pc + K::LEN <= code.len() {
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

/// Maximum PUSH immediate length plus a terminating STOP.
const PADDING: usize = 33;

/// Appends zero padding unless the bytecode already ends with 33 zeros.
pub(crate) fn pad_legacy(bytecode: Bytes) -> Bytes {
    if bytecode.is_empty() {
        return Bytes::from_static(&[opcode::STOP]);
    }
    if bytecode.ends_with(&[0; PADDING]) {
        return bytecode;
    }

    let padded_len = bytecode.len() + PADDING;
    match bytecode.0.try_into_mut() {
        Ok(mut bytecode) => {
            bytecode.resize(padded_len, 0);
            bytecode.freeze().into()
        }
        Err(bytecode) => {
            let mut padded = Vec::with_capacity(padded_len);
            padded.extend_from_slice(&bytecode);
            padded.resize(padded_len, 0);
            padded.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcode::OpCode;
    use rand::{rngs::StdRng, RngExt, SeedableRng};
    use std::vec;

    /// Returns the jump table of `code`, one instruction at a time.
    fn reference(code: &[u8]) -> Vec<u8> {
        let mut table = vec![0; code.len().div_ceil(8)];
        let mut pc = 0;
        while pc < code.len() {
            let last = code[pc];
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
        table
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
            analyze_scalar(code, &mut table, pc);
            assert_eq!(table, reference(code), "{}", primitives::hex::encode(code));
        };

        // Every PUSH near the end, followed by different instruction endings.
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
    fn test_bytecode_ends_with_stop_still_padded() {
        let bytecode = vec![
            opcode::PUSH1,
            0x01,
            opcode::PUSH1,
            0x02,
            opcode::ADD,
            opcode::STOP,
        ];
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
    }

    #[test]
    fn test_bytecode_ends_without_stop_requires_padding() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH1, 0x02, opcode::ADD];
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
    }

    #[test]
    fn test_bytecode_ends_with_push16() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH16];
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
    }

    #[test]
    fn test_bytecode_ends_with_push2() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH2, 0x02];
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
    }

    #[test]
    fn test_bytecode_with_jumpdest_at_start() {
        let bytecode = vec![opcode::JUMPDEST, opcode::PUSH1, 0x01, opcode::STOP];
        let jump_table = analyze_legacy(&bytecode);
        assert!(jump_table.is_valid(0)); // First byte should be a valid jumpdest
    }

    #[test]
    fn test_bytecode_with_jumpdest_after_push() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::JUMPDEST, opcode::STOP];
        let jump_table = analyze_legacy(&bytecode);
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
        let jump_table = analyze_legacy(&bytecode);
        assert!(jump_table.is_valid(0)); // First JUMPDEST
        assert!(jump_table.is_valid(3)); // Second JUMPDEST
    }

    #[test]
    fn test_bytecode_with_max_push32() {
        let bytecode = vec![opcode::PUSH32];
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33); // PUSH32 + 32 bytes + STOP
    }

    #[test]
    fn test_truncated_pushes_are_padded_without_inbounds_pointer_advance() {
        for push in opcode::PUSH1..=opcode::PUSH32 {
            let bytecode = vec![push];
            let padded_bytecode = pad_legacy(bytecode.clone().into());
            let push_immediate_len = (push - opcode::PUSH1 + 1) as usize;
            assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
            assert!(padded_bytecode.len() > bytecode.len() + push_immediate_len);
        }
    }

    #[test]
    fn test_bytecode_with_invalid_opcode() {
        let bytecode = vec![0xFF, opcode::STOP]; // 0xFF is an invalid opcode
        let jump_table = analyze_legacy(&bytecode);
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
        let jump_table = analyze_legacy(&bytecode);
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
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
        let jump_table = analyze_legacy(&bytecode);
        assert!(!jump_table.is_valid(1)); // JUMPDEST in push data should not be valid
    }

    #[test]
    fn test_bytecode_ends_with_immediate_opcode_and_stop_requires_padding() {
        // For SWAPN/DUPN/EXCHANGE, the STOP (0x00) is consumed as the immediate operand,
        // not as an actual STOP instruction, so padding is needed.
        // The fixed padding supplies both the immediate and a terminating STOP.
        for op in [opcode::SWAPN, opcode::DUPN, opcode::EXCHANGE] {
            for bytecode in [vec![op], vec![op, opcode::STOP]] {
                let original_len = bytecode.len();
                let padded_bytecode = pad_legacy(bytecode.into());
                assert_eq!(padded_bytecode.len(), original_len + 33);
                assert_eq!(padded_bytecode[0], op);
                assert_eq!(padded_bytecode[1], opcode::STOP);
                assert_eq!(padded_bytecode[2], opcode::STOP);
            }
        }
    }

    #[test]
    fn padding_zero_suffix_boundary() {
        for len in [1, 32, 33, 34, 65] {
            let raw = Bytes::from(vec![0; len]);
            let padded = pad_legacy(raw.clone());
            if len >= 33 {
                assert_eq!(padded.len(), len);
                assert_eq!(padded.as_ptr(), raw.as_ptr());
            } else {
                assert_eq!(padded.len(), len + 33);
            }
            assert!(padded.iter().all(|&byte| byte == 0));
        }

        // Every byte of the suffix must be zero to skip padding.
        for nonzero in 0..33 {
            let mut raw = vec![0; 33];
            raw[nonzero] = opcode::JUMPDEST;
            let padded = pad_legacy(raw.clone().into());
            assert_eq!(padded.len(), 66);
            assert_eq!(&padded[..33], &raw);
            assert_eq!(&padded[33..], &[0; 33]);
        }
    }

    #[test]
    fn padding_reuses_owned_capacity() {
        let mut raw = Vec::with_capacity(34);
        raw.push(opcode::PUSH32);
        let raw = Bytes::from(raw);
        let ptr = raw.as_ptr();
        let padded = pad_legacy(raw);
        assert_eq!(padded.as_ptr(), ptr);
        assert_eq!(padded.len(), 34);
        assert_eq!(&padded[1..], &[0; 33]);
    }

    #[test]
    fn padding_releases_shared_input() {
        let raw = Bytes::copy_from_slice(&[opcode::PUSH32]);
        let padded = pad_legacy(raw.clone());
        assert!(raw.is_unique());
        assert_ne!(padded.as_ptr(), raw.as_ptr());
        assert_eq!(&raw[..], &[opcode::PUSH32]);
        assert_eq!(padded.len(), 34);
        assert_eq!(&padded[1..], &[0; 33]);
    }
}
