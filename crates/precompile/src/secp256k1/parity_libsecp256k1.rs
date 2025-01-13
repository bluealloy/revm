use libsecp256k1::{recover, Error, Message, RecoveryId, Signature};
use primitives::{alloy_primitives::B512, keccak256, B256};

pub fn ecrecover(sig: &B512, recid: u8, msg: &B256) -> Result<B256, Error> {
    let recid = RecoveryId::parse(recid)?;
    let sig = Signature::parse_standard(sig)?;
    let msg = Message::parse(msg.as_ref());

    // uses static context.
    let public = recover(&msg, &sig, &recid)?;

    let mut hash = keccak256(&public.serialize()[1..]);
    hash[..12].fill(0);
    Ok(hash)
}
