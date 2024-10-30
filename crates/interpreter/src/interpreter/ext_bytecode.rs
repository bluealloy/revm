use bytecode::{
    eof::TypesSection,
    utils::{read_i16, read_u16},
    Bytecode,
};
use primitives::Bytes;

use super::{EofCodeInfo, EofContainer, EofData, Immediates, Jumps, LegacyBytecode};

#[derive(Debug)]
pub struct ExtBytecode {
    pub base: Bytecode,
    pub instruction_pointer: *const u8,
}

impl Jumps for ExtBytecode {
    fn relative_jump(&mut self, offset: isize) {
        self.instruction_pointer = unsafe { self.instruction_pointer.offset(offset) };
    }

    fn absolute_jump(&mut self, offset: usize) {
        self.instruction_pointer = unsafe { self.base.bytecode().as_ptr().add(offset) };
    }

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

    fn pc(&self) -> usize {
        // SAFETY: `instruction_pointer` should be at an offset from the start of the bytecode.
        // In practice this is always true unless a caller modifies the `instruction_pointer` field manually.
        unsafe {
            self.instruction_pointer
                .offset_from(self.base.bytecode().as_ptr()) as usize
        }
    }
}

impl Immediates for ExtBytecode {
    fn read_i16(&self) -> i16 {
        unsafe { read_i16(self.instruction_pointer) }
    }

    fn read_u16(&self) -> u16 {
        unsafe { read_u16(self.instruction_pointer) }
    }

    fn read_i8(&self) -> i8 {
        unsafe { core::mem::transmute(*self.instruction_pointer) }
    }

    fn read_u8(&self) -> u8 {
        unsafe { *self.instruction_pointer }
    }

    fn read_slice(&self, len: usize) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.instruction_pointer, len) }
    }

    fn read_offset_i16(&self, offset: isize) -> i16 {
        unsafe {
            read_i16(
                self.instruction_pointer
                    // offset for max_index that is one byte
                    .offset(offset),
            )
        }
    }
    fn read_offset_u16(&self, offset: isize) -> u16 {
        unsafe {
            read_u16(
                self.instruction_pointer
                    // offset for max_index that is one byte
                    .offset(offset),
            )
        }
    }
}

impl EofCodeInfo for ExtBytecode {
    fn code_section_info(&self, idx: usize) -> Option<&TypesSection> {
        self.base
            .eof()
            .map(|eof| eof.body.types_section.get(idx))
            .flatten()
    }

    fn code_section_pc(&self, idx: usize) -> Option<usize> {
        self.base
            .eof()
            .map(|eof| eof.body.eof_code_section_start(idx))
            .flatten()
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
            .map(|eof| eof.body.container_section.get(index))
            .flatten()
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
