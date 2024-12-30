use revm::{
    context_interface::transaction::AuthorizationItem,
    primitives::{Address, U256},
};
use serde::{Deserialize, Serialize};

/// Struct for test authorization
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

impl From<TestAuthorization> for AuthorizationItem {
    fn from(auth: TestAuthorization) -> AuthorizationItem {
        (
            auth.signer,
            auth.chain_id,
            auth.nonce.try_into().unwrap_or(u64::MAX),
            auth.address,
        )
    }
}
