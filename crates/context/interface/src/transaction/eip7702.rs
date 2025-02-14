use auto_impl::auto_impl;
use primitives::{Address, U256};

/// Authorization trait.
#[auto_impl(&, Box, Arc, Rc)]
pub trait AuthorizationTr {
    /// Authority address.
    ///
    /// # Note
    ///
    /// Authority signature can be invalid, so this method returns None if the authority
    /// could not be recovered.
    ///
    /// Valid signature Parity should be 0 or 1 and
    /// signature s-value should be less than SECP256K1N_HALF.
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
