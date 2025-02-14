use super::{AccessListTr, AuthorizationTr};
use primitives::{Address, B256, U256};

use alloy_eip2930::AccessList;
use alloy_eip7702::{RecoveredAuthorization, SignedAuthorization};

impl AccessListTr for AccessList {
    fn access_list(&self) -> impl Iterator<Item = (Address, impl Iterator<Item = B256>)> {
        self.0
            .iter()
            .map(|item| (item.address, item.storage_keys.iter().cloned()))
    }
}

impl AuthorizationTr for SignedAuthorization {
    fn authority(&self) -> Option<Address> {
        self.recover_authority().ok()
    }

    fn chain_id(&self) -> U256 {
        self.chain_id
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn address(&self) -> Address {
        self.address
    }
}

impl AuthorizationTr for RecoveredAuthorization {
    fn authority(&self) -> Option<Address> {
        self.authority()
    }

    fn chain_id(&self) -> U256 {
        self.chain_id
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn address(&self) -> Address {
        self.address
    }
}
