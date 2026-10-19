use crate::{CreateInputs, Gas, InstructionResult, InterpreterResult};
use primitives::{Address, Bytes};

/// Represents the outcome of a create operation in an interpreter.
///
/// This struct holds the result of the operation along with an optional address.
///
/// It provides methods to determine the next action based on the result of the operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateOutcome {
    /// The result of the interpreter operation
    pub result: InterpreterResult,
    /// An optional address associated with the create operation
    pub address: Option<Address>,
    /// EIP-8037: whether the CREATE opcode charged the conditional
    /// `create_state_gas` on the parent's tracker (the destination did not
    /// exist at access time). When the create fails the parent refunds it.
    pub charged_create_state_gas: bool,
}

impl CreateOutcome {
    /// Constructs a new [`CreateOutcome`].
    ///
    /// # Arguments
    ///
    /// * `result` - An [`InterpreterResult`] representing the result of the interpreter operation.
    /// * `address` - An optional [`Address`] associated with the create operation.
    ///
    /// # Returns
    ///
    /// A new [`CreateOutcome`] instance.
    pub const fn new(result: InterpreterResult, address: Option<Address>) -> Self {
        Self {
            result,
            address,
            charged_create_state_gas: false,
        }
    }

    /// Constructs a new [`CreateOutcome`] for an out-of-gas error.
    ///
    /// # Arguments
    ///
    /// * `gas_limit` - The gas limit that was exceeded.
    ///
    /// # Returns
    ///
    /// A new [`CreateOutcome`] instance with no address.
    pub fn new_oog(gas_limit: u64, reservoir: u64) -> Self {
        Self::new(InterpreterResult::new_oog(gas_limit, reservoir), None)
    }

    /// Constructs a [`CreateOutcome`] that answers the creation described by `inputs` without
    /// running it and at no gas cost, for example a mocked result returned from an inspector's
    /// `create` hook.
    ///
    /// The outcome carries the creation's whole gas limit unspent, the state gas reservoir the
    /// creation inherited (EIP-8037), the given `address` and the creation's
    /// [`charged_create_state_gas`](Self::charged_create_state_gas) flag. They are copied from
    /// `inputs` as given: if an inspector rewrote the gas limit, the reservoir or the flag, the
    /// outcome carries the rewritten values.
    ///
    /// The standard handler settles the outcome exactly like the outcome of a creation that ran,
    /// from the outcome the end hooks leave:
    ///
    /// - It adopts the outcome's reservoir as its own. This is why the outcome must carry the
    ///   reservoir the creation inherited: an outcome built with [`Gas::new`] has none, and the
    ///   caller would lose the reservoir it forwarded.
    /// - The gas limit returns to the caller on success or revert. A halt consumes it.
    /// - When no account existed at the address being created, the caller charged the
    ///   account-creation state gas upfront. If the outcome is not a success, or has no address,
    ///   the caller refunds that charge. If it is a success with an address, the caller keeps it,
    ///   whether or not the answer created the account: a successful answer that creates nothing
    ///   still pays for the account. Keeping the charge is correct when the answer created the
    ///   account itself.
    ///
    /// The constructor changes no state: it creates no account and bumps no nonce. An outcome
    /// that stands for real execution, for example a creation that the inspector ran elsewhere
    /// and merged into the state, should carry the gas of that execution instead, including its
    /// reservoir and state gas charges.
    ///
    /// # Arguments
    ///
    /// * `inputs` - The inputs of the creation that is answered.
    /// * `result` - The result the creation reports.
    /// * `output` - The output data the creation returns.
    /// * `address` - The address of the created contract, if any.
    pub fn new_answer(
        inputs: &CreateInputs,
        result: InstructionResult,
        output: Bytes,
        address: Option<Address>,
    ) -> Self {
        let gas = Gas::new_with_regular_gas_and_reservoir(inputs.gas_limit(), inputs.reservoir());
        Self {
            charged_create_state_gas: inputs.charged_create_state_gas(),
            ..Self::new(InterpreterResult::new(result, output, gas), address)
        }
    }

    /// Retrieves a reference to the [`InstructionResult`] from the [`InterpreterResult`].
    ///
    /// This method provides access to the [`InstructionResult`] which represents the
    /// outcome of the instruction execution.
    ///
    /// It encapsulates the result information such as whether the instruction was
    /// executed successfully, resulted in a revert, or encountered a fatal error.
    ///
    /// # Returns
    ///
    /// A reference to the [`InstructionResult`].
    pub const fn instruction_result(&self) -> &InstructionResult {
        &self.result.result
    }

    /// Retrieves a reference to the output bytes from the [`InterpreterResult`].
    ///
    /// This method returns the output of the interpreted operation.
    ///
    /// The output is typically used when the operation successfully completes and
    /// returns data.
    ///
    /// # Returns
    ///
    /// A reference to the output [`Bytes`].
    pub const fn output(&self) -> &Bytes {
        &self.result.output
    }

    /// Retrieves a reference to the [`Gas`] details from the [`InterpreterResult`].
    ///
    /// This method provides access to the gas details of the operation, which includes
    /// information about gas used, remaining, and refunded.
    ///
    /// It is essential for understanding the gas consumption of the operation.
    ///
    /// # Returns
    ///
    /// A reference to the [`Gas`] details.
    pub const fn gas(&self) -> &Gas {
        &self.result.gas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_interface::CreateScheme;
    use primitives::U256;

    fn create_inputs(reservoir: u64, charged_create_state_gas: bool) -> CreateInputs {
        let mut inputs = CreateInputs::new(
            Address::ZERO,
            CreateScheme::Create,
            U256::ZERO,
            Bytes::new(),
            50_000,
            reservoir,
        );
        inputs.set_charged_create_state_gas(charged_create_state_gas);
        inputs
    }

    #[test]
    fn test_new_answer_carries_the_inherited_reservoir_and_flag() {
        let address = Some(Address::with_last_byte(1));
        let outcome = CreateOutcome::new_answer(
            &create_inputs(7_000, true),
            InstructionResult::Return,
            Bytes::new(),
            address,
        );
        let mut expected = CreateOutcome::new(
            InterpreterResult::new(
                InstructionResult::Return,
                Bytes::new(),
                Gas::new_with_regular_gas_and_reservoir(50_000, 7_000),
            ),
            address,
        );
        expected.charged_create_state_gas = true;
        assert_eq!(outcome, expected);
        assert_eq!(outcome.gas().remaining(), 50_000);
        assert_eq!(outcome.gas().reservoir(), 7_000);
        assert_eq!(outcome.gas().state_gas_spent(), 0);
    }

    #[test]
    fn test_new_answer_without_reservoir_equals_gas_new() {
        let outcome = CreateOutcome::new_answer(
            &create_inputs(0, false),
            InstructionResult::Revert,
            Bytes::from_static(&[1]),
            None,
        );
        let expected = CreateOutcome::new(
            InterpreterResult::new(
                InstructionResult::Revert,
                Bytes::from_static(&[1]),
                Gas::new(50_000),
            ),
            None,
        );
        assert_eq!(outcome, expected);
    }
}
