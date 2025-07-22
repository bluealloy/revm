//! `ecrecover` precompile.
//!
//! The implementation uses the `secp256k1` curve to recover the public key from a signature.
//! See [`crypto::secp256k1`](crate::crypto::secp256k1) for the underlying implementations.
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

    let msg = <&B256>::try_from(&input[0..32]).unwrap();
    let recid = input[63] - 27;
    let sig = <&B512>::try_from(&input[64..128]).unwrap();

    let res = ecrecover_bytes(sig.0, recid, msg.0);
    let out = res.map(|o| o.to_vec().into()).unwrap_or_default();
    Ok(PrecompileOutput::new(ECRECOVER_BASE, out))
}

fn ecrecover_bytes(sig: [u8; 64], recid: u8, msg: [u8; 32]) -> Option<[u8; 32]> {
    let sig = B512::from(sig);
    let msg = B256::from(msg);

    crate::crypto::get_provider()
        .ecrecover(&sig, recid, &msg)
        .map(|address| address.0)
}
