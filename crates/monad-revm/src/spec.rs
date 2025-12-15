//! Contains the `[MonadSpecId]` type and its implementation.
use core::str::FromStr;
use revm::primitives::hardfork::{name as eth_name, SpecId, UnknownHardfork};

/// Monad spec id.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
pub enum MonadSpecId {
    /// Monad launch spec id.
    #[default]
    Monad = 100,
}

impl MonadSpecId {
    /// Converts the [`MonadSpecId`] into a [`SpecId`].
    pub const fn into_eth_spec(self) -> SpecId {
        match self {
            Self::Monad => SpecId::PRAGUE,
        }
    }

    /// Checks if the [`MonadSpecId`] is enabled in the other [`MonadSpecId`].
    pub const fn is_enabled_in(self, other: MonadSpecId) -> bool {
        other as u8 <= self as u8
    }
}

impl From<MonadSpecId> for SpecId {
    fn from(spec: MonadSpecId) -> Self {
        spec.into_eth_spec()
    }
}

impl FromStr for MonadSpecId {
    type Err = UnknownHardfork;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            name::MONAD => Ok(MonadSpecId::Monad),
            _ => Err(UnknownHardfork),
        }
    }
}

impl From<MonadSpecId> for &'static str {
    fn from(spec_id: MonadSpecId) -> Self {
        match spec_id {
            MonadSpecId::Monad => name::MONAD,
        }
    }
}

/// String identifiers for Monad hardforks
pub mod name {
    /// Mainnet launch spec name.
    pub const MONAD: &str = "Monad";
}
