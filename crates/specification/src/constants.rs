use super::eip170;

/// EVM interpreter stack limit
pub const STACK_LIMIT: usize = 1024;

/// EIP-3860: Limit and meter initcode
///
/// Limit of maximum initcode size is `2 * MAX_CODE_SIZE`.
pub const MAX_INITCODE_SIZE: usize = 2 * eip170::MAX_CODE_SIZE;

/// EVM call stack limit
pub const CALL_STACK_LIMIT: u64 = 1024;
