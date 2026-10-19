use crate::{CallInputs, Gas, InstructionResult, InterpreterResult};
use core::ops::Range;
use primitives::{Bytes, Log};
use std::vec::Vec;

/// Represents the outcome of a call operation in a virtual machine.
///
/// This struct encapsulates the result of executing an instruction by an interpreter, including
/// the result itself, gas usage information, and the memory offset where output data is stored.
///
/// # Fields
///
/// * `result` - The result of the interpreter's execution, including output data and gas usage.
/// * `memory_offset` - The range in memory where the output data is located.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallOutcome {
    /// The result of the interpreter's execution, including output data and gas usage
    pub result: InterpreterResult,
    /// The range in memory where the output data is located
    pub memory_offset: Range<usize>,
    /// Flag to indicate if the call is precompile call.
    /// Used by inspector so it can copy the logs for Inspector::logs call.
    pub was_precompile_called: bool,
    /// Precompile call logs. Needs as revert/halt would delete them from Journal.
    /// So they can't be accessed by inspector.
    pub precompile_call_logs: Vec<Log>,
    /// EIP-8037: copied from `CallInputs::charged_new_account_state_gas`. Tells
    /// the parent frame whether `new_account_state_gas` was upfront-charged on
    /// the parent's tracker for this call, so the parent can refund it when
    /// the call reverts/halts.
    pub charged_new_account_state_gas: bool,
}

impl CallOutcome {
    /// Constructs a new [`CallOutcome`].
    ///
    /// Creates an instance of [`CallOutcome`] with the given interpreter result and memory offset.
    ///
    /// # Arguments
    ///
    /// * `result` - The result of the interpreter's execution.
    /// * `memory_offset` - The range in memory indicating where the output data is stored.
    pub const fn new(result: InterpreterResult, memory_offset: Range<usize>) -> Self {
        Self {
            result,
            memory_offset,
            was_precompile_called: false,
            precompile_call_logs: Vec::new(),
            charged_new_account_state_gas: false,
        }
    }

    /// Constructs a new [`CallOutcome`] for an out-of-gas error.
    ///
    /// # Arguments
    ///
    /// * `gas_limit` - The gas limit that was exceeded.
    /// * `memory_offset` - The range in memory indicating where the output data is stored.
    pub fn new_oog(gas_limit: u64, memory_offset: Range<usize>, reservoir: u64) -> Self {
        Self::new(
            InterpreterResult::new_oog(gas_limit, reservoir),
            memory_offset,
        )
    }

    /// Constructs a [`CallOutcome`] that answers the call described by `inputs` without
    /// running it, for example from an inspector's `call` hook.
    ///
    /// The outcome carries the call's whole gas limit unspent, the state gas reservoir the call
    /// inherited (EIP-8037), the call's return memory range and its
    /// [`charged_new_account_state_gas`](Self::charged_new_account_state_gas) flag.
    ///
    /// The caller settles the outcome exactly like the outcome of a call that ran:
    ///
    /// - It adopts the outcome's reservoir as its own. This is why the outcome must carry the
    ///   reservoir the call inherited: an outcome built with [`Gas::new`] has none, and the
    ///   caller would lose the reservoir it forwarded.
    /// - The gas limit returns to the caller on success or revert. A halt consumes it.
    /// - When the call transfers value into an empty account, the caller charged the new-account
    ///   state gas upfront. If the outcome is not a success, the caller refunds that charge. If
    ///   it is a success, the caller keeps it, as for a call that ran and created the account.
    ///   That is correct when the answer moved the value itself, as a mocked call that transfers
    ///   the value does.
    ///
    /// An outcome that stands for real execution, for example a call that the inspector ran
    /// elsewhere and merged into the state, should carry the gas of that execution instead.
    ///
    /// # Arguments
    ///
    /// * `inputs` - The inputs of the call that is answered.
    /// * `result` - The result the call reports.
    /// * `output` - The output data the call returns.
    pub fn new_answer(inputs: &CallInputs, result: InstructionResult, output: Bytes) -> Self {
        let gas = Gas::new_with_regular_gas_and_reservoir(inputs.gas_limit, inputs.reservoir);
        Self {
            charged_new_account_state_gas: inputs.charged_new_account_state_gas,
            ..Self::new(
                InterpreterResult::new(result, output, gas),
                inputs.return_memory_offset.clone(),
            )
        }
    }

