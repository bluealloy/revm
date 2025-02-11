use crate::{return_ok, return_revert, Gas, InstructionResult, InterpreterResult};
use revm_primitives::{Address, Bytes, B256, U256};
use std::boxed::Box;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SystemInterruptionInputs {
    pub target_address: Address,
    pub caller: Address,
    pub call_value: U256,
    pub call_id: u32,
    pub code_hash: B256,
    pub input: Bytes,
    pub gas: Gas,
    pub local_gas_limit: u64,
    pub state: u32,
    pub is_create: bool,
    pub is_static: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct SystemInterruptionOutcome {
    pub call_id: u32,
    pub target_address: Address,
    pub caller: Address,
    pub call_value: U256,
    pub is_create: bool,
    pub is_static: bool,
    pub result: InterpreterResult,
    pub exit_code: i32,
    pub gas_remaining: u64,
    pub is_frame: bool,
}

impl SystemInterruptionOutcome {
    pub fn new(inputs: Box<SystemInterruptionInputs>, gas_remaining: u64, is_frame: bool) -> Self {
        Self {
            call_id: inputs.call_id,
            target_address: inputs.target_address,
            caller: inputs.caller,
            call_value: inputs.call_value,
            is_create: inputs.is_create,
            is_static: inputs.is_static,
            result: InterpreterResult {
                result: InstructionResult::Stop,
                output: Default::default(),
                gas: inputs.gas,
            },
            exit_code: 0,
            gas_remaining,
            is_frame,
        }
    }

    fn insert_frame_result(&mut self, result: InterpreterResult) {
        match result.result {
            return_ok!() => {
                let remaining = result.gas.remaining();
                self.result.gas.erase_cost(remaining);
                let refunded = result.gas.refunded();
                self.result.gas.record_refund(refunded);
                self.exit_code = 0;
            }
            return_revert!() => {
                self.result.gas.erase_cost(result.gas.remaining());

                self.exit_code = 1;
            }
            InstructionResult::FatalExternalError => {
                panic!("revm: fatal external error");
            }
            _ => {
                self.exit_code = 2;
            }
        }
        self.result.output = result.output;
    }

    pub fn insert_result(&mut self, result: InterpreterResult) {
        if self.is_frame {
            // frame interruptions are caused by nested CALL/CREATE-like opcodes;
            // it means that they charge cost for the entire gas limit we need to erase
            self.insert_frame_result(result);
        } else {
            self.exit_code = match result.result {
                return_ok!() => 0,
                return_revert!() => 1,
                InstructionResult::FatalExternalError => {
                    panic!("revm: fatal external error");
                }
                _ => 2,
            };
            self.result.result = result.result;
            self.result.output = result.output;
            self.result.gas = result.gas;
        }
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_remaining - self.result.gas.remaining()
    }
}
