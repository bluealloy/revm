pub use alloy_eips::eip7702::{Authorization, SignedAuthorization};
pub use alloy_primitives::Signature;

use crate::Address;
use core::ops::Deref;
use std::{boxed::Box, vec::Vec};

/// Authorization list for EIP-7702 transaction type.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AuthorizationList {
    Signed(Vec<SignedAuthorization>),
    Recovered(Vec<RecoveredAuthorization>),
}

impl From<Vec<SignedAuthorization>> for AuthorizationList {
    fn from(signed: Vec<SignedAuthorization>) -> Self {
        Self::Signed(signed)
    }
}

impl From<Vec<RecoveredAuthorization>> for AuthorizationList {
    fn from(recovered: Vec<RecoveredAuthorization>) -> Self {
        Self::Recovered(recovered)
    }
}

impl AuthorizationList {
    /// Returns length of the authorization list.
    pub fn len(&self) -> usize {
        match self {
            Self::Signed(signed) => signed.len(),
            Self::Recovered(recovered) => recovered.len(),
        }
    }

    /// Return empty authorization list.
    pub fn empty() -> Self {
        Self::Recovered(Vec::new())
    }

    /// Returns true if the authorization list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns iterator of recovered Authorizations.
    pub fn recovered_iter<'a>(&'a self) -> Box<dyn Iterator<Item = RecoveredAuthorization> + 'a> {
        match self {
            Self::Signed(signed) => Box::new(signed.iter().map(|signed| signed.clone().into())),
            Self::Recovered(recovered) => Box::new(recovered.clone().into_iter()),
        }
    }

    /// Returns recovered authorizations list.
    pub fn into_recovered(self) -> Self {
        let Self::Signed(signed) = self else {
            return self;
        };
        Self::Recovered(signed.into_iter().map(|signed| signed.into()).collect())
    }
}

/// A recovered authorization.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecoveredAuthorization {
    #[cfg_attr(feature = "serde", serde(flatten))]
    inner: Authorization,
    authority: Option<Address>,
}

impl RecoveredAuthorization {
    /// Instantiate without performing recovery. This should be used carefully.
    pub const fn new_unchecked(inner: Authorization, authority: Option<Address>) -> Self {
        Self { inner, authority }
    }

    /// Get the `authority` for the authorization.
    ///
    /// If this is `None`, then the authority could not be recovered.
    pub const fn authority(&self) -> Option<Address> {
        self.authority
    }

    /// Splits the authorization into parts.
    pub const fn into_parts(self) -> (Authorization, Option<Address>) {
        (self.inner, self.authority)
    }
}

impl From<SignedAuthorization> for RecoveredAuthorization {
    fn from(signed_auth: SignedAuthorization) -> Self {
        let authority = signed_auth.recover_authority().ok();
        let (authorization, _) = signed_auth.into_parts();
        Self::new_unchecked(authorization, authority)
    }
}

impl Deref for RecoveredAuthorization {
    type Target = Authorization;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
