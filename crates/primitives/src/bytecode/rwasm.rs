use alloy_primitives::{bytes, Bytes};

/// Rwasm magic number in array form.
pub static WASM_MAGIC_BYTES: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];

/// Rwasm magic number in array form.
pub static RWASM_MAGIC_BYTES: Bytes = bytes!("ef52");
pub static WASM_MAGIC_BYTES: Bytes = bytes!("0061736d");
