//! GMP-based modular exponentiation implementation

use rug::{integer::Order::Msf, Integer};
use std::vec::Vec;

/// Perform modular exponentiation using GMP library.
pub fn modexp(base: &[u8], exponent: &[u8], modulus: &[u8]) -> Vec<u8> {
    // Convert byte slices to GMP integers
    let base_int = Integer::from_digits(base, Msf);
    let exp_int = Integer::from_digits(exponent, Msf);
    let mod_int = Integer::from_digits(modulus, Msf);

    // Perform modular exponentiation using GMP's pow_mod
    let result = base_int.pow_mod(&exp_int, &mod_int).unwrap_or_default();

    // Convert result back to bytes
    let byte_count = result.significant_bits().div_ceil(8);
    let mut output = vec![0u8; byte_count as usize];
    result.write_digits(&mut output, Msf);
    output
}
