//! EIP-7907: Meter Contract Code Size And Increase Limit

/// By default the limit is `0xc000` (~48KiB).
pub const MAX_CODE_SIZE: usize = 0xc000;
/// By default the limit is `0x18000` (~96KiB).
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;

/// Gas cost per word for code loading
pub const GAS_CODE_LOAD_WORD_COST: u64 = 4;
