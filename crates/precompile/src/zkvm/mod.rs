//! zkVM implementations of precompiles.
//!
//! This module contains zkVM-optimized implementations of various precompiles
//! that can be used when running in zero-knowledge virtual machine environments.

pub mod bn128;
pub mod hash;
pub mod kzg_point_evaluation;
pub mod modexp;
pub mod secp256k1;
