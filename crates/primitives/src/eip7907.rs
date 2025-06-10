//! EIP-7907: Meter Contract Code Size And Increase Limit

/// EIP-7907: Meter Contract Code Size And Increase Limit
///
/// From the max code size in EIP-170. Default is `0x6000` (~24kb).
pub const LARGE_CODE_SIZE_THRESHOLD: usize = 0x6000;
/// By default the limit is `0x40000` (~262kb).
pub const MAX_CODE_SIZE: usize = 0x40000;
/// By default the limit is `0x80000` (~524kb).
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;
