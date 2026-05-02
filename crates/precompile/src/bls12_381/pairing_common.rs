//! Common pairing validation logic shared between arkworks and blst backends.
//!
//! This module intentionally holds only the byte-level validation/filtering logic that is
//! identical across backends. Backend-specific parsing and pairing computation are injected
//! as function parameters.

use crate::PrecompileHalt;
use std::vec::Vec;

/// Zero-check on a 48-byte field element via 6 `u64` chunks. Faster than a
/// byte loop because LLVM emits a SIMD load + horizontal reduce on aarch64
/// and an SSE compare on x86_64 while still short-circuiting at the field
/// boundary. The `try_into` panics are elided since slice lengths are
/// statically known.
#[inline]
fn fp_is_zero(a: &[u8; 48]) -> bool {
    let w = [
        u64::from_ne_bytes(a[0..8].try_into().unwrap()),
        u64::from_ne_bytes(a[8..16].try_into().unwrap()),
        u64::from_ne_bytes(a[16..24].try_into().unwrap()),
        u64::from_ne_bytes(a[24..32].try_into().unwrap()),
        u64::from_ne_bytes(a[32..40].try_into().unwrap()),
        u64::from_ne_bytes(a[40..48].try_into().unwrap()),
    ];
    w.iter().all(|&x| x == 0)
}

/// Shared implementation of `pairing_check_bytes`.
#[inline]
pub(super) fn pairing_check_bytes_generic<G1, G2, ReadG1, ReadG2, PairingCheck>(
    pairs: &[super::PairingPair],
    read_g1: ReadG1,
    read_g2: ReadG2,
    pairing_check: PairingCheck,
) -> Result<bool, PrecompileHalt>
where
    ReadG1: Fn(&[u8; 48], &[u8; 48]) -> Result<G1, PrecompileHalt>,
    ReadG2: Fn(&[u8; 48], &[u8; 48], &[u8; 48], &[u8; 48]) -> Result<G2, PrecompileHalt>,
    PairingCheck: FnOnce(&[(G1, G2)]) -> bool,
{
    if pairs.is_empty() {
        return Ok(true);
    }

    let mut parsed_pairs = Vec::with_capacity(pairs.len());
    for ((g1_x, g1_y), (g2_x_0, g2_x_1, g2_y_0, g2_y_1)) in pairs {
        // Check if G1 point is zero (point at infinity)
        let g1_is_zero = fp_is_zero(g1_x) && fp_is_zero(g1_y);

        // Check if G2 point is zero (point at infinity)
        let g2_is_zero =
            fp_is_zero(g2_x_0) && fp_is_zero(g2_x_1) && fp_is_zero(g2_y_0) && fp_is_zero(g2_y_1);

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
