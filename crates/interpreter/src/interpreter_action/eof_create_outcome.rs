use core::ops::Range;

use crate::{Gas, InstructionResult, InterpreterResult};
use revm_primitives::{Address, Bytes};

/// Represents the outcome of a create operation in an interpreter.
///
/// This struct holds the result of the operation along with an optional address.
/// It provides methods to determine the next action based on the result of the operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EOFCreateOutcome {
    /// The result of the interpreter operation.
    pub result: InterpreterResult,
    /// An optional address associated with the create operation.
    pub address: Address,
    /// Return memory range. If EOF creation Reverts it can return bytes from the memory.
    pub return_memory_range: Range<usize>,
}

impl EOFCreateOutcome {
    /// Constructs a new [`EOFCreateOutcome`].
    ///
    /// # Arguments
    ///
    /// * `result` - An `InterpreterResult` representing the result of the interpreter operation.
    /// * `address` - An optional `Address` associated with the create operation.
    /// * `return_memory_range` - The memory range that Revert bytes are going to be written.
    ///
    /// # Returns
    ///
    /// A new [`EOFCreateOutcome`] instance.
    pub fn new(
        result: InterpreterResult,
        address: Address,
        return_memory_range: Range<usize>,
    ) -> Self {
        Self {
            result,
            address,
            return_memory_range,
        }
    }

    /// Retrieves a reference to the [`InstructionResult`] from the [`InterpreterResult`].
    ///
    /// This method provides access to the `InstructionResult` which represents the
    /// outcome of the instruction execution. It encapsulates the result information
    /// such as whether the instruction was executed successfully, resulted in a revert,
    /// or encountered a fatal error.
    ///
    /// # Returns
    ///
    /// A reference to the `InstructionResult`.
    pub fn instruction_result(&self) -> &InstructionResult {
        &self.result.result
    }

    /// Retrieves a reference to the output bytes from the `InterpreterResult`.
    ///
    /// This method returns the output of the interpreted operation. The output is
    /// typically used when the operation successfully completes and returns data.
    ///
    /// # Returns
    ///
    /// A reference to the output `Bytes`.
    pub fn output(&self) -> &Bytes {
        &self.result.output
    }

    /// Retrieves a reference to the `Gas` details from the `InterpreterResult`.
    ///
    /// This method provides access to the gas details of the operation, which includes
    /// information about gas used, remaining, and refunded. It is essential for
    /// understanding the gas consumption of the operation.
    ///
    /// # Returns
    ///
    /// A reference to the `Gas` details.
    pub fn gas(&self) -> &Gas {
        &self.result.gas
    }

    /// Returns the memory range that Revert bytes are going to be written.
    pub fn return_range(&self) -> Range<usize> {
        self.return_memory_range.clone()
    }
}
