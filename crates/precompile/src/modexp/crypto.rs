//! Pure cryptographic implementation for modular exponentiation.
//!
//! This module isolates the modexp computation from the precompile runner,
//! containing only the mathematical implementation without EVM-specific concerns.

use std::vec::Vec;

#[cfg(feature = "gmp")]
/// GMP-based modular exponentiation implementation
pub(crate) fn modexp(base: &[u8], exponent: &[u8], modulus: &[u8]) -> Vec<u8> {
    use core::ffi::c_void;
    use core::mem::MaybeUninit;
    use gmp_mpfr_sys::gmp;

    struct Mpz(gmp::mpz_t);

    impl Mpz {
        fn new() -> Self {
            unsafe {
                let mut inner = MaybeUninit::<gmp::mpz_t>::uninit();
                gmp::mpz_init(inner.as_mut_ptr());
                Self(inner.assume_init())
            }
        }

        fn as_ptr(&self) -> *const gmp::mpz_t {
            &self.0
        }

        fn as_mut_ptr(&mut self) -> *mut gmp::mpz_t {
            &mut self.0
        }

        fn set_from_be_bytes(&mut self, bytes: &[u8]) {
            unsafe {
                if bytes.is_empty() {
                    gmp::mpz_set_ui(self.as_mut_ptr(), 0);
                    return;
                }

                gmp::mpz_import(
                    self.as_mut_ptr(),
                    bytes.len(),
                    1,
                    1,
                    1,
                    0,
                    bytes.as_ptr() as *const c_void,
                );
            }
        }

        fn to_be_bytes(&self) -> Vec<u8> {
            unsafe {
                if gmp::mpz_sgn(self.as_ptr()) == 0 {
                    return Vec::new();
                }

                let bits = gmp::mpz_sizeinbase(self.as_ptr(), 2);
                let mut output = vec![0u8; bits.div_ceil(8)];
                let mut count: usize = 0;
                gmp::mpz_export(
                    output.as_mut_ptr() as *mut c_void,
                    &mut count,
                    1,
                    1,
                    1,
                    0,
                    self.as_ptr(),
                );
                output.truncate(count);
                output
            }
        }
    }

    impl Drop for Mpz {
        fn drop(&mut self) {
            unsafe {
                gmp::mpz_clear(self.as_mut_ptr());
            }
        }
    }

    let mut base_int = Mpz::new();
    let mut exp_int = Mpz::new();
    let mut mod_int = Mpz::new();
    let mut result = Mpz::new();

    base_int.set_from_be_bytes(base);
    exp_int.set_from_be_bytes(exponent);
    mod_int.set_from_be_bytes(modulus);

    unsafe {
        if gmp::mpz_sgn(mod_int.as_ptr()) == 0 {
            return Vec::new();
        }

        gmp::mpz_powm(
            result.as_mut_ptr(),
            base_int.as_ptr(),
            exp_int.as_ptr(),
            mod_int.as_ptr(),
        );
    }

    result.to_be_bytes()
}

#[cfg(not(feature = "gmp"))]
pub(crate) fn modexp(base: &[u8], exponent: &[u8], modulus: &[u8]) -> Vec<u8> {
    aurora_engine_modexp::modexp(base, exponent, modulus)
}
