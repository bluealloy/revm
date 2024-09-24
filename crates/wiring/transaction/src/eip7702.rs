use crate::CommonTxFields;
use specification::eip7702::AuthorizationList;

pub trait Eip7702Tx: CommonTxFields {
    /// List of authorizations, that contains the signature that authorizes this
    /// caller to place the code to signer account.
    ///
    /// Set EOA account code for one transaction
    ///
    /// [EIP-Set EOA account code for one transaction](https://eips.ethereum.org/EIPS/eip-7702)
    fn authorization_list(&self) -> Option<&AuthorizationList>;
}
