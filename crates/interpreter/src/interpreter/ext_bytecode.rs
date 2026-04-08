use super::{Immediates, Jumps, LegacyBytecode};
use crate::{interpreter_types::LoopControl, InterpreterAction};
use bytecode::Bytecode;
use core::ops::Deref;
use primitives::B256;

#[cfg(feature = "serde")]
mod serde;

/// Extended bytecode structure that wraps base bytecode with additional execution metadata.
///
/// Internally tracks position as a `usize` program counter rather than a raw pointer,
/// ensuring all jump and read operations go through checked arithmetic before accessing
/// the backing byte buffer.
#[derive(Debug)]
pub struct ExtBytecode {
    /// Current program counter (byte offset into `base.bytes_slice()`).
    pc: usize,
    /// Whether the execution should continue.
    continue_execution: bool,
    /// Bytecode Keccak-256 hash.
    /// This is `None` if it hasn't been calculated yet.
    /// Since it's not necessary for execution, it's not calculated by default.
    bytecode_hash: Option<B256>,
    /// Actions that the EVM should do. It contains return value of the Interpreter or inputs for `CALL` or `CREATE` instructions.
    /// For `RETURN` or `REVERT` instructions it contains the result of the instruction.
    pub action: Option<InterpreterAction>,
    /// The base bytecode.
    base: Bytecode,
}

impl Deref for ExtBytecode {
    type Target = Bytecode;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Default for ExtBytecode {
    #[inline]
    fn default() -> Self {
        Self::new(Bytecode::default())
    }
}

impl ExtBytecode {
    /// Create new extended bytecode and set the instruction pointer to the start of the bytecode.
    ///
    /// The bytecode hash will not be calculated.
    #[inline]
    pub fn new(base: Bytecode) -> Self {
        Self::new_with_optional_hash(base, None)
    }

    /// Creates new `ExtBytecode` with the given hash.
    #[inline]
    pub fn new_with_hash(base: Bytecode, hash: B256) -> Self {
        Self::new_with_optional_hash(base, Some(hash))
    }

    /// Creates new `ExtBytecode` with the given hash.
    #[inline]
    pub fn new_with_optional_hash(base: Bytecode, hash: Option<B256>) -> Self {
        Self {
            pc: 0,
            base,
            bytecode_hash: hash,
            action: None,
            continue_execution: true,
        }
    }

    /// Re-calculates the bytecode hash.
    ///
    /// Prefer [`get_or_calculate_hash`](Self::get_or_calculate_hash) if you just need to get the hash.
    #[inline]
    pub fn calculate_hash(&mut self) -> B256 {
        let hash = self.base.hash_slow();
        self.bytecode_hash = Some(hash);
        hash
    }

    /// Returns the bytecode hash.
    #[inline]
    pub fn hash(&mut self) -> Option<B256> {
        self.bytecode_hash
    }

    /// Returns the bytecode hash or calculates it if it is not set.
    #[inline]
    pub fn get_or_calculate_hash(&mut self) -> B256 {
        *self.bytecode_hash.get_or_insert_with(
            #[cold]
            || self.base.hash_slow(),
        )
    }
}

impl LoopControl for ExtBytecode {
    #[inline]
    fn is_not_end(&self) -> bool {
        self.continue_execution
    }

    #[inline]
    fn reset_action(&mut self) {
        self.continue_execution = true;
    }

    #[inline]
    fn set_action(&mut self, action: InterpreterAction) {
        debug_assert_eq!(
            !self.continue_execution,
            self.action.is_some(),
            "has_set_action out of sync"
        );
        debug_assert!(
            self.continue_execution,
            "action already set;\nold: {:#?}\nnew: {:#?}",
            self.action, action,
        );
        self.continue_execution = false;
        self.action = Some(action);
    }

    #[inline]
    fn action(&mut self) -> &mut Option<InterpreterAction> {
        &mut self.action
    }
}

impl Jumps for ExtBytecode {
    #[inline]
    fn relative_jump(&mut self, offset: isize) {
        let new_pc = (self.pc as isize)
            .checked_add(offset)
            .expect("relative_jump overflow");
        assert!(
            new_pc >= 0 && (new_pc as usize) <= self.base.bytes_slice().len(),
            "relative_jump out of bounds: pc {} + offset {} = {}, bytecode len {}",
            self.pc,
            offset,
            new_pc,
            self.base.bytes_slice().len(),
        );
        self.pc = new_pc as usize;
    }

    #[inline]
    fn absolute_jump(&mut self, offset: usize) {
        assert!(
            offset <= self.base.bytes_slice().len(),
            "absolute_jump out of bounds: offset {}, bytecode len {}",
            offset,
            self.base.bytes_slice().len(),
        );
        self.pc = offset;
    }

    #[inline]
    fn is_valid_legacy_jump(&mut self, offset: usize) -> bool {
        match self.base.legacy_jump_table() {
            Some(jt) => jt.is_valid(offset),
            None => false,
        }
    }

