use super::{AccessListItemTr, AuthorizationTr};
use either::{for_both, Either};
use primitives::{Address, B256, U256};

pub use alloy_eip2930::{AccessList, AccessListItem};
pub use alloy_eip7702::{
    Authorization, RecoveredAuthority, RecoveredAuthorization, SignedAuthorization,
};

impl AccessListItemTr for AccessListItem {
    fn address(&self) -> &Address {
        &self.address
    }

    fn storage_slots(&self) -> impl Iterator<Item = &B256> {
        self.storage_keys.iter()
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

impl<L: AuthorizationTr, R: AuthorizationTr> AuthorizationTr for Either<L, R> {
    fn authority(&self) -> Option<Address> {
        for_both!(self, s => s.authority())
    }

    fn chain_id(&self) -> U256 {
        for_both!(self, s => s.chain_id())
    }

    fn nonce(&self) -> u64 {
        for_both!(self, s => s.nonce())
    }

    fn address(&self) -> Address {
        for_both!(self, s => s.address())
    }
}
