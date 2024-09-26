use revm::{
    primitives::{Address, U256},
    specification::eip7702::{Authorization, Parity, RecoveredAuthorization, Signature},
};
use serde::{Deserialize, Serialize};

/// Test authorization.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestAuthorization {
    chain_id: U256,
    address: Address,
    nonce: U256,
    v: U256,
    r: U256,
    s: U256,
    signer: Option<Address>,
}

impl TestAuthorization {
    /// Get the signature using the v, r, s values.
    pub fn signature(&self) -> Signature {
        let v = u64::try_from(self.v).unwrap_or(u64::MAX);
        let parity = Parity::try_from(v).unwrap_or(Parity::Eip155(36));
        Signature::from_rs_and_parity(self.r, self.s, parity).unwrap()
    }

    /// Convert to a recovered authorization.
    pub fn into_recovered(self) -> RecoveredAuthorization {
        let authorization = Authorization {
            chain_id: self.chain_id,
            address: self.address,
            nonce: u64::try_from(self.nonce).unwrap(),
        };
        let authority = self
            .signature()
            .recover_address_from_prehash(&authorization.signature_hash())
            .ok();
        RecoveredAuthorization::new_unchecked(
            authorization.into_signed(self.signature()),
            authority,
        )
    }
}
