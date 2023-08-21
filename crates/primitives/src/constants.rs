use crate::B160;

/// Interpreter stack limit
pub const STACK_LIMIT: u64 = 1024;
/// EVM call stack limit
pub const CALL_STACK_LIMIT: u64 = 1024;

/// EIP-170: Contract code size limit
/// By default limit is 0x6000 (~25kb)
pub const MAX_CODE_SIZE: usize = 0x6000;

/// Number of blocks hashes that EVM can access in the past
pub const BLOCK_HASH_HISTORY: usize = 256;

/// EIP-3860: Limit and meter initcode
///
/// Limit of maximum initcode size is 2 * MAX_CODE_SIZE
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;

/// Precompile 3 is special in few places
pub const PRECOMPILE3: B160 = B160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]);
