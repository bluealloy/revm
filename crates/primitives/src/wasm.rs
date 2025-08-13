use crate::Bytes;

/// WASM max code size
///
/// This value is temporary for testing purposes, requires recalculation.
/// The limit is equal to 1Mb.
pub const WASM_MAX_CODE_SIZE: usize = 0x200000;

/// SVM max code size
pub const SVM_MAX_CODE_SIZE: usize = 0x200000;

/// ERC20 max code size (25kB)
///
/// Keep the same value as for EVM contract deployment
pub const ERC20_MAX_CODE_SIZE: usize = 0x6000;

/// An alias for EVM max code size
pub const EVM_MAX_CODE_SIZE: usize = crate::eip170::MAX_CODE_SIZE;

/// WASM magic bytes
///
/// These values are equal to \0ASM
pub const WASM_MAGIC_BYTES: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];

/// SVM magic bytes (ELF header)
pub const SVM_ELF_MAGIC_BYTES: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];

/// ERC20 magic bytes: as char codes for "ERC" and the number 0x20
///
/// These values are equal to ERC\20
pub const ERC20_MAGIC_BYTES: [u8; 4] = [0x45, 0x52, 0x43, 0x20];

/// Get max code size based on the input signature
pub fn wasm_max_code_size(input: &Bytes) -> Option<usize> {
    let input: [u8; 4] = input.get(0..4)?.try_into().unwrap();
    match input {
        WASM_MAGIC_BYTES => Some(WASM_MAX_CODE_SIZE),
        SVM_ELF_MAGIC_BYTES => Some(SVM_MAX_CODE_SIZE),
        ERC20_MAGIC_BYTES => Some(ERC20_MAX_CODE_SIZE),
        _ => None,
    }
}
