use crate::{return_ok, return_revert, Gas, InstructionResult, InterpreterResult};
use fluentbase_types::SyscallInvocationParams;
use revm_primitives::{Address, U256};
use std::boxed::Box;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SystemInterruptionInputs {
    pub target_address: Address,
    pub bytecode_address: Address,
    pub caller: Address,
    pub call_value: U256,
    pub call_id: u32,
    pub syscall_params: SyscallInvocationParams,
    pub gas: Gas,
    pub is_create: bool,
    pub is_static: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct SystemInterruptionOutcome {
    pub inputs: Box<SystemInterruptionInputs>,
    pub result: InterpreterResult,
    pub is_frame: bool,
}

impl SystemInterruptionOutcome {
    pub fn new(inputs: Box<SystemInterruptionInputs>, gas_consumed: Gas, is_frame: bool) -> Self {
        Self {
            inputs,
            result: InterpreterResult {
                result: InstructionResult::Stop,
                output: Default::default(),
                gas: gas_consumed,
            },
            is_frame,
        }
    }

    fn insert_frame_result(&mut self, result: InterpreterResult) {
        // for frame result we take gas from result field
        // because it stores information about gas consumed before the call as well
        let mut gas = self.result.gas;
        match result.result {
            return_ok!() => {
                let remaining = result.gas.remaining();
                gas.erase_cost(remaining);
                let refunded = result.gas.refunded();
                gas.record_refund(refunded);
            }
            return_revert!() => {
                gas.erase_cost(result.gas.remaining());
            }
            InstructionResult::FatalExternalError => {
                panic!("revm: fatal external error");
            }
            _ => {}
        }
        self.result.result = result.result;
        self.result.output = result.output;
        // we can rewrite here gas since it's adjusted with the consumed value
        self.result.gas = gas;
    }

    pub fn insert_result(&mut self, result: InterpreterResult) {
        if self.is_frame {
            // frame interruptions are caused by nested CALL/CREATE-like opcodes;
            // it means that they charge cost for the entire gas limit we need to erase
            self.insert_frame_result(result);
        } else {
            self.result.result = result.result;
            self.result.output = result.output;
            let mut gas = self.inputs.gas;
            assert!(
                gas.record_cost(result.gas.spent()),
                "revm: interruption gas overflow"
            );
            gas.record_refund(result.gas.refunded());
            self.result.gas = gas;
        }
    }
}
