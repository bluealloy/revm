use crate::interpreter::Jumps;

use super::ExtBytecode;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
struct ExtBytecodeSerde {
    base: bytecode::Bytecode,
    program_counter: usize,
}

impl Serialize for ExtBytecode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ExtBytecodeSerde {
            base: self.base.clone(),
            program_counter: self.pc(),
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
        } = ExtBytecodeSerde::deserialize(deserializer)?;

        let mut bytecode = Self::new(base);
        bytecode.absolute_jump(program_counter);
        Ok(bytecode)
    }
}
