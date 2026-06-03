use context_interface::{
    context::{SStoreResult, StateLoad},
    Host,
};
use primitives::Address;

use crate::{
    InstructionContext as Ictx, InstructionResult, Interpreter, InterpreterTypes as ITy,
    InterpreterTypes,
};

/// Context passed to instruction implementations containing the host and interpreter.
/// This struct provides access to both the host interface for external state operations
/// and the interpreter state for stack, memory, and gas operations.
pub struct InstructionContext<'a, H: ?Sized, ITy: InterpreterTypes> {
    /// Reference to the interpreter containing execution state (stack, memory, gas, etc).
    pub interpreter: &'a mut Interpreter<ITy>,
    /// Reference to the host interface for accessing external blockchain state.
    pub host: &'a mut H,
}

impl<H: ?Sized, IT: InterpreterTypes> std::fmt::Debug for InstructionContext<'_, H, IT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstructionContext")
            .field("host", &"<host>")
            .field("interpreter", &"<interpreter>")
            .finish()
    }
}

/// Result of SSTORE gas-state side effects.
///
/// Implementations of [`GasStateTr`] return this to let opcode-level SSTORE
/// accounting be overridden without knowing how the gas-state backend is stored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GasStateOutcome {
    /// Whether normal SSTORE refund accounting should be skipped.
    pub skip_refund: bool,
    /// Whether opcode-level SSTORE dynamic gas and EIP-8037 state-gas accounting should be skipped.
    ///
    /// When true, the gas-state policy is responsible for any replacement gas accounting.
    pub skip_gas: bool,
}

/// Type-level SSTORE gas-state policy.
///
/// This hook is called after the storage write has been journaled and before
/// the subsequent state-gas/refund accounting. The default policy is a no-op.
pub trait GasStateTr<IT: ITy, H: Host + ?Sized> {
    /// Called after the main SSTORE journal update and before final gas/refund accounting.
    fn sstore_gas_state(
        context: &mut Ictx<'_, H, IT>,
        owner: Address,
        state_load: &StateLoad<SStoreResult>,
    ) -> Result<GasStateOutcome, InstructionResult>;
}

/// No-op SSTORE gas-state policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NoGasState;

impl<IT: ITy, H: Host + ?Sized> GasStateTr<IT, H> for NoGasState {
    #[inline]
    fn sstore_gas_state(
        _context: &mut Ictx<'_, H, IT>,
        _owner: Address,
        _state_load: &StateLoad<SStoreResult>,
    ) -> Result<GasStateOutcome, InstructionResult> {
        Ok(GasStateOutcome::default())
    }
}
