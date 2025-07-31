use crate::eip7702::Eip7702DecodeError;
use core::fmt::Debug;
use std::fmt;

/// Bytecode decode errors
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BytecodeDecodeError {
    /// EIP-7702 decode error
    Eip7702(Eip7702DecodeError),
}

impl From<Eip7702DecodeError> for BytecodeDecodeError {
    fn from(error: Eip7702DecodeError) -> Self {
        Self::Eip7702(error)
    }
}

impl core::error::Error for BytecodeDecodeError {}

impl fmt::Display for BytecodeDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eip7702(e) => fmt::Display::fmt(e, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_eip7702_decode_error() {
        // Test conversion from each EIP7702 error variant
        let invalid_length = BytecodeDecodeError::from(Eip7702DecodeError::InvalidLength);
        assert_eq!(
            invalid_length,
            BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidLength)
        );

        let invalid_magic = BytecodeDecodeError::from(Eip7702DecodeError::InvalidMagic);
        assert_eq!(
            invalid_magic,
            BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidMagic)
        );

        let unsupported_version = BytecodeDecodeError::from(Eip7702DecodeError::UnsupportedVersion);
        assert_eq!(
            unsupported_version,
            BytecodeDecodeError::Eip7702(Eip7702DecodeError::UnsupportedVersion)
        );
    }

    #[test]
    fn test_display() {
        // Test Display implementation for each error variant
        let invalid_length = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidLength);
        assert_eq!(format!("{invalid_length}"), "Eip7702 is not 23 bytes long");

        let invalid_magic = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidMagic);
        assert_eq!(
            format!("{invalid_magic}"),
            "Bytecode is not starting with 0xEF01"
        );

        let unsupported_version =
            BytecodeDecodeError::Eip7702(Eip7702DecodeError::UnsupportedVersion);
        assert_eq!(
            format!("{unsupported_version}"),
            "Unsupported Eip7702 version."
        );
    }

    #[test]
    fn test_error_trait() {
        // Test that BytecodeDecodeError implements Error trait
        let error = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidLength);
        // This will fail to compile if Error trait is not implemented
        let _: &dyn core::error::Error = &error;
    }

    #[test]
    fn test_debug_trait() {
        let error = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidMagic);
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("Eip7702"));
        assert!(debug_str.contains("InvalidMagic"));
    }

    #[test]
    fn test_clone() {
        let error = BytecodeDecodeError::Eip7702(Eip7702DecodeError::UnsupportedVersion);
        let cloned = error;
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_copy() {
        let error = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidLength);
        let copied = error; // Copy trait allows this
        assert_eq!(error, copied);
    }

    #[test]
    fn test_eq_and_ne() {
        let error1 = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidLength);
        let error2 = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidLength);
        let error3 = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidMagic);

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let error1 = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidLength);
        let error2 = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidLength);

        let mut hasher1 = DefaultHasher::new();
        error1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        error2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_ord() {
        let error1 = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidLength);
        let error2 = BytecodeDecodeError::Eip7702(Eip7702DecodeError::InvalidMagic);
        let error3 = BytecodeDecodeError::Eip7702(Eip7702DecodeError::UnsupportedVersion);

        // Test Ord trait
        assert!(error1 < error2);
        assert!(error2 < error3);
        assert!(error1 < error3);

        // Test PartialOrd
        assert_eq!(error1.partial_cmp(&error1), Some(std::cmp::Ordering::Equal));
        assert_eq!(error1.partial_cmp(&error2), Some(std::cmp::Ordering::Less));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        use serde::{Deserialize, Serialize};

        // Test that the type implements Serialize and Deserialize traits
        fn assert_serde<T: Serialize + for<'de> Deserialize<'de>>() {}
        assert_serde::<BytecodeDecodeError>();
    }
}
