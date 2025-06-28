//! Extension traits for `Address` type.

use alloy_primitives::Address;

/// Trait for converting between `Address` and `NamedAddress`.
pub trait AddressTr: Copy {
    /// Converts `self` into an `Address`.
    fn into_address(self) -> Address;

    /// Converts `self` into a `NamedAddress`.
    fn into_named_address(self) -> NamedAddress;

    /// Converts `self` into a `NamedAddress::Caller`.
    fn to_caller(self) -> NamedAddress {
        NamedAddress::Caller(self.into_address())
    }

    /// Converts `self` into a `NamedAddress::Target`.
    fn to_target(self) -> NamedAddress {
        NamedAddress::Target(self.into_address())
    }

    /// Converts `self` into a `NamedAddress::Beneficiary`.
    fn to_beneficiary(self) -> NamedAddress {
        NamedAddress::Beneficiary(self.into_address())
    }
}

impl AddressTr for Address {
    fn into_address(self) -> Address {
        self
    }

    fn into_named_address(self) -> NamedAddress {
        NamedAddress::Unnamed(self)
    }
}

impl AddressTr for NamedAddress {
    fn into_address(self) -> Address {
        match self {
            NamedAddress::Caller(addr)
            | NamedAddress::Target(addr)
            | NamedAddress::Beneficiary(addr)
            | NamedAddress::Unnamed(addr) => addr,
        }
    }

    fn into_named_address(self) -> NamedAddress {
        self
    }
}

/// Address with some known meaning in the context of a transaction or contract execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedAddress {
    /// The address of the caller of the current transaction.
    Caller(Address),
    /// The address of the target of the current transaction.
    Target(Address),
    /// Coinbase address, which is the address that receives the block reward.
    Beneficiary(Address),
    /// An unnamed address.
    Unnamed(Address),
}

impl From<Address> for NamedAddress {
    fn from(address: Address) -> Self {
        NamedAddress::Unnamed(address)
    }
}

impl From<NamedAddress> for Address {
    fn from(named_address: NamedAddress) -> Self {
        match named_address {
            NamedAddress::Caller(addr)
            | NamedAddress::Target(addr)
            | NamedAddress::Beneficiary(addr)
            | NamedAddress::Unnamed(addr) => addr,
        }
    }
}
