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
    utilities::right_pad, PrecompileError, PrecompileOutput, PrecompileResult,
    PrecompileWithAddress,
};
use primitives::{alloy_primitives::B512, Bytes, B256};

/// `ecrecover` precompile, containing address and function to run.
pub const ECRECOVER: PrecompileWithAddress =
    PrecompileWithAddress(crate::u64_to_address(1), ec_recover_run);

/// `ecrecover` precompile function. Read more about input and output format in [this module docs](self).
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
