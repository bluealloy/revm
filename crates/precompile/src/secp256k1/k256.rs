//! k256 implementation of `ecrecover`. More about it in [`crate::secp256k1`].
use k256::ecdsa::{Error, RecoveryId, Signature, VerifyingKey};
use primitives::{alloy_primitives::B512, keccak256, B256};

/// Recover the public key from a signature and a message.
///
/// This function is using the `k256` crate.
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
