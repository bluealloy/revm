#[cfg(feature = "std")]
mod customprinter;
#[cfg(all(feature = "std", feature = "serde-json"))]
mod eip3155;
mod gas;
mod handler_register;
mod noop;

pub use handler_register::{inspector_handle_register, GetInspector};

use crate::{
    interpreter::{
        CallInputs, CallOutcome, CreateInputs, CreateOutcome, EOFCreateInputs, Interpreter,
    },
    primitives::{Address, Log, U256},
    EvmContext, EvmWiring,
};
use auto_impl::auto_impl;

/// [Inspector] implementations.
pub mod inspectors {
    #[cfg(feature = "std")]
    pub use super::customprinter::CustomPrintTracer;
    #[cfg(all(feature = "std", feature = "serde-json"))]
    pub use super::eip3155::TracerEip3155;
    pub use super::gas::GasInspector;
    pub use super::noop::NoOpInspector;
}

/// EVM [Interpreter] callbacks.
#[auto_impl(&mut, Box)]
pub trait Inspector<EvmWiringT: EvmWiring> {
    /// Called before the interpreter is initialized.
    ///
    /// If `interp.instruction_result` is set to anything other than [crate::interpreter::InstructionResult::Continue] then the execution of the interpreter
    /// is skipped.
    #[inline]
    fn initialize_interp(
        &mut self,
        interp: &mut Interpreter,
        context: &mut EvmContext<EvmWiringT>,
    ) {
        let _ = interp;
        let _ = context;
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
    fn step(&mut self, interp: &mut Interpreter, context: &mut EvmContext<EvmWiringT>) {
        let _ = interp;
        let _ = context;
    }

    /// Called after `step` when the instruction has been executed.
    ///
    /// Setting `interp.instruction_result` to anything other than [crate::interpreter::InstructionResult::Continue] alters the execution
    /// of the interpreter.
    #[inline]
    fn step_end(&mut self, interp: &mut Interpreter, context: &mut EvmContext<EvmWiringT>) {
        let _ = interp;
        let _ = context;
    }

    /// Called when a log is emitted.
    #[inline]
    fn log(&mut self, interp: &mut Interpreter, context: &mut EvmContext<EvmWiringT>, log: &Log) {
        let _ = interp;
        let _ = context;
        let _ = log;
    }

    /// Called whenever a call to a contract is about to start.
    ///
    /// InstructionResulting anything other than [crate::interpreter::InstructionResult::Continue] overrides the result of the call.
    #[inline]
    fn call(
        &mut self,
        context: &mut EvmContext<EvmWiringT>,
        inputs: &mut CallInputs,
    ) -> Option<CallOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when a call to a contract has concluded.
    ///
    /// The returned [CallOutcome] is used as the result of the call.
    ///
    /// This allows the inspector to modify the given `result` before returning it.
    #[inline]
    fn call_end(
        &mut self,
        context: &mut EvmContext<EvmWiringT>,
        inputs: &CallInputs,
        outcome: CallOutcome,
    ) -> CallOutcome {
        let _ = context;
        let _ = inputs;
        outcome
    }

    /// Called when a contract is about to be created.
    ///
    /// If this returns `Some` then the [CreateOutcome] is used to override the result of the creation.
    ///
    /// If this returns `None` then the creation proceeds as normal.
    #[inline]
    fn create(
        &mut self,
        context: &mut EvmContext<EvmWiringT>,
        inputs: &mut CreateInputs,
    ) -> Option<CreateOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when a contract has been created.
    ///
    /// InstructionResulting anything other than the values passed to this function (`(ret, remaining_gas,
    /// address, out)`) will alter the result of the create.
    #[inline]
    fn create_end(
        &mut self,
        context: &mut EvmContext<EvmWiringT>,
        inputs: &CreateInputs,
        outcome: CreateOutcome,
    ) -> CreateOutcome {
        let _ = context;
        let _ = inputs;
        outcome
    }

    /// Called when EOF creating is called.
    ///
    /// This can happen from create TX or from EOFCREATE opcode.
    fn eofcreate(
        &mut self,
        context: &mut EvmContext<EvmWiringT>,
        inputs: &mut EOFCreateInputs,
    ) -> Option<CreateOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when eof creating has ended.
    fn eofcreate_end(
        &mut self,
        context: &mut EvmContext<EvmWiringT>,
        inputs: &EOFCreateInputs,
        outcome: CreateOutcome,
    ) -> CreateOutcome {
        let _ = context;
        let _ = inputs;
        outcome
    }

    /// Called when a contract has been self-destructed with funds transferred to target.
    #[inline]
    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        let _ = contract;
        let _ = target;
        let _ = value;
    }
}