    /// Returns a reference to the instruction result.
    ///
    /// Provides access to the result of the executed instruction.
    ///
    /// # Returns
    ///
    /// A reference to the [`InstructionResult`].
    pub const fn instruction_result(&self) -> &InstructionResult {
        &self.result.result
    }

    /// Returns the gas usage information.
    ///
    /// Provides access to the gas usage details of the executed instruction.
    ///
    /// # Returns
    ///
    /// An instance of [`Gas`] representing the gas usage.
    pub const fn gas(&self) -> Gas {
        self.result.gas
    }

    /// Returns a reference to the output data.
    ///
    /// Provides access to the output data generated by the executed instruction.
    ///
    /// # Returns
    ///
    /// A reference to the output data as [`Bytes`].
    pub const fn output(&self) -> &Bytes {
        &self.result.output
    }

    /// Returns the start position of the memory offset.
    ///
    /// Provides the starting index of the memory range where the output data is stored.
    ///
    /// # Returns
    ///
    /// The starting index of the memory offset as [`usize`].
    pub const fn memory_start(&self) -> usize {
        self.memory_offset.start
    }

    /// Returns the length of the memory range.
    ///
    /// Provides the length of the memory range where the output data is stored.
    ///
    /// # Returns
    ///
    /// The length of the memory range as [`usize`].
    pub fn memory_length(&self) -> usize {
        self.memory_offset.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CallInput, CallScheme, CallValue};
    use primitives::{Address, B256, U256};
    use state::Bytecode;

    fn call_inputs(reservoir: u64, charged_new_account_state_gas: bool) -> CallInputs {
        CallInputs {
            input: CallInput::default(),
            return_memory_offset: 4..36,
            gas_limit: 50_000,
            reservoir,
            bytecode_address: Address::ZERO,
            known_bytecode: (B256::ZERO, Bytecode::default()),
            target_address: Address::ZERO,
            caller: Address::ZERO,
            value: CallValue::Transfer(U256::from(1)),
            scheme: CallScheme::Call,
            is_static: false,
            charged_new_account_state_gas,
        }
    }

    #[test]
    fn test_new_answer_carries_the_inherited_reservoir_and_flag() {
        let output = Bytes::from_static(&[1, 2]);
        let outcome = CallOutcome::new_answer(
            &call_inputs(7_000, true),
            InstructionResult::Revert,
            output.clone(),
        );
        let mut expected = CallOutcome::new(
            InterpreterResult::new(
                InstructionResult::Revert,
                output,
                Gas::new_with_regular_gas_and_reservoir(50_000, 7_000),
            ),
            4..36,
        );
        expected.charged_new_account_state_gas = true;
        assert_eq!(outcome, expected);
        assert_eq!(outcome.gas().remaining(), 50_000);
        assert_eq!(outcome.gas().reservoir(), 7_000);
        assert_eq!(outcome.gas().state_gas_spent(), 0);
    }

    #[test]
    fn test_new_answer_without_reservoir_equals_gas_new() {
        let output = Bytes::from_static(&[1, 2]);
        let outcome = CallOutcome::new_answer(
            &call_inputs(0, false),
            InstructionResult::Return,
            output.clone(),
        );
        let expected = CallOutcome::new(
            InterpreterResult::new(InstructionResult::Return, output, Gas::new(50_000)),
            4..36,
        );
        assert_eq!(outcome, expected);
    }
}
