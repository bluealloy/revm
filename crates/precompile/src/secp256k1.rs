use crate::{utilities::right_pad, Error, Precompile, PrecompileResult, PrecompileWithAddress};
use revm_primitives::{Bytes, B256};

pub const ECRECOVER: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(1),
    Precompile::Standard(ec_recover_run),
);

#[link(wasm_import_module = "fluentbase_v1alpha")]
extern "C" {
    fn _crypto_keccak256(data_offset: *const u8, data_len: u32, output32_offset: *mut u8);
    fn _crypto_ecrecover(
        digest32_offset: *const u8,
        sig64_offset: *const u8,
        output65_offset: *mut u8,
        rec_id: u32,
    );
}

pub fn ec_recover_run(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    const ECRECOVER_BASE: u64 = 3_000;
    if ECRECOVER_BASE > gas_limit {
        return Err(Error::OutOfGas);
    }

    let input = right_pad::<128>(input);

    // `v` must be a 32-byte big-endian integer equal to 27 or 28.
    if !(input[32..63].iter().all(|&b| b == 0) && matches!(input[63], 27 | 28)) {
        return Ok((ECRECOVER_BASE, Bytes::new()));
    }

    let mut public_key: [u8; 65] = [0u8; 65];
    let mut hash: B256 = B256::ZERO;
    unsafe {
        _crypto_ecrecover(
            input[0..32].as_ptr(),
            input[64..128].as_ptr(),
            public_key.as_mut_ptr(),
            (input[63] - 27) as u32,
        );
        _crypto_keccak256(public_key[1..].as_ptr(), 64, hash.as_mut_ptr())
    }
    hash[..12].fill(0);
    Ok((ECRECOVER_BASE, Bytes::from(hash)))
}
