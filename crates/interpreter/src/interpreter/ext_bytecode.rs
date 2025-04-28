use core::ops::Deref;

use bytecode::{eof::CodeInfo, utils::read_u16, Bytecode};
use primitives::{Bytes, B256};

use super::{EofCodeInfo, EofContainer, EofData, Immediates, Jumps, LegacyBytecode};

#[cfg(feature = "serde")]
mod serde;

#[derive(Debug)]
pub struct ExtBytecode {
    base: Bytecode,
    bytecode_hash: Option<B256>,
    instruction_pointer: *const u8,
}

impl Deref for ExtBytecode {
    type Target = Bytecode;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl ExtBytecode {
    /// Create new extended bytecode and set the instruction pointer to the start of the bytecode.
    pub fn new(base: Bytecode) -> Self {
        let instruction_pointer = base.bytecode_ptr();
        Self {
            base,
            instruction_pointer,
            bytecode_hash: None,
        }
    }

    /// Creates new `ExtBytecode` with the given hash.
    pub fn new_with_hash(base: Bytecode, hash: B256) -> Self {
        let instruction_pointer = base.bytecode_ptr();
        Self {
            base,
            instruction_pointer,
            bytecode_hash: Some(hash),
        }
    }

    /// Regenerates the bytecode hash.
    pub fn regenerate_hash(&mut self) -> B256 {
        let hash = self.base.hash_slow();
        self.bytecode_hash = Some(hash);
        hash
    }

    /// Returns the bytecode hash.
    pub fn hash(&mut self) -> Option<B256> {
        self.bytecode_hash
    }
}

impl Jumps for ExtBytecode {
    #[inline]
    fn relative_jump(&mut self, offset: isize) {
        self.instruction_pointer = unsafe { self.instruction_pointer.offset(offset) };
    }

    #[inline]
    fn absolute_jump(&mut self, offset: usize) {
        self.instruction_pointer = unsafe { self.base.bytes_ref().as_ptr().add(offset) };
    }

    #[inline]
    fn is_valid_legacy_jump(&mut self, offset: usize) -> bool {
        self.base
            .legacy_jump_table()
            .expect("Panic if not legacy")
            .is_valid(offset)
    }

    #[inline]
    fn opcode(&self) -> u8 {
        // SAFETY: `instruction_pointer` always point to bytecode.
        unsafe { *self.instruction_pointer }
    }

    #[inline]
    fn pc(&self) -> usize {
        // SAFETY: `instruction_pointer` should be at an offset from the start of the bytes.
        // In practice this is always true unless a caller modifies the `instruction_pointer` field manually.
        unsafe {
            self.instruction_pointer
                .offset_from(self.base.bytes_ref().as_ptr()) as usize
        }
    }
}

impl Immediates for ExtBytecode {
    #[inline]
    fn read_u16(&self) -> u16 {
        unsafe { read_u16(self.instruction_pointer) }
    }

    #[inline]
    fn read_u8(&self) -> u8 {
        unsafe { *self.instruction_pointer }
    }

    #[inline]
    fn read_slice(&self, len: usize) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.instruction_pointer, len) }
    }

    #[inline]
    fn read_offset_u16(&self, offset: isize) -> u16 {
        unsafe {
            read_u16(
                self.instruction_pointer
                    // Offset for max_index that is one byte
                    .offset(offset),
            )
        }
    }
}

impl EofCodeInfo for ExtBytecode {
    fn code_info(&self, idx: usize) -> Option<&CodeInfo> {
        self.base.eof().and_then(|eof| eof.body.code_info.get(idx))
    }

    fn code_section_pc(&self, idx: usize) -> Option<usize> {
        self.base
            .eof()
            .and_then(|eof| eof.body.eof_code_section_start(idx))
    }
}

impl EofData for ExtBytecode {
    fn data(&self) -> &[u8] {
        self.base.eof().expect("eof").data()
    }

    fn data_slice(&self, offset: usize, len: usize) -> &[u8] {
        self.base.eof().expect("eof").data_slice(offset, len)
    }

    fn data_size(&self) -> usize {
        self.base.eof().expect("eof").header.data_size as usize
    }
}

impl EofContainer for ExtBytecode {
    fn eof_container(&self, index: usize) -> Option<&Bytes> {
        self.base
            .eof()
            .and_then(|eof| eof.body.container_section.get(index))
    }
}

impl LegacyBytecode for ExtBytecode {
    fn bytecode_len(&self) -> usize {
        // Inform the optimizer that the bytecode cannot be EOF to remove a bounds check.
        assume!(!self.base.is_eof());
        self.base.len()
    }

    fn bytecode_slice(&self) -> &[u8] {
        // Inform the optimizer that the bytecode cannot be EOF to remove a bounds check.
        assume!(!self.base.is_eof());
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
