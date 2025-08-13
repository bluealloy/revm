use primitives::Bytes;

/// Rwasm magic number in array form.
pub static RWASM_MAGIC_BYTES: Bytes = primitives::bytes!("ef52");
/// Wasm magic number in array form.
pub static WASM_MAGIC_BYTES: Bytes = primitives::bytes!("0061736d");
