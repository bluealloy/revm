//! TODO dont have specific EIP. It is part of: EIP-7907: Meter Contract Code Size And Increase Limit

/// By default the limit is `0xC000` (49_152 bytes).
pub const MAX_CODE_SIZE: usize = 0xC000;
/// By default the limit is `0x12000` (73_728 bytes).
pub const MAX_INITCODE_SIZE: usize = 0x12000;