    #[inline]
    fn opcode(&self) -> u8 {
        self.base.bytes_slice()[self.pc]
    }

    #[inline]
    fn pc(&self) -> usize {
        self.pc
    }
}

impl Immediates for ExtBytecode {
    #[inline]
    fn read_u16(&self) -> u16 {
        let bytes = self.base.bytes_slice();
        u16::from_be_bytes([bytes[self.pc], bytes[self.pc + 1]])
    }

    #[inline]
    fn read_u8(&self) -> u8 {
        self.base.bytes_slice()[self.pc]
    }

    #[inline]
    fn read_slice(&self, len: usize) -> &[u8] {
        &self.base.bytes_slice()[self.pc..self.pc + len]
    }

    #[inline]
    fn read_offset_u16(&self, offset: isize) -> u16 {
        let pos = (self.pc as isize + offset) as usize;
        let bytes = self.base.bytes_slice();
        u16::from_be_bytes([bytes[pos], bytes[pos + 1]])
    }
}

impl LegacyBytecode for ExtBytecode {
    fn bytecode_len(&self) -> usize {
        self.base.len()
    }

    fn bytecode_slice(&self) -> &[u8] {
        self.base.original_byte_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Bytes;

    #[test]
    fn test_with_hash_constructor() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[0x60, 0x00][..]));
        let hash = bytecode.hash_slow();
        let ext_bytecode = ExtBytecode::new_with_hash(bytecode.clone(), hash);
        assert_eq!(ext_bytecode.bytecode_hash, Some(hash));
    }

    #[test]
    fn test_valid_relative_jump() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[0x00, 0x01][..]));
        let mut ext = ExtBytecode::new(bytecode);
        assert_eq!(ext.pc(), 0);
        ext.relative_jump(1);
        assert_eq!(ext.pc(), 1);
    }

    #[test]
    fn test_valid_absolute_jump() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[0x00, 0x01][..]));
        let mut ext = ExtBytecode::new(bytecode);
        ext.absolute_jump(1);
        assert_eq!(ext.pc(), 1);
    }

    #[test]
    fn test_opcode_reads_correct_byte() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[0x60, 0x01, 0x00][..]));
        let mut ext = ExtBytecode::new(bytecode);
        assert_eq!(ext.opcode(), 0x60);
        ext.relative_jump(1);
        assert_eq!(ext.opcode(), 0x01);
    }

    #[test]
    fn test_read_u8_and_read_u16() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[0xAB, 0xCD, 0x00][..]));
        let ext = ExtBytecode::new(bytecode);
        assert_eq!(ext.read_u8(), 0xAB);
        assert_eq!(ext.read_u16(), 0xABCD);
    }

    #[test]
    fn test_read_slice() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[0x01, 0x02, 0x03, 0x00][..]));
        let ext = ExtBytecode::new(bytecode);
        assert_eq!(ext.read_slice(3), &[0x01, 0x02, 0x03]);
    }

    // Regression test for the original PoC from #3487.
    // Before the fix, this triggered undefined behavior via ptr::offset.
    // Now it panics with a bounds check.
    #[test]
    #[should_panic(expected = "relative_jump out of bounds")]
    fn test_relative_jump_negative_oob_panics() {
        let bytecode = Bytecode::new_raw(Bytes::from(vec![0x00, 0x01]));
        let mut ext = ExtBytecode::new(bytecode);
        ext.relative_jump(-1);
    }

    #[test]
    #[should_panic(expected = "relative_jump")]
    fn test_relative_jump_large_positive_oob_panics() {
        let bytecode = Bytecode::new_raw(Bytes::from(vec![0x00, 0x01]));
        let mut ext = ExtBytecode::new(bytecode);
        ext.relative_jump(isize::MAX);
    }

    #[test]
    #[should_panic(expected = "absolute_jump out of bounds")]
    fn test_absolute_jump_oob_panics() {
        let bytecode = Bytecode::new_raw(Bytes::from(vec![0x00, 0x01]));
        let mut ext = ExtBytecode::new(bytecode);
        ext.absolute_jump(usize::MAX);
    }

    #[test]
    #[should_panic]
    fn test_read_u16_oob_panics() {
        let bytecode = Bytecode::new_raw(Bytes::from(vec![0x00]));
        let mut ext = ExtBytecode::new(bytecode);
        // Jump to the last valid byte; reading u16 requires 2 bytes.
        let last = ext.base.bytes_slice().len() - 1;
        ext.absolute_jump(last);
        let _ = ext.read_u16();
    }

    #[test]
    #[should_panic]
    fn test_read_slice_oob_panics() {
        let bytecode = Bytecode::new_raw(Bytes::from(vec![0x00, 0x01]));
        let ext = ExtBytecode::new(bytecode);
        let _ = ext.read_slice(usize::MAX);
    }
}
