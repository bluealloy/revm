use super::ExtBytecode;
use crate::interpreter::Jumps;
use primitives::B256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
struct ExtBytecodeSerde {
    base: bytecode::Bytecode,
    program_counter: usize,
    bytecode_hash: Option<B256>,
}

impl Serialize for ExtBytecode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ExtBytecodeSerde {
            base: self.base.clone(),
            program_counter: self.pc(),
            bytecode_hash: self.bytecode_hash,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ExtBytecode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ExtBytecodeSerde {
            base,
            program_counter,
            bytecode_hash,
        } = ExtBytecodeSerde::deserialize(deserializer)?;

        let mut bytecode = if let Some(hash) = bytecode_hash {
            Self::new_with_hash(base, hash)
        } else {
            Self::new(base)
        };

        if program_counter >= bytecode.base.bytecode().len() {
            panic!("serde pc: {program_counter} is greater than or equal to bytecode len");
        }
        bytecode.absolute_jump(program_counter);
        Ok(bytecode)
    }
}
