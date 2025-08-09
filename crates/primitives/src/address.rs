//! Contains the [`AddressOrId`] enum, which is used to represent an address or an id.
use crate::Address;

/// Address id.
pub type AccountId = usize;

/// Address or account id. Id is used for internal account management.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AddressOrId {
    /// An Ethereum address.
    Address(Address),
    /// Id of account, used for internal account management.
    Id(AccountId),
}

impl From<Address> for AddressOrId {
    fn from(address: Address) -> Self {
        Self::Address(address)
    }
}

impl From<AccountId> for AddressOrId {
    fn from(id: AccountId) -> Self {
        Self::Id(id)
    }
}

impl Default for AddressOrId {
    fn default() -> Self {
        Self::Address(Address::default())
    }
}

impl AddressOrId {
    /// Returns true if the address is an Ethereum address.
    #[inline]
    pub fn is_address(&self) -> bool {
        matches!(self, AddressOrId::Address(_))
    }

    /// Returns true if the address is an id.
    #[inline]
    pub fn is_id(&self) -> bool {
        matches!(self, AddressOrId::Id(_))
    }

    /// Returns the address if it is an Ethereum address.
    #[inline]
    pub fn address(&self) -> Option<Address> {
        if let AddressOrId::Address(address) = self {
            Some(*address)
        } else {
            None
        }
    }

    /// Returns the id if it is an id.
    #[inline]
    pub fn as_id(&self) -> Option<AccountId> {
        if let AddressOrId::Id(id) = self {
            Some(*id)
        } else {
            None
        }
    }
}

/// Address and id.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AddressAndId {
    /// Address.
    address: Address,
    /// Id.
    id: AccountId,
}

impl PartialEq<AddressOrId> for AddressAndId {
    fn eq(&self, other: &AddressOrId) -> bool {
        match other {
            AddressOrId::Address(address) => self.address == *address,
            AddressOrId::Id(id) => self.id == *id,
        }
    }
}

impl AddressAndId {
    /// Creates a new address and id.
    #[inline]
    pub fn new(address: Address, id: AccountId) -> Self {
        Self { address, id }
    }

    /// Returns the address.
    #[inline]
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the id.
    #[inline]
    pub fn id(&self) -> AccountId {
        self.id
    }

    /// Converts the address and id to an id.
    pub fn to_id(&self) -> AddressOrId {
        AddressOrId::Id(self.id)
    }

    /// Converts the address and id to an address or id.
    pub fn to_address(&self) -> AddressOrId {
        AddressOrId::Address(self.address)
    }
}

/// Address and optional id.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AddressAndOptionalId {
    /// Address.
    address: Address,
    /// Id.
    id: Option<AccountId>,
}

impl From<Address> for AddressAndOptionalId {
    fn from(address: Address) -> Self {
        Self { address, id: None }
    }
}

impl AddressAndOptionalId {
    /// Creates a new address and optional id.
    #[inline]
    pub fn new(address: Address, id: Option<AccountId>) -> Self {
        Self { address, id }
    }

    /// Converts the address and optional id to an address or id.
    #[inline]
    pub fn to_address_or_id(&self) -> AddressOrId {
        if let Some(id) = self.id {
            AddressOrId::Id(id)
        } else {
            AddressOrId::Address(self.address)
        }
    }
}
