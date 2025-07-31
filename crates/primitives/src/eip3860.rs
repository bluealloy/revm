//! EIP-3860: Limit and Meter Initcode
//!
//! Introduces limits and gas metering for contract creation code.

use crate::eip170;

/// EIP-3860: Limit and meter initcode
///
/// Limit of maximum initcode size is `2 * eip170::MAX_CODE_SIZE`.
pub const MAX_INITCODE_SIZE: usize = 2 * eip170::MAX_CODE_SIZE;
