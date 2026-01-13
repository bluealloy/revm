//! Common pairing validation logic shared between arkworks and blst backends.
//!
//! This module intentionally holds only the byte-level validation/filtering logic that is
//! identical across backends. Backend-specific parsing and pairing computation are injected
//! as function parameters.

use crate::PrecompileError;
use std::vec::Vec;

/// Shared implementation of `pairing_check_bytes`.
#[inline]
pub(super) fn pairing_check_bytes_generic<G1, G2, ReadG1, ReadG2, PairingCheck>(
    pairs: &[super::PairingPair],
    read_g1: ReadG1,
    read_g2: ReadG2,
    pairing_check: PairingCheck,
) -> Result<bool, PrecompileError>
where
    ReadG1: Fn(&[u8; 48], &[u8; 48]) -> Result<G1, PrecompileError>,
    ReadG2: Fn(&[u8; 48], &[u8; 48], &[u8; 48], &[u8; 48]) -> Result<G2, PrecompileError>,
    PairingCheck: FnOnce(&[(G1, G2)]) -> bool,
{
    if pairs.is_empty() {
        return Ok(true);
    }

    let mut parsed_pairs = Vec::with_capacity(pairs.len());
    for ((g1_x, g1_y), (g2_x_0, g2_x_1, g2_y_0, g2_y_1)) in pairs {
        // Check if G1 point is zero (point at infinity)
        let g1_is_zero = g1_x.iter().all(|&b| b == 0) && g1_y.iter().all(|&b| b == 0);

        // Check if G2 point is zero (point at infinity)
        let g2_is_zero = g2_x_0.iter().all(|&b| b == 0)
            && g2_x_1.iter().all(|&b| b == 0)
            && g2_y_0.iter().all(|&b| b == 0)
            && g2_y_1.iter().all(|&b| b == 0);

        // Skip this pair if either point is at infinity as it's a no-op
        if g1_is_zero || g2_is_zero {
            // Still need to validate the non-zero point if one exists
            if !g1_is_zero {
                let _ = read_g1(g1_x, g1_y)?;
            }
            if !g2_is_zero {
                let _ = read_g2(g2_x_0, g2_x_1, g2_y_0, g2_y_1)?;
            }
            continue;
        }

        let g1_point = read_g1(g1_x, g1_y)?;
        let g2_point = read_g2(g2_x_0, g2_x_1, g2_y_0, g2_y_1)?;
        parsed_pairs.push((g1_point, g2_point));
    }

    // If all pairs were filtered out, return true (identity element)
    if parsed_pairs.is_empty() {
        return Ok(true);
    }

    Ok(pairing_check(&parsed_pairs))
}
