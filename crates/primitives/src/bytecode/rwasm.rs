use alloy_primitives::{bytes, Bytes};

/// Rwasm magic number in array form.
pub static RWASM_MAGIC_BYTES: Bytes = bytes!("ef52");
pub static WASM_MAGIC_BYTES: Bytes = bytes!("0061736d");
