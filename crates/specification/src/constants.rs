/// EVM interpreter stack limit.
pub const STACK_LIMIT: usize = 1024;

/// EIP-170: Contract code size limit
///
/// By default the limit is `0x6000` (~25kb)
pub const MAX_CODE_SIZE: usize = 0x6000;

/// EIP-3860: Limit and meter initcode
///
/// Limit of maximum initcode size is `2 * MAX_CODE_SIZE`.
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;
