use crate::{Error, Precompile, PrecompileAddress, PrecompileResult, StandardPrecompileFn};
use alloc::vec::Vec;
use core::cmp::min;

const ECRECOVER_BASE: u64 = 3_000;

pub const ECRECOVER: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(1),
    Precompile::Standard(ec_recover_run as StandardPrecompileFn),
);

#[cfg(feature = "k256_ecrecover")]
#[allow(clippy::module_inception)]
mod secp256k1 {
    use core::convert::TryFrom;
    use k256::{
        ecdsa::{recoverable, Error},
        elliptic_curve::sec1::ToEncodedPoint,
        PublicKey as K256PublicKey,
    };
    use sha3::{Digest, Keccak256};

    use crate::B256;

    pub fn ecrecover(sig: &[u8; 65], msg: &B256) -> Result<B256, Error> {
        let sig = recoverable::Signature::try_from(sig.as_ref())?;
        let verify_key = sig.recover_verifying_key_from_digest_bytes(msg.into())?;
        let public_key = K256PublicKey::from(&verify_key);
        let public_key = public_key.to_encoded_point(/* compress = */ false);
        let public_key = public_key.as_bytes();
        let hash = Keccak256::digest(&public_key[1..]);
        let mut hash: B256 = hash[..].try_into().unwrap();
        hash.iter_mut().take(12).for_each(|i| *i = 0);
        Ok(hash)
    }
}

#[cfg(all(not(feature = "k256_ecrecover"), feature = "secp256k1"))]
#[allow(clippy::module_inception)]
mod secp256k1 {
    use crate::B256;
    use secp256k1::{
        ecdsa::{RecoverableSignature, RecoveryId},
        Message, Secp256k1,
    };
    use sha3::{Digest, Keccak256};

    pub fn ecrecover(sig: &[u8; 65], msg: &B256) -> Result<B256, secp256k1::Error> {
        let sig =
            RecoverableSignature::from_compact(&sig[0..64], RecoveryId::from_i32(sig[64] as i32)?)?;

        let secp = Secp256k1::new();
        let public = secp.recover_ecdsa(&Message::from_slice(&msg[..32])?, &sig)?;

        let hash = Keccak256::digest(&public.serialize_uncompressed()[1..]);
        let mut hash: B256 = hash[..].try_into().unwrap();
        hash.iter_mut().take(12).for_each(|i| *i = 0);
        Ok(hash)
    }
}

fn ec_recover_run(i: &[u8], target_gas: u64) -> PrecompileResult {
    if ECRECOVER_BASE > target_gas {
        return Err(Error::OutOfGas);
    }
    let mut input = [0u8; 128];
    input[..min(i.len(), 128)].copy_from_slice(&i[..min(i.len(), 128)]);

    let mut msg = [0u8; 32];
    let mut sig = [0u8; 65];

    msg[0..32].copy_from_slice(&input[0..32]);
    sig[0..32].copy_from_slice(&input[64..96]);
    sig[32..64].copy_from_slice(&input[96..128]);

    if input[32..63] != [0u8; 31] || !matches!(input[63], 27 | 28) {
        return Ok((ECRECOVER_BASE, Vec::new()));
    }

    sig[64] = input[63] - 27;

    let out = secp256k1::ecrecover(&sig, &msg)
        .map(Vec::from)
        .unwrap_or_default();

    Ok((ECRECOVER_BASE, out))
}
