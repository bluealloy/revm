use core::fmt;
use primitives::{b256, bytes, Address, Bytes, B256};

/// Hash of EF44 bytes that is used for EXTCODEHASH when called from legacy bytecode.
pub const OWNABLE_ACCOUNT_MAGIC_HASH: B256 =
    b256!("0x85160e14613bd11c0e87050b7f84bbea3095f7f0ccd58026f217fdff9043c16b");

/// Version Magic in u16 form
pub const OWNABLE_ACCOUNT_MAGIC: u16 = 0xEF44;

/// Magic number in array form
pub static OWNABLE_ACCOUNT_MAGIC_BYTES: Bytes = bytes!("ef44");

/// First version of metadata
pub const OWNABLE_ACCOUNT_VERSION: u8 = 0;

/// Ownable account bytecode representation
///
/// Format consists of:
/// `0xEF44` (MAGIC) + `0x00` (VERSION) + 20 bytes of address.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OwnableAccountBytecode {
    /// Address of the delegated account.
    pub owner_address: Address,
    /// Version. Currently, only version 0 is supported.
    pub version: u8,
    /// Metadata. Extra bytes stored by runtime.
    pub metadata: Bytes,
}

impl OwnableAccountBytecode {
    /// Creates a new metadata representation or returns None if the metadata is invalid.
    #[inline]
    pub fn new_raw(raw: Bytes) -> Result<Self, OwnableAccountDecodeError> {
        if raw.len() < 23 {
            return Err(OwnableAccountDecodeError::InvalidLength);
        } else if !raw.starts_with(&OWNABLE_ACCOUNT_MAGIC_BYTES) {
            return Err(OwnableAccountDecodeError::InvalidMagic);
        }
        // The only supported version is version 0.
        if raw[2] != OWNABLE_ACCOUNT_VERSION {
            return Err(OwnableAccountDecodeError::UnsupportedVersion);
        }
        Ok(Self {
            owner_address: Address::new(raw[3..23].try_into().unwrap()),
            version: OWNABLE_ACCOUNT_VERSION,
            metadata: raw.slice(23..),
        })
    }

    /// Creates a new metadata representation with the given address.
    pub fn new(address: Address, metadata: Bytes) -> Self {
        Self {
            owner_address: address,
            version: OWNABLE_ACCOUNT_VERSION,
            metadata,
        }
    }

    /// Returns the raw metadata with version MAGIC number.
    #[inline]
    pub fn metadata(&self) -> &Bytes {
        &self.metadata
    }

    /// Returns the address of the delegated contract.
    #[inline]
    pub fn owner(&self) -> Address {
        self.owner_address
    }
}

/// Bytecode errors
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OwnableAccountDecodeError {
    /// Invalid length of the raw bytecode
    ///
    /// It should be 23 bytes.
    InvalidLength,
    /// Invalid magic number
    ///
    /// All metadata should start with the magic number 0xEF44.
    InvalidMagic,
    /// Unsupported version
    ///
    /// The only supported version is version 0x00
    UnsupportedVersion,
}

impl fmt::Display for OwnableAccountDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::InvalidLength => "Metadata is not 23 bytes long",
            Self::InvalidMagic => "Metadata is not starting with 0xEF44",
            Self::UnsupportedVersion => "Unsupported Metadata version.",
        };
        f.write_str(s)
    }
}

impl core::error::Error for OwnableAccountDecodeError {}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::keccak256;

    #[test]
    fn magic_bytes_hash_check() {
        let result = keccak256(&OWNABLE_ACCOUNT_MAGIC_BYTES);
        assert_eq!(OWNABLE_ACCOUNT_MAGIC_HASH.as_slice(), result.as_slice());
    }

    #[test]
    fn sanity_decode() {
        let metadata = bytes!("ef44deadbeef");
        assert_eq!(
            OwnableAccountBytecode::new_raw(metadata),
            Err(OwnableAccountDecodeError::InvalidLength)
        );
        let metadata = bytes!("ef4401deadbeef00000000000000000000000000000000");
        assert_eq!(
            OwnableAccountBytecode::new_raw(metadata),
            Err(OwnableAccountDecodeError::UnsupportedVersion)
        );
        let metadata = bytes!("ef4400deadbeef00000000000000000000000000000000");
        let address = metadata[3..].try_into().unwrap();
        assert_eq!(
            OwnableAccountBytecode::new_raw(metadata.clone()),
            Ok(OwnableAccountBytecode {
                owner_address: address,
                version: 0,
                metadata: metadata.slice(23..),
            })
        );
    }

    #[test]
    fn create_metadata_from_address() {
        let address = Address::new([0x01; 20]);
        let bytecode = OwnableAccountBytecode::new(address, bytes!("0102030405"));
        assert_eq!(bytecode.owner_address, address);
        assert_eq!(bytecode.metadata, bytes!("0102030405"));
    }
}
