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

    let msg: [u8; 32] = input[0..32].try_into().expect("expected 32 bytes");
    let recid = input[63] - 27;
    let sig: [u8; 64] = input[64..128].try_into().expect("expected 64 bytes");

    let res = ecrecover_bytes(sig, recid, msg);
    let out = res.map(|o| o.to_vec().into()).unwrap_or_default();
    Ok(PrecompileOutput::new(ECRECOVER_BASE, out))
}

fn ecrecover_bytes(sig: [u8; 64], recid: u8, msg: [u8; 32]) -> Option<[u8; 32]> {
    let sig = B512::from_slice(&sig);
    let msg = B256::from_slice(&msg);

    match ecrecover(&sig, recid, &msg) {
        Ok(address) => Some(address.0),
        Err(_) => None,
    }
}

// Re-export ecrecover from crypto_provider for backward compatibility
// TODO: switch to crypto_provider
pub use crate::crypto_provider::secp256k1::ecrecover;
