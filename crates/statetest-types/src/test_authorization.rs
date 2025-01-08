use revm::{
    context_interface::transaction::AuthorizationItem,
    primitives::{Address, U256},
    specification::eip2::SECP256K1N_HALF,
};
use serde::{Deserialize, Serialize};

/// Struct for test authorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestAuthorization {
    /// The chain ID of the authorization.
    pub chain_id: U256,
    /// The address of the authorization.
    pub address: Address,
    /// The nonce for the authorization.
    pub nonce: U256,
    v: U256,
    r: U256,
    s: U256,
    signer: Option<Address>,
}

impl From<TestAuthorization> for AuthorizationItem {
    fn from(auth: TestAuthorization) -> AuthorizationItem {
        let mut signer = auth.signer;

        if auth.s > SECP256K1N_HALF {
            signer = None
        }

        if auth.v > U256::from(1) {
            signer = None
        }

        (
            signer,
            auth.chain_id,
            auth.nonce.try_into().unwrap_or(u64::MAX),
            auth.address,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recover_auth() {
        // Test named:
        // tests/prague/eip7702_set_code_tx/test_gas.py::test_account_warming[fork_Prague-state_test-single_valid_authorization_single_signer-check_delegated_account_first_True]

        let auth = r#"{
            "chainId": "0x00",
            "address": "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b",
            "nonce": "0x00",
            "v": "0x01",
            "r": "0x5a8cac98fd240d8ef83c22db4a061ffa0facb1801245283cc05fc809d8b92837",
            "s": "0x1c3162fe11d91bc24d4fa00fb19ca34531e0eacdf8142c804be44058d5b8244f",
            "signer": "0x6389e7f33ce3b1e94e4325ef02829cd12297ef71"
        }"#;

        let _: TestAuthorization = serde_json::from_str(auth).unwrap();
    }
}
