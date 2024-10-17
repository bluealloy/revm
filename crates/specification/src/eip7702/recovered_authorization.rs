use crate::eip7702::{Authorization, SignedAuthorization};
use core::ops::Deref;
use primitives::Address;

/// A recovered authorization.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecoveredAuthorization {
    #[cfg_attr(feature = "serde", serde(flatten))]
    inner: SignedAuthorization,
    authority: Option<Address>,
}

impl RecoveredAuthorization {
    /// Instantiate without performing recovery. This should be used carefully.
    pub const fn new_unchecked(inner: SignedAuthorization, authority: Option<Address>) -> Self {
        Self { inner, authority }
    }

    /// Returns a reference to the inner [`SignedAuthorization`].
    pub fn inner(&self) -> &SignedAuthorization {
        &self.inner
    }

    /// Get the `authority` for the authorization.
    ///
    /// If this is `None`, then the authority could not be recovered.
    pub const fn authority(&self) -> Option<Address> {
        self.authority
    }

    /// Splits the authorization into parts.
    pub const fn into_parts(self) -> (SignedAuthorization, Option<Address>) {
        (self.inner, self.authority)
    }
}

impl From<SignedAuthorization> for RecoveredAuthorization {
    fn from(signed_auth: SignedAuthorization) -> Self {
        let authority = signed_auth.recover_authority().ok();
        Self::new_unchecked(signed_auth, authority)
    }
}

impl Deref for RecoveredAuthorization {
    type Target = Authorization;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
