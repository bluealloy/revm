//! TODO EIP, it is part of: EIP-7907: Meter Contract Code Size And Increase Limit

/// By default the limit is `0xC000` (~262kb).
pub const MAX_CODE_SIZE: usize = 0xC000;
/// By default the limit is `0x80000` (~524kb).
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;
