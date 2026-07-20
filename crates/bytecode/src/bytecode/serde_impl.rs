//! Serde support for [`Bytecode`].
//!
//! Serialization goes through the private [`BytecodeSerde`] type.
//! It fixes the wire format independently of the in-memory representation.
//! Its shape mirrors the previous `Bytecode` enum, so data serialized before
//! the internal flattening still deserializes.

use super::{Bytecode, BytecodeKind, JumpTable};
use primitives::{Address, Bytes};
use serde::{Deserialize, Serialize};

/// Wire representation of [`Bytecode`].
///
/// Derivable state is not stored: the EIP-7702 variant keeps only the
/// delegated address (the 23-byte designator is rebuilt on deserialization),
/// and the cached code hash is recomputed on demand.
#[derive(Serialize, Deserialize)]
enum BytecodeSerde {
    LegacyAnalyzed {
        bytecode: Bytes,
        original_len: usize,
        jump_table: JumpTable,
    },
    Eip7702 {
        delegated_address: Address,
    },
}

impl Serialize for Bytecode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let repr = match self.kind() {
            BytecodeKind::LegacyAnalyzed => BytecodeSerde::LegacyAnalyzed {
                bytecode: self.0.bytecode.clone(),
                original_len: self.0.original_len,
                jump_table: self.0.jump_table.clone(),
            },
            BytecodeKind::Eip7702 => BytecodeSerde::Eip7702 {
                delegated_address: self.eip7702_address().unwrap(),
            },
        };
        repr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Bytecode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match BytecodeSerde::deserialize(deserializer)? {
            BytecodeSerde::LegacyAnalyzed {
                bytecode,
                original_len,
                ..
            } => {
                // Reject inconsistent input with an error: `slice(..original_len)`
                // below panics on out-of-range, and serde input is untrusted.
                if original_len > bytecode.len() {
                    return Err(serde::de::Error::custom(
                        "original_len is greater than bytecode length",
                    ));
                }
                // Re-analyze from original bytes to ensure padding invariants are
                // satisfied, rather than trusting the serialized form: the stored
                // jump_table is deliberately ignored (it remains in the wire format
                // only for compatibility).
                Ok(Self::new_legacy(bytecode.slice(..original_len)))
            }
            BytecodeSerde::Eip7702 { delegated_address } => {
                Ok(Self::new_eip7702(delegated_address))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::bytes;

    #[test]
    fn test_eip7702_roundtrip() {
        let original = Bytecode::new_eip7702(Address::new([0xAA; 20]));
        let json = serde_json::to_string(&original).unwrap();
        let deser: Bytecode = serde_json::from_str(&json).unwrap();

        assert!(deser.is_eip7702());
        assert_eq!(deser.eip7702_address(), original.eip7702_address());
        assert_eq!(deser, original);
    }

    #[test]
    fn test_legacy_rejects_oversized_original_len() {
        let original = Bytecode::new_legacy(bytes!("6001600155"));
        let mut value = serde_json::to_value(&original).unwrap();
        value["LegacyAnalyzed"]["original_len"] = 1000.into();

        let err = serde_json::from_value::<Bytecode>(value).unwrap_err();
        assert!(err.to_string().contains("original_len"));
    }

    #[test]
    fn test_legacy_ignores_serialized_jump_table() {
        // `5b` is a JUMPDEST opcode; in `605b` the 0x5b is PUSH1 data.
        let original = Bytecode::new_legacy(bytes!("5b"));
        let other = Bytecode::new_legacy(bytes!("605b"));

        let mut value = serde_json::to_value(&original).unwrap();
        value["LegacyAnalyzed"]["jump_table"] =
            serde_json::to_value(&other).unwrap()["LegacyAnalyzed"]["jump_table"].take();

        let deser: Bytecode = serde_json::from_value(value).unwrap();
        assert_eq!(deser, original);
        assert_eq!(deser.legacy_jump_table(), original.legacy_jump_table());
    }

    #[test]
    fn test_legacy_roundtrip() {
        // PUSH1 0x5b, JUMPDEST, STOP: only the second 0x5b is a valid dest.
        let original = Bytecode::new_legacy(bytes!("605b5b00"));
        let json = serde_json::to_string(&original).unwrap();
        let deser: Bytecode = serde_json::from_str(&json).unwrap();

        assert!(deser.is_legacy());
        assert_eq!(deser.legacy_jump_table(), original.legacy_jump_table());
        assert_eq!(deser, original);
    }
}
