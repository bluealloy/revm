use crate::{
    Contract, FunctionStack, Gas, InstructionResult, InterpreterAction, SharedMemory, Stack,
};

use super::Interpreter;
use revm_primitives::Bytes;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

impl Serialize for Interpreter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Interpreter", 8)?;
        // Convert the instruction pointer to a usize for serialization
        let program_counter = self.program_counter();
        state.serialize_field("program_counter", &program_counter)?;
        state.serialize_field("gas", &self.gas)?;
        state.serialize_field("contract", &self.contract)?;
        state.serialize_field("instruction_result", &self.instruction_result)?;
        state.serialize_field("bytecode", &self.bytecode)?;
        state.serialize_field("is_eof", &self.is_eof)?;
        state.serialize_field("is_eof_init", &self.is_eof_init)?;
        state.serialize_field("shared_memory", &self.shared_memory)?;
        state.serialize_field("stack", &self.stack)?;
        state.serialize_field("function_stack", &self.function_stack)?;
        state.serialize_field("return_data_buffer", &self.return_data_buffer)?;
        state.serialize_field("is_static", &self.is_static)?;
        state.serialize_field("next_action", &self.next_action)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Interpreter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct InterpreterVisitor;

        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum InterpreterFields {
            ProgramCounter,
            Gas,
            Contract,
            InstructionResult,
            Bytecode,
            IsEof,
            IsEofInit,
            SharedMemory,
            Stack,
            FunctionStack,
            ReturnDataBuffer,
            IsStatic,
            NextAction,
        }

        #[allow(clippy::too_many_arguments)]
        fn rebuild_interp(
            program_counter: isize,
            gas: Gas,
            contract: Contract,
            instruction_result: InstructionResult,
            bytecode: Bytes,
            is_eof: bool,
            is_eof_init: bool,
            shared_memory: SharedMemory,
            stack: Stack,
            function_stack: FunctionStack,
            return_data_buffer: Bytes,
            is_static: bool,
            next_action: InterpreterAction,
        ) -> Result<Interpreter, &'static str> {
            // Reconstruct the instruction pointer from usize
            if program_counter < 0 || program_counter >= bytecode.len() as isize {
                return Err("program_counter index out of range");
            }

            // SAFETY: range of program_counter checked above
            let instruction_pointer = unsafe { bytecode.as_ptr().offset(program_counter) };

            // Construct and return the Interpreter instance
            Ok(Interpreter {
                instruction_pointer,
                gas,
                contract,
                instruction_result,
                bytecode,
                is_eof,
                is_eof_init,
                shared_memory,
                stack,
                function_stack,
                return_data_buffer,
                is_static,
                next_action,
            })
        }

        impl<'de> Visitor<'de> for InterpreterVisitor {
            type Value = Interpreter;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Interpreter")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                macro_rules! extract_field {
                    ($i:ident, $idx:expr) => {
                        let $i = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length($idx, &self))?;
                    };
                }
                extract_field!(instruction_pointer, 0);
                extract_field!(gas, 1);
                extract_field!(contract, 2);
                extract_field!(instruction_result, 3);
                extract_field!(bytecode, 4);
                extract_field!(is_eof, 5);
                extract_field!(is_eof_init, 6);
                extract_field!(shared_memory, 7);
                extract_field!(stack, 8);
                extract_field!(function_stack, 9);
                extract_field!(return_data_buffer, 10);
                extract_field!(is_static, 11);
                extract_field!(next_action, 12);
                rebuild_interp(
                    instruction_pointer,
                    gas,
                    contract,
                    instruction_result,
                    bytecode,
                    is_eof,
                    is_eof_init,
                    shared_memory,
                    stack,
                    function_stack,
                    return_data_buffer,
                    is_static,
                    next_action,
                )
                .map_err(de::Error::custom)
            }

            fn visit_map<V>(self, mut map: V) -> Result<Interpreter, V::Error>
            where
                V: MapAccess<'de>,
            {
                macro_rules! parse_map {
                    ( $(($enum:pat, $var_name:ident)),* ) => {
                        $(
                            let mut $var_name = None;
                        )*
                        while let Some(key) = map.next_key()? {
                            match key {
                                $(
                                    $enum => {
                                        $var_name = Some(map.next_value()?);
                                    }
                                )*
                            }
                        }
                        $(
                            let $var_name = $var_name.ok_or_else(|| de::Error::missing_field(stringify!($var_name)))?;
                        )*
                    };
                }
                parse_map!(
                    (InterpreterFields::ProgramCounter, program_counter),
                    (InterpreterFields::Gas, gas),
                    (InterpreterFields::Contract, contract),
                    (InterpreterFields::InstructionResult, instruction_result),
                    (InterpreterFields::Bytecode, bytecode),
                    (InterpreterFields::IsEof, is_eof),
                    (InterpreterFields::IsEofInit, is_eof_init),
                    (InterpreterFields::SharedMemory, shared_memory),
                    (InterpreterFields::Stack, stack),
                    (InterpreterFields::FunctionStack, function_stack),
                    (InterpreterFields::ReturnDataBuffer, return_data_buffer),
                    (InterpreterFields::IsStatic, is_static),
                    (InterpreterFields::NextAction, next_action)
                );

                rebuild_interp(
                    program_counter,
                    gas,
                    contract,
                    instruction_result,
                    bytecode,
                    is_eof,
                    is_eof_init,
                    shared_memory,
                    stack,
                    function_stack,
                    return_data_buffer,
                    is_static,
                    next_action,
                )
                .map_err(de::Error::custom)
            }
        }

        const FIELDS: &[&str] = &[
            "program_counter",
            "gas",
            "contract",
            "instruction_result",
            "bytecode",
            "is_eof",
            "is_eof_init",
            "shared_memory",
            "stack",
            "function_stack",
            "return_data_buffer",
            "is_static",
            "next_action",
        ];

        deserializer.deserialize_struct("Interpreter", FIELDS, InterpreterVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let interp = Interpreter::new(Contract::default(), u64::MAX, false);
        let serialized = bincode::serialize(&interp).unwrap();
        let de: Interpreter = bincode::deserialize(&serialized).unwrap();
        assert_eq!(interp.program_counter(), de.program_counter());
    }
}
