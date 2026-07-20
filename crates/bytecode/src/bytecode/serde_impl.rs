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
