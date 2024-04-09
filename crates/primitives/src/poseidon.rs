use alloy_primitives::B256;

#[link(wasm_import_module = "fluentbase_v1alpha")]
extern "C" {
    pub fn _crypto_poseidon(data_offset: *const u8, data_len: u32, output32_offset: *mut u8);
}

#[inline(always)]
pub fn poseidon_hash(input: &[u8]) -> B256 {
    let mut result = B256::ZERO;
    unsafe {
        _crypto_poseidon(input.as_ptr(), input.len() as u32, result.as_mut_ptr());
    }
    result
}
