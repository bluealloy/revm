use crate::interpreter_types::LoopControl as LoopControlTr;
use crate::{Gas, InstructionResult, InterpreterAction};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LoopControl {
    /// The execution control flag.
    ///
    /// If this is not set to [`Continue`][InstructionResult::Continue], the interpreter will stop execution.
    pub instruction_result: InstructionResult,
    /// Actions that the EVM should do.
    ///
    /// Set inside `CALL` or `CREATE` instructions and `RETURN` or `REVERT` instructions.
    ///
    /// Additionally those instructions will set [`InstructionResult`] to
    /// [`CallOrCreate`][InstructionResult::CallOrCreate]/[`Return`][InstructionResult::Return]/[`Revert`][InstructionResult::Revert]
    /// so we know the reason.
    pub next_action: InterpreterAction,
    pub gas: Gas,
}

impl LoopControl {
    pub fn new(gas_limit: u64) -> Self {
        Self {
            instruction_result: InstructionResult::Continue,
            next_action: InterpreterAction::None,
            gas: Gas::new(gas_limit),
        }
    }
}

impl LoopControlTr for LoopControl {
    fn set_instruction_result(&mut self, result: InstructionResult) {
        self.instruction_result = result;
    }

    fn set_next_action(&mut self, action: InterpreterAction, result: InstructionResult) {
        self.next_action = action;
        self.instruction_result = result;
    }

    fn gas(&self) -> &Gas {
        &self.gas
    }

    fn gas_mut(&mut self) -> &mut Gas {
        &mut self.gas
    }

    fn instruction_result(&self) -> InstructionResult {
        self.instruction_result
    }
    fn take_next_action(&mut self) -> InterpreterAction {
        core::mem::take(&mut self.next_action)
    }
}
