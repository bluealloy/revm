use primitives::{alloy_primitives::B512, keccak256, B256};
use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message, SECP256K1,
};

// Silence the unused crate dependency warning.
use k256 as _;

pub fn ecrecover(sig: &B512, recid: u8, msg: &B256) -> Result<B256, secp256k1::Error> {
    let recid = RecoveryId::try_from(recid as i32).expect("recovery ID is valid");
    let sig = RecoverableSignature::from_compact(sig.as_slice(), recid)?;

    let msg = Message::from_digest(msg.0);
    let public = SECP256K1.recover_ecdsa(&msg, &sig)?;

    let mut hash = keccak256(&public.serialize_uncompressed()[1..]);
    hash[..12].fill(0);
    Ok(hash)
}
