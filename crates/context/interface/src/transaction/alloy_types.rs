use super::{AccessListTr, AuthorizationTr};
use primitives::{Address, B256, U256};

pub use alloy_eip2930::{AccessList, AccessListItem};
pub use alloy_eip7702::{
    Authorization, RecoveredAuthority, RecoveredAuthorization, SignedAuthorization,
};

use std::vec::Vec;

impl AccessListTr for Vec<AccessListItem> {
    fn access_list(&self) -> impl Iterator<Item = (Address, impl Iterator<Item = B256>)> {
        self.iter()
            .map(|item| (item.address, item.storage_keys.iter().cloned()))
    }
}

impl AccessListTr for AccessList {
    fn access_list(&self) -> impl Iterator<Item = (Address, impl Iterator<Item = B256>)> {
        self.0.access_list()
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
