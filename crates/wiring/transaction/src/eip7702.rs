use crate::Eip1559Tx;
use auto_impl::auto_impl;
use primitives::{Address, U256};

/// EIP-7702 transaction, TODO set Trait for AuthorizationList.
pub trait Eip7702Tx: Eip1559Tx {
    /// Destination address of the call.
    fn destination(&self) -> Address;

    /// Returns length of the authorization list.
    ///
    /// # Note
    ///
    /// Transaction is considered invalid if list is empty.
    fn authorization_list_len(&self) -> usize;

    /// List of authorizations, that contains the signature that authorizes this
    /// caller to place the code to signer account.
    ///
    /// Set EOA account code for one transaction
    ///
    /// [EIP-Set EOA account code for one transaction](https://eips.ethereum.org/EIPS/eip-7702)
    fn authorization_list_iter(&self) -> impl Iterator<Item = impl Authorization>;
}

/// Autorization trait.
#[auto_impl(&, Arc)]
pub trait Authorization {
    /// Authority address.
    ///
    /// # Note
    ///
    /// Authority signature can be invalid, so this method returns None if the authority
    /// could not be recovered.
    fn authority(&self) -> Option<Address>;

    /// Returns authorization the chain id.
    fn chain_id(&self) -> U256;

    /// Returns the nonce.
    ///
    /// # Note
    ///
    /// If nonce is not same as the nonce of the signer account,
    /// the authorization is skipped.
    fn nonce(&self) -> u64;

    /// Returns the address that this account is delegated to.
    fn address(&self) -> Address;
}

// TODO move to default context
use specification::eip7702::RecoveredAuthorization;

impl Authorization for RecoveredAuthorization {
    fn authority(&self) -> Option<Address> {
        self.authority()
    }

    fn chain_id(&self) -> U256 {
        self.inner().chain_id()
    }

    fn nonce(&self) -> u64 {
        self.inner().nonce()
    }

    fn address(&self) -> Address {
        *self.inner().address()
    }
}
