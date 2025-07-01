//! zkVM implementation of modular exponentiation.

use std::vec::Vec;

extern "C" {
    /// zkVM implementation of modular exponentiation.
    ///
    /// # Arguments
    /// * `base_ptr` - Pointer to base value bytes
    /// * `base_len` - Length of base in bytes
    /// * `exp_ptr` - Pointer to exponent value bytes
    /// * `exp_len` - Length of exponent in bytes
    /// * `mod_ptr` - Pointer to modulus value bytes
    /// * `mod_len` - Length of modulus in bytes
    /// * `result_ptr` - Pointer to output buffer (must be at least mod_len bytes)
    /// * `result_len` - Length of the result buffer
    ///
    /// # Returns
    /// * Positive number: Number of bytes written to result buffer
    /// * -1: Invalid input or computation error
    fn zkvm_modexp_impl(
        base_ptr: *const u8,
        base_len: u32,
        exp_ptr: *const u8,
        exp_len: u32,
        mod_ptr: *const u8,
        mod_len: u32,
        result_ptr: *mut u8,
        result_len: u32,
    ) -> i32;
}

/// Compute modular exponentiation using zkVM implementation.
///
/// This function provides a hook for zkVM-optimized modular exponentiation.
/// The external implementation should handle arbitrary precision arithmetic
/// and return the result of (base^exponent) mod modulus.
pub fn modexp(base: &[u8], exponent: &[u8], modulus: &[u8]) -> Vec<u8> {
    // Special case: if modulus is zero, return empty result
    if modulus.is_empty() || modulus.iter().all(|&b| b == 0) {
        return Vec::new();
    }

    // Allocate result buffer with the same size as modulus
    let mut result = vec![0u8; modulus.len()];

    let bytes_written = unsafe {
        zkvm_modexp_impl(
            base.as_ptr(),
            base.len() as u32,
            exponent.as_ptr(),
            exponent.len() as u32,
            modulus.as_ptr(),
            modulus.len() as u32,
            result.as_mut_ptr(),
            result.len() as u32,
        )
    };

    if bytes_written < 0 {
        // Error occurred, fall back to a zero result of modulus length
        // This matches the behavior when modexp fails
        vec![0u8; modulus.len()]
    } else {
        // Resize to actual bytes written (in case result is smaller than modulus)
        result.truncate(bytes_written as usize);
        // Left pad with zeros if needed to match modulus length
        if result.len() < modulus.len() {
            let mut padded = vec![0u8; modulus.len() - result.len()];
            padded.extend(result);
            padded
        } else {
            result
        }
    }
}