/// Read big-endian i16 from u8 pointer
///
/// # Safety
///
/// Pointer needs to point to at least 2 byte.
pub unsafe fn read_i16(ptr: *const u8) -> i16 {
    i16::from_be_bytes(core::slice::from_raw_parts(ptr, 2).try_into().unwrap())
}

/// Read big-endian u16 from u8 pointer
///
/// # Safety
///
/// Pointer needs to point to at least 2 byte.
pub unsafe fn read_u16(ptr: *const u8) -> u16 {
    u16::from_be_bytes(core::slice::from_raw_parts(ptr, 2).try_into().unwrap())
}
