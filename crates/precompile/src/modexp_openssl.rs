//! OpenSSL-based modular exponentiation implementation

use openssl::bn::{BigNum, BigNumContext};
use std::vec::Vec;

/// Performs modular exponentiation using OpenSSL's BN_mod_exp function.
///
/// This function computes: base^exp mod modulus
///
/// # Arguments
/// * `base` - The base as a byte slice
/// * `exponent` - The exponent as a byte slice
/// * `modulus` - The modulus as a byte slice
///
/// # Returns
/// The result of the modular exponentiation as a vector of bytes
pub fn modexp(base: &[u8], exponent: &[u8], modulus: &[u8]) -> Vec<u8> {
    // Handle edge cases
    if modulus.is_empty() || modulus.iter().all(|&b| b == 0) {
        return vec![];
    }

    // Create BIGNUMs from the input byte slices
    let base_bn = match BigNum::from_slice(base) {
        Ok(bn) => bn,
        Err(_) => return vec![],
    };

    let exp_bn = match BigNum::from_slice(exponent) {
        Ok(bn) => bn,
        Err(_) => return vec![],
    };

    let mod_bn = match BigNum::from_slice(modulus) {
        Ok(bn) => bn,
        Err(_) => return vec![],
    };

    // Create a new BigNum for the result
    let mut result = BigNum::new().unwrap();

    // Create a BigNumContext for the modexp operation
    let mut ctx = BigNumContext::new().unwrap();

    // Perform modular exponentiation: result = base^exp mod modulus
    match result.mod_exp(&base_bn, &exp_bn, &mod_bn, &mut ctx) {
        Ok(_) => {
            // Convert the result to bytes
            result.to_vec()
        }
        Err(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modexp_basic() {
        // Test: 2^3 mod 5 = 8 mod 5 = 3
        let base = vec![2];
        let exp = vec![3];
        let modulus = vec![5];
        let result = modexp(&base, &exp, &modulus);
        assert_eq!(result, vec![3]);
    }

    #[test]
    fn test_modexp_large_numbers() {
        // Test with larger numbers
        let base = vec![0x10, 0x00]; // 4096
        let exp = vec![0x02]; // 2
        let modulus = vec![0x03, 0xe8]; // 1000
        let result = modexp(&base, &exp, &modulus);

        // 4096^2 mod 1000 = 16777216 mod 1000 = 216
        assert_eq!(result, vec![0xd8]); // 216
    }

    #[test]
    fn test_modexp_zero_exponent() {
        // Test: anything^0 mod m = 1
        let base = vec![123];
        let exp = vec![0];
        let modulus = vec![100];
        let result = modexp(&base, &exp, &modulus);
        assert_eq!(result, vec![1]);
    }

    #[test]
    fn test_modexp_zero_modulus() {
        // Test: modulus of 0 should return empty
        let base = vec![2];
        let exp = vec![3];
        let modulus = vec![0];
        let result = modexp(&base, &exp, &modulus);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_modexp_empty_modulus() {
        // Test: empty modulus should return empty
        let base = vec![2];
        let exp = vec![3];
        let modulus = vec![];
        let result = modexp(&base, &exp, &modulus);
        assert_eq!(result, vec![]);
    }
}
