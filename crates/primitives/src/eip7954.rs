//! EIP-7954: Increase Maximum Contract Size
//!
//! Increases the contract code size limit and initcode size limit.

/// EIP-7954: Maximum contract code size: 32,768 bytes (0x8000).
pub const MAX_CODE_SIZE: usize = 0x8000;

/// EIP-7954: Maximum initcode size: 65,536 bytes (0x10000).
pub const MAX_INITCODE_SIZE: usize = 0x10000;
