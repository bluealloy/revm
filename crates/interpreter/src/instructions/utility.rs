pub(crate) fn read_i16(ptr: *const u8) -> i16 {
    unsafe { i16::from_be_bytes(core::slice::from_raw_parts(ptr, 2).try_into().unwrap()) }
}

pub(crate) fn read_u16(ptr: *const u8) -> u16 {
    unsafe { u16::from_be_bytes(core::slice::from_raw_parts(ptr, 2).try_into().unwrap()) }
}
