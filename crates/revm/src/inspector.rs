use crate::evm_impl::EVMData;
use crate::interpreter::{CallInputs, CreateInputs, Gas, InstructionResult, Interpreter};
use crate::primitives::{db::Database, Address, Bytes, B256, U256};
use auto_impl::auto_impl;

#[cfg(feature = "std")]
mod customprinter;
#[cfg(all(feature = "std", feature = "serde"))]
mod eip3155;
mod gas;
mod instruction;
mod noop;

pub use instruction::inspector_instruction;
/// [Inspector] implementations.
pub mod inspectors {
    #[cfg(feature = "std")]
    pub use super::customprinter::CustomPrintTracer;
    #[cfg(all(feature = "std", feature = "serde"))]
    pub use super::eip3155::TracerEip3155;
    pub use super::gas::GasInspector;
    pub use super::noop::NoOpInspector;
}

/// EVM [Interpreter] callbacks.
#[auto_impl(&mut, Box)]
pub trait Inspector<DB: Database> {
    /// Called before the interpreter is initialized.
    ///
    /// If anything other than [InstructionResult::Continue] is returned then execution of the interpreter is
    /// skipped.
    #[inline]
    fn initialize_interp(
        &mut self,
        interp: &mut Interpreter<'_>,
        data: &mut EVMData<'_, DB>,
    ) -> InstructionResult {
        let _ = interp;
        let _ = data;
        InstructionResult::Continue
    }

    /// Called on each step of the interpreter.
    ///
    /// Information about the current execution, including the memory, stack and more is available
    /// on `interp` (see [Interpreter]).
    ///
    /// # Example
    ///
    /// To get the current opcode, use `interp.current_opcode()`.
    #[inline]
    fn step(
        &mut self,
        interp: &mut Interpreter<'_>,
        data: &mut EVMData<'_, DB>,
    ) -> InstructionResult {
        let _ = interp;
        let _ = data;
        InstructionResult::Continue
    }

    /// Called when a log is emitted.
    #[inline]
    fn log(
        &mut self,
        evm_data: &mut EVMData<'_, DB>,
        address: &Address,
        topics: &[B256],
        data: &Bytes,
    ) {
        let _ = evm_data;
        let _ = address;
        let _ = topics;
        let _ = data;
    }

    /// Called after `step` when the instruction has been executed.
    ///
    /// InstructionResulting anything other than [InstructionResult::Continue] alters the execution of the interpreter.
    #[inline]
    fn step_end(
        &mut self,
        interp: &mut Interpreter<'_>,
        data: &mut EVMData<'_, DB>,
    ) -> InstructionResult {
        let _ = interp;
        let _ = data;
        InstructionResult::Continue
    }

    /// Called whenever a call to a contract is about to start.
    ///
    /// InstructionResulting anything other than [InstructionResult::Continue] overrides the result of the call.
    #[inline]
    fn call(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &mut CallInputs,
    ) -> (InstructionResult, Gas, Bytes) {
        let _ = data;
        let _ = inputs;
        (InstructionResult::Continue, Gas::new(0), Bytes::new())
    }

    /// Called when a call to a contract has concluded.
    ///
    /// InstructionResulting anything other than the values passed to this function (`(ret, remaining_gas,
    /// out)`) will alter the result of the call.
    #[inline]
    fn call_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &CallInputs,
        remaining_gas: Gas,
        ret: InstructionResult,
        out: Bytes,
    ) -> (InstructionResult, Gas, Bytes) {
        let _ = data;
        let _ = inputs;
        (ret, remaining_gas, out)
    }

    /// Called when a contract is about to be created.
    ///
    /// InstructionResulting anything other than [InstructionResult::Continue] overrides the result of the creation.
    #[inline]
    fn create(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &mut CreateInputs,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        let _ = data;
        let _ = inputs;
        (
            InstructionResult::Continue,
            None,
            Gas::new(0),
            Bytes::default(),
        )
    }

    /// Called when a contract has been created.
    ///
    /// InstructionResulting anything other than the values passed to this function (`(ret, remaining_gas,
    /// address, out)`) will alter the result of the create.
    #[inline]
    fn create_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &CreateInputs,
        ret: InstructionResult,
        address: Option<Address>,
        remaining_gas: Gas,
        out: Bytes,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        let _ = data;
        let _ = inputs;
        (ret, address, remaining_gas, out)
    }

    /// Called when a contract has been self-destructed with funds transferred to target.
    #[inline]
    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        let _ = contract;
        let _ = target;
        let _ = value;
    }
}
