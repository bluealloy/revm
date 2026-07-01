//! EIP-7954: Increase Maximum Contract Size
//!
//! Increases the contract code size limit and initcode size limit.

/// EIP-7954: Maximum contract code size: 65,536 bytes (0x10000).
pub const MAX_CODE_SIZE: usize = 0x10000;

/// EIP-7954: Maximum initcode size: 131,072 bytes (0x20000).
pub const MAX_INITCODE_SIZE: usize = 0x20000;
