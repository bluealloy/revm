//! `ecrecover` precompile.
//!
//! Depending on enabled features, it will use different implementations of `ecrecover`.
//! * [`k256`](https://crates.io/crates/k256) - uses maintained pure rust lib `k256`, it is perfect use for no_std environments.
//! * [`secp256k1`](https://crates.io/crates/secp256k1) - uses `bitcoin_secp256k1` lib, it is a C implementation of secp256k1 used in bitcoin core.
//!   It is faster than k256 and enabled by default and in std environment.
//! * [`libsecp256k1`](https://crates.io/crates/libsecp256k1) - is made from parity in pure rust, it is alternative for k256.
//!
//! Order of preference is `secp256k1` -> `k256` -> `libsecp256k1`. Where if no features are enabled, it will use `k256`.
//!
//! Input format:
//! [32 bytes for message][64 bytes for signature][1 byte for recovery id]
//!
//! Output format:
//! [32 bytes for recovered address]
#[cfg(feature = "secp256k1")]
pub mod bitcoin_secp256k1;
pub mod k256;
#[cfg(feature = "libsecp256k1")]
pub mod parity_libsecp256k1;

use crate::{
    utilities::right_pad,
    PrecompileError,
    PrecompileOutput,
    PrecompileResult,
    PrecompileWithAddress,
};
#[cfg(feature = "std")]
use primitives::alloy_primitives::B512;
use primitives::Bytes;
#[cfg(feature = "std")]
use primitives::B256;

/// `ecrecover` precompile, containing address and function to run.
pub const ECRECOVER: PrecompileWithAddress =
    PrecompileWithAddress(crate::u64_to_address(1), ec_recover_run);

/// `ecrecover` precompile function. Read more about input and output format in [this module docs](self).
#[cfg(feature = "std")]
pub fn ec_recover_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    const ECRECOVER_BASE: u64 = 3_000;

    if ECRECOVER_BASE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    let input = right_pad::<128>(input);

    // `v` must be a 32-byte big-endian integer equal to 27 or 28.
    if !(input[32..63].iter().all(|&b| b == 0) && matches!(input[63], 27 | 28)) {
        return Ok(PrecompileOutput::new(ECRECOVER_BASE, Bytes::new()));
    }

    let msg = <&B256>::try_from(&input[0..32]).unwrap();
    let recid = input[63] - 27;
    let sig = <&B512>::try_from(&input[64..128]).unwrap();

    let res = ecrecover(sig, recid, msg);

    let out = res.map(|o| o.to_vec().into()).unwrap_or_default();
    Ok(PrecompileOutput::new(ECRECOVER_BASE, out))
}

#[allow(missing_docs)]
#[cfg(not(feature = "std"))]
#[link(wasm_import_module = "fluentbase_v1preview")]
extern "C" {
    fn _keccak256(data_offset: *const u8, data_len: u32, output32_offset: *mut u8);
    fn _secp256k1_recover(
        digest32_offset: *const u8,
        sig64_offset: *const u8,
        output65_offset: *mut u8,
        rec_id: u32,
    ) -> i32;
}

#[allow(missing_docs)]
#[cfg(not(feature = "std"))]
pub fn ec_recover_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    const ECRECOVER_BASE: u64 = 3_000;
    if ECRECOVER_BASE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }
    let input = right_pad::<128>(input);
    // `v` must be a 32-byte big-endian integer equal to 27 or 28.
    if !(input[32..63].iter().all(|&b| b == 0) && matches!(input[63], 27 | 28)) {
        return Ok(PrecompileOutput::new(ECRECOVER_BASE, Bytes::new()));
    }

    let mut public_key: [u8; 65] = [0u8; 65];
    let ok = unsafe {
        _secp256k1_recover(
            input[0..32].as_ptr(),
            input[64..128].as_ptr(),
            public_key.as_mut_ptr(),
            (input[63] - 27) as u32,
        )
    };
    if ok == 0 {
        let mut hash = [0u8; 32];
        unsafe {
            _keccak256(public_key[1..].as_ptr(), 64, hash.as_mut_ptr());
        }
        hash[..12].fill(0);
        Ok(PrecompileOutput::new(ECRECOVER_BASE, Bytes::from(hash)))
    } else {
        Ok(PrecompileOutput::new(ECRECOVER_BASE, Bytes::new()))
    }
}

// Select the correct implementation based on the enabled features.
cfg_if::cfg_if! {
    if #[cfg(feature = "secp256k1")] {
        pub use bitcoin_secp256k1::ecrecover;
    } else if #[cfg(feature = "libsecp256k1")] {
        pub use parity_libsecp256k1::ecrecover;
    } else {
        pub use k256::ecrecover;
    }
}
