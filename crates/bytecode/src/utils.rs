//! Various utilities for the bytecode

/// Reads a big-endian `i16` from a `u8` pointer.
///
/// # Safety
///
/// The pointer must point to at least 2 bytes.
#[inline]
pub unsafe fn read_i16(ptr: *const u8) -> i16 {
    read_u16(ptr) as i16
}

/// Reads a big-endian `u16` from a `u8` pointer.
///
/// # Safety
///
/// The pointer must point to at least 2 bytes.
#[inline]
pub unsafe fn read_u16(ptr: *const u8) -> u16 {
    u16::from_be_bytes(unsafe { ptr.cast::<[u8; 2]>().read() })
}
