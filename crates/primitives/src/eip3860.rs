//! EIP-3860: Limit and meter initcode

use crate::eip170;

/// EIP-3860: Limit and meter initcode
///
/// Limit of maximum initcode size is `2 * eip170::MAX_CODE_SIZE`.
pub const MAX_INITCODE_SIZE: usize = 2 * eip170::MAX_CODE_SIZE;
