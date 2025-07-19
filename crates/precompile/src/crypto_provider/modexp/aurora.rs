//! Aurora engine modular exponentiation implementation

use std::vec::Vec;

/// Perform modular exponentiation using aurora-engine-modexp library.
pub fn modexp(base: &[u8], exponent: &[u8], modulus: &[u8]) -> Vec<u8> {
    aurora_engine_modexp::modexp(base, exponent, modulus)
}
