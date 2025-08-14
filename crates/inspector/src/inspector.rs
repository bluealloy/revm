use auto_impl::auto_impl;
use context::{Database, Journal, JournalEntry};
use interpreter::{
    interpreter::EthInterpreter, CallInputs, CallOutcome, CreateInputs, CreateOutcome, Interpreter,
    InterpreterTypes,
};
use primitives::{Address, Log, U256};
use state::EvmState;

/// EVM hooks into execution.
///
/// This trait is used to enabled tracing of the EVM execution.
///
/// Object that is implemented this trait is used in `InspectorHandler` to trace the EVM execution.
/// And API that allow calling the inspector can be found in [`crate::InspectEvm`] and [`crate::InspectCommitEvm`].
#[auto_impl(&mut, Box)]
pub trait Inspector<CTX, INTR: InterpreterTypes = EthInterpreter> {
    /// Called before the interpreter is initialized.
    ///
    /// If `interp.bytecode.set_action` is set the execution of the interpreter is skipped.
    #[inline]
    fn initialize_interp(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
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
    /// To get the current opcode, use `interp.bytecode.opcode()`.
    #[inline]
    fn step(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let _ = interp;
        let _ = context;
    }

    /// Called after `step` when the instruction has been executed.
    ///
    /// Setting `interp.bytecode.set_action` will result in stopping the execution of the interpreter.
    #[inline]
    fn step_end(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let _ = interp;
        let _ = context;
    }

    /// Called when a log is emitted.
    #[inline]
    fn log(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX, log: Log) {
        let _ = interp;
        let _ = context;
        let _ = log;
    }

    /// Called whenever a call to a contract is about to start.
    ///
    /// Returning `CallOutcome` will override the result of the call.
    #[inline]
    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
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
    fn call_end(&mut self, context: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {
        let _ = context;
        let _ = inputs;
        let _ = outcome;
    }

    /// Called when a contract is about to be created.
    ///
    /// If this returns `Some` then the [CreateOutcome] is used to override the result of the creation.
    ///
    /// If this returns `None` then the creation proceeds as normal.
    #[inline]
    fn create(&mut self, context: &mut CTX, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when a contract has been created.
    ///
    /// Modifying the outcome will alter the result of the create operation.
    #[inline]
    fn create_end(
        &mut self,
        context: &mut CTX,
        inputs: &CreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        let _ = context;
        let _ = inputs;
        let _ = outcome;
    }

    /// Called when a contract has been self-destructed with funds transferred to target.
    #[inline]
    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        let _ = contract;
        let _ = target;
        let _ = value;
    }
}

impl<CTX, INTR: InterpreterTypes, L, R> Inspector<CTX, INTR> for (L, R)
where
    L: Inspector<CTX, INTR>,
    R: Inspector<CTX, INTR>,
{
    fn initialize_interp(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        self.0.initialize_interp(interp, context);
        self.1.initialize_interp(interp, context);
    }

    fn step(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        self.0.step(interp, context);
        self.1.step(interp, context);
    }

    fn step_end(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        self.0.step_end(interp, context);
        self.1.step_end(interp, context);
    }

    fn log(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX, log: Log) {
        self.0.log(interp, context, log.clone());
        self.1.log(interp, context, log);
    }

    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        self.0
            .call(context, inputs)
            .or_else(|| self.1.call(context, inputs))
    }

    fn call_end(&mut self, context: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {
        self.0.call_end(context, inputs, outcome);
        self.1.call_end(context, inputs, outcome);
    }

    fn create(&mut self, context: &mut CTX, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        self.0
            .create(context, inputs)
            .or_else(|| self.1.create(context, inputs))
    }

    fn create_end(
        &mut self,
        context: &mut CTX,
        inputs: &CreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        self.0.create_end(context, inputs, outcome);
        self.1.create_end(context, inputs, outcome);
    }

    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        self.0.selfdestruct(contract, target, value);
        self.1.selfdestruct(contract, target, value);
    }
}

/// Extends the journal with additional methods that are used by the inspector.
#[auto_impl(&mut, Box)]
pub trait JournalExt {
    /// Get all logs from the journal.
    fn logs(&self) -> &[Log];

    /// Get the journal entries that are created from last checkpoint.
    /// new checkpoint is created when sub call is made.
    fn journal(&self) -> &[JournalEntry];

    /// Return the current Journaled state.
    fn evm_state(&self) -> &EvmState;

    /// Return the mutable current Journaled state.
    fn evm_state_mut(&mut self) -> &mut EvmState;
}

impl<DB: Database> JournalExt for Journal<DB> {
    #[inline]
    fn logs(&self) -> &[Log] {
        &self.logs
    }

    #[inline]
    fn journal(&self) -> &[JournalEntry] {
        &self.journal
    }

    #[inline]
    fn evm_state(&self) -> &EvmState {
        &self.state
    }

    #[inline]
    fn evm_state_mut(&mut self) -> &mut EvmState {
        &mut self.state
    }
}
