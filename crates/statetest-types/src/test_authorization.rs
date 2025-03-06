use revm::context_interface::transaction::SignedAuthorization;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

/// Struct for test authorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestAuthorization {
    #[serde(flatten)]
    inner: SignedAuthorization,
}

impl From<TestAuthorization> for SignedAuthorization {
    fn from(auth: TestAuthorization) -> Self {
        auth.inner
    }
}

impl<'de> Deserialize<'de> for TestAuthorization {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // This is a hack to remove duplicate yParity and v fields which can be used by the test files for cross client compat
        let mut value: serde_json::Value = Deserialize::deserialize(deserializer)?;
        if let Some(val) = value.as_object_mut() {
            if val.contains_key("v") && val.contains_key("yParity") {
                val.remove("v");
            }
        }
        let inner: SignedAuthorization = serde_json::from_value(value).map_err(D::Error::custom)?;
        Ok(Self { inner })
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

        let auth: TestAuthorization = serde_json::from_str(auth).unwrap();
        println!("{:?}", auth);
    }

    #[test]
    fn recover_auth_duplicate_v_yparity() {
        let auth = r#"{
            "chainId": "0x00",
            "address": "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b",
            "nonce": "0x00",
            "v": "0x01",
            "yParity": "0x01",
            "r": "0x5a8cac98fd240d8ef83c22db4a061ffa0facb1801245283cc05fc809d8b92837",
            "s": "0x1c3162fe11d91bc24d4fa00fb19ca34531e0eacdf8142c804be44058d5b8244f",
            "signer": "0x6389e7f33ce3b1e94e4325ef02829cd12297ef71"
        }"#;

        let _: TestAuthorization = serde_json::from_str(auth).unwrap();
    }
}
