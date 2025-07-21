//! Cryptographic backend implementations for precompiles
//!
//! This module contains pure cryptographic implementations used by various precompiles.
//! The precompile logic (addresses, gas costs, input parsing) remains in the parent modules.

/// BN128 elliptic curve operations
pub mod bn128;

/// BLS12-381 elliptic curve operations  
pub mod bls12_381;

/// Blake2 compression function
pub mod blake2;

/// Hash functions (SHA-256, RIPEMD-160)
pub mod hash;

/// KZG point evaluation
#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub mod kzg;

/// Modular exponentiation
pub mod modexp;

/// secp256k1 elliptic curve operations
pub mod secp256k1;

/// secp256r1 (P-256) elliptic curve operations
pub mod secp256r1;