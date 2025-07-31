//! EIP-170: Contract Code Size Limit
//!
//! Introduces a maximum limit on smart contract code size.

/// EIP-170: Contract code size limit
///
/// By default the limit is `0x6000` (~25kb).
pub const MAX_CODE_SIZE: usize = 0x6000;
