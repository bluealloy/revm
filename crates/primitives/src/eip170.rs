//! EIP-170: Contract code size limit

/// EIP-170: Contract code size limit
///
/// By default the limit is `0x6000` (~25kb).
#[cfg(not(feature = "extended_code_size"))]
pub const MAX_CODE_SIZE: usize = 0x6000;
#[cfg(feature = "extended_code_size")]
pub const MAX_CODE_SIZE: usize = 0x60000;
