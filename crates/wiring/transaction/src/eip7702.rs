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

/// Authorization trait.
#[auto_impl(&, Arc)]
pub trait Authorization {
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

    /// Returns true if the authorization is valid.
    ///
    /// Temporary method needed for older EIP spec and will removed in future
    /// when test get updated.
    fn is_invalid(&self) -> bool;
}

// TODO move to default context
use specification::eip7702::RecoveredAuthorization;

impl Authorization for RecoveredAuthorization {
    /// Authority address. Obtained by recovering of the signature.
    fn authority(&self) -> Option<Address> {
        self.authority()
    }

    /// Returns authorization the chain id.
    fn chain_id(&self) -> U256 {
        self.inner().chain_id()
    }

    /// Returns the nonce.
    ///
    /// # Note
    ///
    /// If nonce is not same as the nonce of the signer account,
    /// authorization is skipped and considered invalidated.
    fn nonce(&self) -> u64 {
        self.inner().nonce()
    }

    /// Returns the address that this account should delegate to.
    fn address(&self) -> Address {
        *self.inner().address()
    }

    /// Returns true if the authorization is valid.
    ///
    /// Temporary method needed for older EIP spec and will removed in future
    fn is_invalid(&self) -> bool {
        use specification::{eip2::SECP256K1N_HALF, eip7702::Parity};

        // Check y_parity, Parity::Parity means that it was 0 or 1.
        if !matches!(self.inner().signature().v(), Parity::Parity(_)) {
            return true;
        }

        // Check s-value
        if self.inner().signature().s() > SECP256K1N_HALF {
            return true;
        }

        false
    }
}
