use alloy_primitives::B256;

#[link(wasm_import_module = "fluentbase_v1preview")]
extern "C" {
    fn _keccak256(data_offset: *const u8, data_len: u32, output32_offset: *mut u8);
}

#[inline(always)]
pub fn keccak256(input: &[u8]) -> B256 {
    let mut result = B256::ZERO;
    unsafe {
        _keccak256(input.as_ptr(), input.len() as u32, result.as_mut_ptr());
    }
    result
}
