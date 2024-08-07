use crate::{bytes, Address, Bytes};

/// EIP Version Magic in u16 form.
pub const EIP7702_MAGIC: u16 = 0xEF01;

/// EOF magic number in array form.
pub static EIP7702_MAGIC_BYTES: Bytes = bytes!("ef01");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eip7702Bytecode {
    pub delegated_address: Address,
    pub raw: Bytes,
}

impl Eip7702Bytecode {
    /// Creates a new EIP-7702 bytecode or returns None if the raw bytecode is invalid.
    #[inline]
    pub fn new(raw: Bytes) -> Option<Self> {
        if raw.len() != 24 {
            return None;
        }
        if raw.starts_with(&EIP7702_MAGIC_BYTES) {
            return None;
        }
        Some(Self {
            delegated_address: Address::new(raw[4..].try_into().unwrap()),
            raw,
        })
    }

    /// Return the raw bytecode with version MAGIC number.
    #[inline]
    pub fn raw(&self) -> &Bytes {
        &self.raw
    }

    /// Return the address of the delegated contract.
    #[inline]
    pub fn address(&self) -> Address {
        self.delegated_address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity_decode() {
        let raw = bytes!("ef01deadbeef");
        assert_eq!(Eip7702Bytecode::new(raw), None);
        let raw = bytes!("ef01deadbeef00000000000000000000");
        let address = raw[2..].try_into().unwrap();
        assert_eq!(
            Eip7702Bytecode::new(raw.clone()),
            Some(Eip7702Bytecode {
                delegated_address: address,
                raw,
            })
        );
    }
}
