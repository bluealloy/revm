use fluentbase_sdk::{Address, SyscallInvocationParams};
use interpreter::{return_ok, return_revert, Gas, InstructionResult, InterpreterResult};

///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemInterruptionInputs {
    ///
    pub call_id: u32,
    ///
    pub syscall_params: SyscallInvocationParams,
    ///
    pub gas: Gas,
    ///
    pub is_create: bool,
    ///
    pub is_static: bool,
    ///
    pub is_gas_free: bool,
}

///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemInterruptionOutcome {
    ///
    pub inputs: Box<SystemInterruptionInputs>,
    ///
    pub result: InterpreterResult,
    ///
    pub is_frame: bool,
}

impl SystemInterruptionOutcome {
    ///
    pub(crate) fn insert_result(
        &mut self,
        mut result: InterpreterResult,
        created_address: Option<Address>,
    ) {
        // for the frame result we take gas from the result field
        // because it stores information about gas consumed before the call as well
        let mut gas = self.result.gas;
        match result.result {
            return_ok!() => {
                let remaining = result.gas.remaining();
                gas.erase_cost(remaining);
                let refunded = result.gas.refunded();
                gas.record_refund(refunded);
                // for CREATE/CREATE2 calls, we need to write the created address into output
                if let Some(created_address) = created_address {
                    result.output = created_address.into_array().into();
                }
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
}
