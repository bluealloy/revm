//! EIP-7907: Meter Contract Code Size And Increase Limit

/// By default the limit is `0x40000` (~262KiB).
pub const MAX_CODE_SIZE: usize = 0x40000;
/// By default the limit is `0x80000` (~524KiB).
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;
