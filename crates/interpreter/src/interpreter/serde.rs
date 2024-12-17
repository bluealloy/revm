// use super::{subroutine_stack::SubRoutineImpl, Interpreter};
// use crate::{Contract, Gas, InstructionResult, InterpreterAction, SharedMemory, Stack};
// use primitives::Bytes;
// use serde::{Deserialize, Deserializer, Serialize, Serializer};
// use specification::hardfork::SpecId;

// #[derive(Serialize)]
// struct InterpreterSerde<'a> {
//     program_counter: usize,

//     gas: &'a Gas,
//     contract: &'a Contract,
//     instruction_result: InstructionResult,
//     bytecode: &'a Bytes,
//     is_eof: bool,
//     is_eof_init: bool,
//     shared_memory: &'a SharedMemory,
//     stack: &'a Stack,
//     function_stack: &'a SubRoutineImpl,
//     return_data_buffer: &'a Bytes,
//     is_static: bool,
//     next_action: &'a InterpreterAction,
//     spec_id: SpecId,
// }

// #[derive(Deserialize)]
// struct InterpreterDe {
//     program_counter: usize,

//     gas: Gas,
//     contract: Contract,
//     instruction_result: InstructionResult,
//     bytecode: Bytes,
//     is_eof: bool,
//     is_eof_init: bool,
//     shared_memory: SharedMemory,
//     stack: Stack,
//     function_stack: SubRoutineImpl,
//     return_data_buffer: Bytes,
//     is_static: bool,
//     next_action: InterpreterAction,
//     spec_id: SpecId,
// }

// impl Serialize for Interpreter {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         InterpreterSerde {
//             program_counter: self.bytecode().pc(),
//             gas: &self.gas,
//             contract: &self.contract,
//             instruction_result: self.instruction_result,
//             bytecode: &self.bytecode,
//             is_eof: self.is_eof,
//             is_eof_init: self.is_eof_init,
//             shared_memory: &self.shared_memory,
//             stack: &self.stack,
//             function_stack: &self.function_stack,
//             return_data_buffer: &self.return_data_buffer,
//             is_static: self.is_static,
//             next_action: &self.next_action,
//             spec_id: self.spec_id,
//         }
//         .serialize(serializer)
//     }
// }

// impl<'de> Deserialize<'de> for Interpreter {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let InterpreterDe {
//             program_counter,
//             gas,
//             contract,
//             instruction_result,
//             bytecode,
//             is_eof,
//             is_eof_init,
//             shared_memory,
//             stack,
//             function_stack,
//             return_data_buffer,
//             is_static,
//             next_action,
//             spec_id,
//         } = InterpreterDe::deserialize(deserializer)?;

//         // Reconstruct the instruction pointer from usize
//         if program_counter >= bytecode.len() {
//             return Err(serde::de::Error::custom("program_counter out of bounds"));
//         }

//         // SAFETY: range of program_counter checked above
//         let instruction_pointer = unsafe { bytecode.as_ptr().add(program_counter) };

//         Ok(Interpreter {
//             instruction_pointer,
//             gas,
//             contract,
//             instruction_result,
//             bytecode,
//             is_eof,
//             is_eof_init,
//             shared_memory,
//             stack,
//             function_stack,
//             return_data_buffer,
//             is_static,
//             next_action,
//             spec_id,
//         })
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_serde() {
//         let interp = Interpreter::new(Contract::default(), u64::MAX, false);
//         let serialized = bincode::serialize(&interp).unwrap();
//         let de: Interpreter = bincode::deserialize(&serialized).unwrap();
//         assert_eq!(interp.program_counter(), de.program_counter());
//     }
// }
