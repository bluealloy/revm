use crate::Bytes;

/// rWASM max code size
///
/// This value is temporary for testing purposes, requires recalculation.
/// The limit is equal to 1Mb.
pub const WASM_MAX_CODE_SIZE: usize = 0x200000;

/// WebAssembly magic bytes
///
/// These values are equal to \0ASM
pub const WASM_MAGIC_BYTES: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];

pub fn wasm_max_code_size(input: &Bytes) -> Option<u32> {
    if input.len() > 4 && &input[0..4] == WASM_MAGIC_BYTES {
        Some(WASM_MAX_CODE_SIZE as u32)
    } else {
        None
    }
}
