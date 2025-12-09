use super::ExtBytecode;
use crate::interpreter::Jumps;
use primitives::B256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{borrow::Cow, format};

#[derive(Serialize, Deserialize)]
struct ExtBytecodeSerde<'a> {
    base: Cow<'a, bytecode::Bytecode>,
    program_counter: usize,
    bytecode_hash: Option<B256>,
}

impl Serialize for ExtBytecode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ExtBytecodeSerde {
            base: Cow::Borrowed(&self.base),
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
        let mut bytecode = Self::new_with_optional_hash(base.into_owned(), bytecode_hash);
        let len = bytecode.base.bytecode().len();
        if program_counter >= len {
            return Err(serde::de::Error::custom(format!(
                "program counter ({program_counter}) exceeds bytecode length ({len})"
            )));
        }
        bytecode.absolute_jump(program_counter);
        Ok(bytecode)
    }
}
