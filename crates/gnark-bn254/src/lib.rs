//! Gnark-crypto BN254 implementation for REVM precompiles
//!
//! This crate provides FFI bindings to the gnark-crypto Go library
//! for BN254 elliptic curve operations.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[link(name = "gnark_bn254", kind = "static")]
extern "C" {
    pub fn gnark_bn254_g1_add(
        p1: *const u8,
        p2: *const u8,
        out: *mut u8,
    ) -> i32;

    pub fn gnark_bn254_g1_mul(
        point: *const u8,
        scalar: *const u8,
        out: *mut u8,
    ) -> i32;

    pub fn gnark_bn254_pairing_check(
        pairs_data: *const u8,
        num_pairs: i32,
        result: *mut u8,
    ) -> i32;
}

// Re-export for use in precompile crate
pub use primitives;
