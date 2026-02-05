use super::{Arc, Bytecode, BytecodeInner, BytecodeKind, JumpTable, OnceLock};
use primitives::{Address, Bytes};
use serde::{Deserialize, Serialize};

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
                jump_table,
            } => Ok(Self(Arc::new(BytecodeInner {
                kind: BytecodeKind::LegacyAnalyzed,
                bytecode,
                original_len,
                jump_table,
                hash: OnceLock::new(),
            }))),
            BytecodeSerde::Eip7702 { delegated_address } => {
                Ok(Self::new_eip7702(delegated_address))
            }
        }
    }
}
