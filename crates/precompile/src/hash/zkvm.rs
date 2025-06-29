//! zkVM implementation of hash functions.

extern "C" {
    /// zkVM implementation of SHA-256 hash function.
    ///
    /// # Arguments
    /// * `input_ptr` - Pointer to input data
    /// * `input_len` - Length of input data
    /// * `output_ptr` - Pointer to 32-byte output buffer
    ///
    /// # Returns
    /// * 0 on success
    /// * Non-zero on error
    fn zkvm_sha256_impl(input_ptr: *const u8, input_len: u32, output_ptr: *mut u8) -> i32;
}

/// Compute SHA-256 hash using zkVM implementation.
pub fn sha256_hash(input: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];

    let result =
        unsafe { zkvm_sha256_impl(input.as_ptr(), input.len() as u32, output.as_mut_ptr()) };

    if result != 0 {
        // Fallback to standard implementation on error
        // TODO: The function signature does not return an error, so we
        // TODO: either panic or fallback to naive implementation.
        sha2::Digest::digest(&sha2::Sha256::new().chain_update(input)).into()
    } else {
        output
    }
}
