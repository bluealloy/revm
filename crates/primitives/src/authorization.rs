//! [EIP-7702](https://eips.ethereum.org/EIPS/eip-7702) Set EOA account code for one transaction
//!
//! These transactions include a list of [`Authorization`]

use crate::AUTHORIZATION_MAGIC_BYTE;
use alloy_primitives::{Address, ChainId, Keccak256, U256};
use alloy_rlp::{BufMut, Encodable, Header};
use k256::{
    ecdsa::{RecoveryId, Signature, VerifyingKey},
    FieldBytes,
};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Authorization {
    /// Chain ID, will not be checked if set to zero
    pub chain_id: ChainId,
    /// The address of the code that will get set to the signer's address
    pub address: Address,
    /// Optional nonce
    pub nonce: Option<u64>,
    /// yParity: Signature Y parity
    pub y_parity: bool,
    /// The R field of the signature
    pub r: U256,
    /// The S field of the signature
    pub s: U256,
}

impl Authorization {
    /// Recover the address of the signer for EIP-7702 transactions
    /// Since these authorizations are fallible, we will ignore errors and optionally
    /// return the signer's address
    pub fn recovered_authority(&self) -> Option<Address> {
        Signature::from_scalars(
            *FieldBytes::from_slice(&self.r.to_be_bytes::<32>()),
            *FieldBytes::from_slice(&self.s.to_be_bytes::<32>()),
        )
        .ok()
        .and_then(|sig| RecoveryId::from_byte(self.y_parity as u8).map(|recid| (sig, recid)))
        .and_then(|(sig, recid)| {
            let nonce = self.nonce.map(|n| vec![n]).unwrap_or(vec![]);

            let mut length = 0;
            length += self.chain_id.length();
            length += self.address.length();
            length += nonce.length();

            let mut buffer = Vec::new();

            buffer.put_u8(AUTHORIZATION_MAGIC_BYTE);

            Header {
                list: true,
                payload_length: length,
            }
            .encode(&mut buffer);
            self.chain_id.encode(&mut buffer);
            self.address.encode(&mut buffer);
            nonce.encode(&mut buffer);

            let mut hasher = Keccak256::new();
            hasher.update(buffer);
            let hash = hasher.finalize();

            let recovered_key =
                VerifyingKey::recover_from_prehash(&hash.as_slice(), &sig, recid).ok()?;
            let encoded_point = recovered_key.to_encoded_point(false);

            let mut hasher = Keccak256::new();
            hasher.update(&encoded_point.as_bytes()[1..]);
            let address = &hasher.finalize()[12..];
            Some(Address::from_slice(address))
        })
    }
}
