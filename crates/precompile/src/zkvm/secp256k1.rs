//! zkVM implementation of `ecrecover`

use primitives::{alloy_primitives::B512, B256};

extern "C" {
    /// zkVM implementation of ecrecover signature recovery.
    ///
    /// # Arguments
    /// * `sig_ptr` - Pointer to 64-byte signature (r || s)
    /// * `recid` - Recovery ID (0 or 1)
    /// * `msg_ptr` - Pointer to 32-byte message hash
    /// * `output_ptr` - Pointer to 32-byte output buffer for recovered address
    ///
    /// # Returns
    /// * 0 on success
    /// * Non-zero on error
    fn zkvm_ecrecover_impl(
        sig_ptr: *const u8,
        recid: u8,
        msg_ptr: *const u8,
        output_ptr: *mut u8,
    ) -> i32;
}

/// Recover the public key from a signature and a message using zkVM implementation.
///
/// This function provides a hook for zkVM-optimized signature recovery.
/// The external implementation should handle all cryptographic operations
/// and return the recovered Ethereum address directly.
pub fn ecrecover(sig: &B512, recid: u8, msg: &B256) -> Result<B256, k256::ecdsa::Error> {
    let mut output = [0u8; 32];

    let result =
        unsafe { zkvm_ecrecover_impl(sig.as_ptr(), recid, msg.as_ptr(), output.as_mut_ptr()) };

    if result == 0 {
        Ok(B256::from(output))
    } else {
        // Map zkVM errors to k256 errors for compatibility
        Err(k256::ecdsa::Error::new())
    }
}
