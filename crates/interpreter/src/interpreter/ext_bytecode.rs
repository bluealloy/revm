use super::{Immediates, Jumps, LegacyBytecode};
use crate::{interpreter_types::LoopControl, InterpreterAction};
use bytecode::{utils::read_u16, Bytecode};
use core::ops::Deref;
use primitives::B256;

#[cfg(feature = "serde")]
mod serde;

/// Extended bytecode structure that wraps base bytecode with additional execution metadata.
#[derive(Debug)]
pub struct ExtBytecode {
    /// The current instruction pointer.
    instruction_pointer: *const u8,
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
        let instruction_pointer = base.bytecode_ptr();
        Self {
            base,
            instruction_pointer,
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

impl ExtBytecode {
    /// Returns the bytecode bounds as `(base_ptr, end_ptr)` for use in debug assertions.
    #[inline]
    fn bytecode_bounds(&self) -> (*const u8, *const u8) {
        let bytes = self.base.bytes_ref();
        let base = bytes.as_ptr();
        (base, base.wrapping_add(bytes.len()))
    }

    /// Returns the current program counter without any bounds assertions,
    /// for use in debug assertion messages to avoid recursive panics.
    #[inline]
    fn pc_unchecked(&self) -> usize {
        // wrapping_sub avoids UB even if instruction_pointer is out of bounds.
        (self.instruction_pointer as usize).wrapping_sub(self.base.bytes_ref().as_ptr() as usize)
    }
}

impl Jumps for ExtBytecode {
    #[inline]
    fn relative_jump(&mut self, offset: isize) {
        let new_ptr = self.instruction_pointer.wrapping_offset(offset);
        let (base, end) = self.bytecode_bounds();
        debug_assert!(
            new_ptr >= base && new_ptr <= end,
            "relative_jump offset {offset} out of bounds (pc: {}, len: {})",
            self.pc_unchecked(),
            self.base.bytes_ref().len(),
        );
        self.instruction_pointer = unsafe { self.instruction_pointer.offset(offset) };
    }

    #[inline]
    fn absolute_jump(&mut self, offset: usize) {
        debug_assert!(
            offset <= self.base.bytes_ref().len(),
            "absolute_jump offset {offset} out of bounds (len: {})",
            self.base.bytes_ref().len(),
        );
        self.instruction_pointer = unsafe { self.base.bytes_ref().as_ptr().add(offset) };
    }

    #[inline]
    fn is_valid_legacy_jump(&mut self, offset: usize) -> bool {
        let jt = self.base.legacy_jump_table();
        debug_assert!(jt.is_some(), "is_valid_legacy_jump called on non-legacy bytecode");
        // SAFETY: Only called by legacy bytecode. Asserted above in debug mode.
        unsafe { jt.unwrap_unchecked() }.is_valid(offset)
    }

    #[inline]
    fn opcode(&self) -> u8 {
        let (base, end) = self.bytecode_bounds();
        debug_assert!(
            self.instruction_pointer >= base && self.instruction_pointer < end,
            "opcode: instruction_pointer out of bounds (pc: {}, len: {})",
            self.pc_unchecked(),
            self.base.bytes_ref().len(),
        );
        // SAFETY: Bounds checked in debug mode above.
        unsafe { *self.instruction_pointer }
    }

    #[inline]
    fn pc(&self) -> usize {
        let (base, end) = self.bytecode_bounds();
        debug_assert!(
            self.instruction_pointer >= base && self.instruction_pointer <= end,
            "pc: instruction_pointer out of bounds",
        );
        // SAFETY: `instruction_pointer` is at an offset from the start of the bytes.
        unsafe {
            self.instruction_pointer
                .offset_from_unsigned(self.base.bytes_ref().as_ptr())
        }
    }
}

impl Immediates for ExtBytecode {
    #[inline]
    fn read_u16(&self) -> u16 {
        debug_assert!(
            self.pc_unchecked() + 2 <= self.base.bytes_ref().len(),
            "read_u16: not enough bytes remaining (pc: {}, len: {})",
            self.pc_unchecked(),
            self.base.bytes_ref().len(),
        );
        unsafe { read_u16(self.instruction_pointer) }
    }

    #[inline]
    fn read_u8(&self) -> u8 {
        debug_assert!(
            self.pc_unchecked() < self.base.bytes_ref().len(),
            "read_u8: instruction_pointer out of bounds (pc: {}, len: {})",
            self.pc_unchecked(),
            self.base.bytes_ref().len(),
        );
        unsafe { *self.instruction_pointer }
    }

    #[inline]
    fn read_slice(&self, len: usize) -> &[u8] {
        debug_assert!(
            self.pc_unchecked() + len <= self.base.bytes_ref().len(),
            "read_slice: not enough bytes remaining (pc: {}, len: {}, bytecode_len: {})",
            self.pc_unchecked(),
            len,
            self.base.bytes_ref().len(),
        );
        unsafe { core::slice::from_raw_parts(self.instruction_pointer, len) }
    }

    #[inline]
    fn read_offset_u16(&self, offset: isize) -> u16 {
        debug_assert!(
            {
                let new_ptr = self.instruction_pointer.wrapping_offset(offset);
                let (base, end) = self.bytecode_bounds();
                new_ptr >= base && new_ptr.wrapping_add(2) <= end
            },
            "read_offset_u16: offset {offset} out of bounds (pc: {}, len: {})",
            self.pc_unchecked(),
            self.base.bytes_ref().len(),
        );
        unsafe {
            read_u16(
                self.instruction_pointer
                    // Offset for max_index that is one byte
                    .offset(offset),
            )
        }
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
}
