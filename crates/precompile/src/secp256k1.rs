use crate::{utilities::right_pad, Precompile, PrecompileResult, PrecompileWithAddress};
use revm_primitives::{Bytes, PrecompileOutput, B256};

pub const ECRECOVER: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(1),
    Precompile::Standard(ec_recover_run),
);

pub use self::secp256k1::ecrecover;

#[cfg(not(feature = "secp256k1"))]
#[allow(clippy::module_inception)]
mod secp256k1 {
    use k256::ecdsa::{Error, RecoveryId, Signature, VerifyingKey};
    use revm_primitives::{alloy_primitives::B512, keccak256, B256};

    pub fn ecrecover(sig: &B512, mut recid: u8, msg: &B256) -> Result<B256, Error> {
        // parse signature
        let mut sig = Signature::from_slice(sig.as_slice())?;

        // normalize signature and flip recovery id if needed.
        if let Some(sig_normalized) = sig.normalize_s() {
            sig = sig_normalized;
            recid ^= 1;
        }
        let recid = RecoveryId::from_byte(recid).expect("recovery ID is valid");

        // recover key
        let recovered_key = VerifyingKey::recover_from_prehash(&msg[..], &sig, recid)?;
        // hash it
        let mut hash = keccak256(
            &recovered_key
                .to_encoded_point(/* compress = */ false)
                .as_bytes()[1..],
        );

        // truncate to 20 bytes
        hash[..12].fill(0);
        Ok(hash)
    }
}

#[cfg(feature = "secp256k1")]
#[allow(clippy::module_inception)]
mod secp256k1 {
    use revm_primitives::{alloy_primitives::B512, keccak256, B256};
    use secp256k1::{
        ecdsa::{RecoverableSignature, RecoveryId},
        Message,
        SECP256K1,
    };

    pub fn ecrecover(sig: &B512, recid: u8, msg: &B256) -> Result<B256, secp256k1::Error> {
        let recid = RecoveryId::from_i32(recid as i32).expect("recovery ID is valid");
        let sig = RecoverableSignature::from_compact(sig.as_slice(), recid)?;

        let msg = Message::from_digest(msg.0);
        let public = SECP256K1.recover_ecdsa(&msg, &sig)?;

        let mut hash = keccak256(&public.serialize_uncompressed()[1..]);
        hash[..12].fill(0);
        Ok(hash)
    }
}

#[cfg(feature = "std")]
pub fn ec_recover_run(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    use revm_primitives::alloy_primitives::B512;

    const ECRECOVER_BASE: u64 = 3_000;

    if ECRECOVER_BASE > gas_limit {
        return Err(crate::Error::OutOfGas.into());
    }

    let input = right_pad::<128>(input);

    // `v` must be a 32-byte big-endian integer equal to 27 or 28.
    if !(input[32..63].iter().all(|&b| b == 0) && matches!(input[63], 27 | 28)) {
        return Ok(PrecompileOutput::new(ECRECOVER_BASE, Bytes::new()));
    }

    let msg = <&B256>::try_from(&input[0..32]).unwrap();
    let recid = input[63] - 27;
    let sig = <&B512>::try_from(&input[64..128]).unwrap();

    let out = secp256k1::ecrecover(sig, recid, msg)
        .map(|o| o.to_vec().into())
        .unwrap_or_default();
    Ok(PrecompileOutput::new(ECRECOVER_BASE, out))
}

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

#[cfg(not(feature = "std"))]
pub fn ec_recover_run(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    use revm_primitives::{PrecompileError, PrecompileErrors};
    const ECRECOVER_BASE: u64 = 3_000;
    if ECRECOVER_BASE > gas_limit {
        return Err(PrecompileErrors::Error(PrecompileError::OutOfGas));
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
