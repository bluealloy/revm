//! zkVM implementation of Blake2 compression function.

extern "C" {
    /// zkVM implementation of Blake2 F compression function.
    ///
    /// # Arguments
    /// * `rounds` - Number of rounds to perform
    /// * `h_ptr` - Pointer to 64-byte state vector (8 u64 values in little-endian)
    /// * `m_ptr` - Pointer to 128-byte message block (16 u64 values in little-endian)
    /// * `t_0` - Lower 64 bits of offset counter
    /// * `t_1` - Upper 64 bits of offset counter
    /// * `f` - Final block indicator flag (0 or 1)
    /// * `result_ptr` - Pointer to 64-byte output buffer for compressed state
    ///
    /// # Returns
    /// * 0 on success
    /// * Non-zero on error
    fn zkvm_blake2f_impl(
        rounds: u32,
        h_ptr: *const u8,
        m_ptr: *const u8,
        t_0: u64,
        t_1: u64,
        f: u8,
        result_ptr: *mut u8,
    ) -> i32;
}

/// Perform Blake2 F compression using zkVM implementation.
///
/// This function provides a hook for zkVM-optimized Blake2 compression.
/// The external implementation should perform the Blake2 F compression function
/// and return the new state vector.
pub fn compress(
    rounds: usize,
    h: &mut [u64; 8],
    m_slice: &[u8; 128], // 16 * size_of::<u64>() = 128 bytes
    t: [u64; 2],
    f: bool,
) {
    // Convert h to bytes for FFI
    let mut h_bytes = [0u8; 64];
    for (i, &h_val) in h.iter().enumerate() {
        h_bytes[i * 8..(i + 1) * 8].copy_from_slice(&h_val.to_le_bytes());
    }

    let mut result = [0u8; 64];

    let success = unsafe {
        zkvm_blake2f_impl(
            rounds as u32,
            h_bytes.as_ptr(),
            m_slice.as_ptr(),
            t[0],
            t[1],
            if f { 1 } else { 0 },
            result.as_mut_ptr(),
        )
    };

    if success == 0 {
        // Convert result back to h array
        for (i, h_val) in h.iter_mut().enumerate() {
            let bytes = &result[i * 8..(i + 1) * 8];
            *h_val = u64::from_le_bytes(bytes.try_into().unwrap());
        }
    } else {
        // On error, fall back to portable implementation
        super::super::blake2::algo::compress_fallback(rounds, h, m_slice, t, f);
    }
}
