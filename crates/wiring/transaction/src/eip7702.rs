use crate::Eip1559Tx;
use primitives::Address;
use specification::eip7702::AuthorizationList;

/// EIP-7702 transaction, TODO set Trait for AuthorizationList.
pub trait Eip7702Tx: Eip1559Tx {
    /// Destination address of the call.
    fn destination(&self) -> Address;

    /// List of authorizations, that contains the signature that authorizes this
    /// caller to place the code to signer account.
    ///
    /// Set EOA account code for one transaction
    ///
    /// [EIP-Set EOA account code for one transaction](https://eips.ethereum.org/EIPS/eip-7702)
    fn authorization_list(&self) -> Option<&AuthorizationList>;
}
