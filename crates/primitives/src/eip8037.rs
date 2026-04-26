//! EIP-8037: State Creation Gas Cost Increase
//!
//! Introduces a reservoir model that separates *state gas* (storage/code/account
//! creation) from *regular* execution gas. State-gas charges are expressed as
//! a number of "state bytes" that get multiplied by `cost_per_state_byte` (CPSB).
//! CPSB itself is derived from the current block's gas limit so that the state
//! growth target scales with block capacity.

/// Blocks per year at a 12-second block time (used by the CPSB formula).
pub const BLOCKS_PER_YEAR: u64 = 2_628_000;

/// Target yearly state growth budget, in bytes.
pub const TARGET_STATE_GROWTH_PER_YEAR: u64 = 100 * 1024 * 1024 * 1024;

/// Offset subtracted after rounding in the CPSB formula.
pub const CPSB_OFFSET: u64 = 9578;

/// Number of high-order bits retained when rounding CPSB.
pub const CPSB_SIGNIFICANT_BITS: u32 = 5;

/// State bytes charged per SSTORE 0→non-zero.
pub const SSTORE_SET_BYTES: u64 = 32;

/// State bytes charged when creating a new account.
pub const NEW_ACCOUNT_BYTES: u64 = 112;

/// State bytes charged per EIP-7702 authorization base cost.
pub const AUTH_BASE_BYTES: u64 = 23;

/// State bytes charged per byte of deployed code.
pub const CODE_DEPOSIT_PER_BYTE: u64 = 1;

/// Regular gas component of EIP-7702 `PER_EMPTY_ACCOUNT_COST` under EIP-8037.
pub const EIP7702_PER_EMPTY_ACCOUNT_REGULAR: u64 = 7500;

/// Computes `cost_per_state_byte` for the given block gas limit per EIP-8037.
///
/// ```text
/// raw     = ceil((block_gas_limit * BLOCKS_PER_YEAR) / (2 * TARGET_STATE_GROWTH_PER_YEAR))
/// shifted = raw + CPSB_OFFSET
/// shift   = max(bit_length(shifted) - CPSB_SIGNIFICANT_BITS, 0)
/// cpsb    = max(((shifted >> shift) << shift) - CPSB_OFFSET, 1)
/// ```
#[inline]
pub const fn cost_per_state_byte(block_gas_limit: u64) -> u64 {
    let numerator = (block_gas_limit as u128) * (BLOCKS_PER_YEAR as u128);
    let denominator = 2u128 * (TARGET_STATE_GROWTH_PER_YEAR as u128);
    let raw = numerator.div_ceil(denominator) as u64;

    let shifted = raw + CPSB_OFFSET;
    let bit_length = u64::BITS - shifted.leading_zeros();
    let shift = bit_length.saturating_sub(CPSB_SIGNIFICANT_BITS);

    let rounded = (shifted >> shift) << shift;
    let cpsb = rounded.saturating_sub(CPSB_OFFSET);
    if cpsb == 0 {
        1
    } else {
        cpsb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpsb_matches_spec_at_100m() {
        // Canonical reference: CPSB at a 100M block gas limit is 1174.
        assert_eq!(cost_per_state_byte(100_000_000), 1174);
    }

    #[test]
    fn cpsb_scales_with_block_gas_limit() {
        // Larger blocks → larger CPSB.
        let a = cost_per_state_byte(30_000_000);
        let b = cost_per_state_byte(100_000_000);
        let c = cost_per_state_byte(300_000_000);
        assert!(a < b);
        assert!(b < c);
    }

    #[test]
    fn cpsb_minimum_is_one() {
        // Degenerate inputs must still yield a positive cost.
        assert!(cost_per_state_byte(0) >= 1);
        assert!(cost_per_state_byte(1) >= 1);
    }
}
