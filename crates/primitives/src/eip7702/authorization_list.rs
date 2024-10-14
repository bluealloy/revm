pub use alloy_eip7702::{Authorization, SignedAuthorization};
pub use alloy_primitives::{Parity, Signature};

use crate::Address;
use core::{fmt, ops::Deref};
use std::{boxed::Box, vec::Vec};

use super::SECP256K1N_HALF;

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
    inner: SignedAuthorization,
    authority: Option<Address>,
}

impl RecoveredAuthorization {
    /// Instantiate without performing recovery. This should be used carefully.
    pub const fn new_unchecked(inner: SignedAuthorization, authority: Option<Address>) -> Self {
        Self { inner, authority }
    }

    /// Get the `authority` for the authorization.
    ///
    /// If this is `None`, then the authority could not be recovered.
    pub fn authority(&self) -> Option<Address> {
        let signature = self.inner.signature();

        // Check s-value
        if signature.s() > SECP256K1N_HALF {
            return None;
        }

        // Check y_parity, Parity::Parity means that it was 0 or 1.
        if !matches!(signature.v(), Parity::Parity(_)) {
            return None;
        }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InvalidAuthorization {
    InvalidChainId,
    InvalidYParity,
    Eip2InvalidSValue,
}

impl fmt::Display for InvalidAuthorization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::InvalidChainId => "Invalid chain_id, Expect chain's ID or zero",
            Self::InvalidYParity => "Invalid y_parity, Expect 0 or 1.",
            Self::Eip2InvalidSValue => "Invalid signature s-value.",
        };
        f.write_str(s)
    }
}
