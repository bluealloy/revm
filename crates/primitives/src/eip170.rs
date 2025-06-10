//! EIP-170: Contract code size limit

/// EIP-170: Contract code size limit
///
/// By default the limit is `0x6000` (~25kb).
pub const MAX_CODE_SIZE: usize = 0x6000;

/// MAX_INITCODE_SIZE is double of MAX_CODE_SIZE
pub const MAX_INITCODE_SIZE: usize = MAX_CODE_SIZE * 2;
