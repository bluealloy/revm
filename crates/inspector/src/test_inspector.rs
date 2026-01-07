//! Test inspector for testing EVM execution.

extern crate alloc;

use crate::Inspector;
use alloc::{format, string::String, vec::Vec};
use interpreter::{
    interpreter_types::{Jumps, MemoryTr, StackTr},
    CallInputs, CallOutcome, CreateInputs, CreateOutcome, Interpreter, InterpreterTypes,
};
use primitives::{Address, Log, U256};

/// Interpreter state at a specific point in execution.
#[derive(Debug, Clone)]
pub struct InterpreterState {
    /// Program counter.
    pub pc: usize,
    /// Stack length.
    pub stack_len: usize,
    /// Memory size.
    pub memory_size: usize,
}

/// Step execution record.
#[derive(Debug, Clone)]
pub struct StepRecord {
    /// State before instruction execution.
    pub before: InterpreterState,
    /// State after instruction execution.
    pub after: Option<InterpreterState>,
    /// Opcode name.
    pub opcode_name: String,
}

/// Events captured during EVM execution.
#[derive(Debug, Clone)]
pub enum InspectorEvent {
    /// Execution step.
    Step(StepRecord),
    /// Call operation.
    Call {
        /// Call inputs.
        inputs: CallInputs,
        /// Call outcome.
        outcome: Option<CallOutcome>,
    },
    /// Create operation.
    Create {
        /// Create inputs.
        inputs: CreateInputs,
        /// Create outcome.
        outcome: Option<CreateOutcome>,
    },
    /// Log emission.
    Log(Log),
    /// Selfdestruct operation.
    Selfdestruct {
        /// Contract address.
        address: Address,
        /// Beneficiary address.
        beneficiary: Address,
        /// Value transferred.
        value: U256,
    },
}

/// Test inspector that records execution events.
#[derive(Debug, Default)]
pub struct TestInspector {
    /// Captured events.
    pub events: Vec<InspectorEvent>,
    /// Total step count.
    pub step_count: usize,
    /// Current call depth.
    pub call_depth: usize,
}

impl TestInspector {
    /// Create a new TestInspector.
    pub fn new() -> Self {
        Self::default()
    }

    fn capture_interpreter_state<INTR: InterpreterTypes>(
        interp: &Interpreter<INTR>,
    ) -> InterpreterState
    where
        INTR::Bytecode: Jumps,
        INTR::Stack: StackTr,
        INTR::Memory: MemoryTr,
    {
        InterpreterState {
            pc: interp.bytecode.pc(),
            stack_len: interp.stack.len(),
            memory_size: interp.memory.size(),
        }
    }

    /// Get all captured events.
    pub fn get_events(&self) -> Vec<InspectorEvent> {
        self.events.clone()
    }

    /// Get the total step count.
    pub fn get_step_count(&self) -> usize {
        self.step_count
    }
}

impl<CTX, INTR> Inspector<CTX, INTR> for TestInspector
where
    INTR: InterpreterTypes,
    INTR::Bytecode: Jumps,
    INTR::Stack: StackTr,
    INTR::Memory: MemoryTr,
{
    fn step(&mut self, interp: &mut Interpreter<INTR>, _context: &mut CTX) {
        self.step_count += 1;

        let state = Self::capture_interpreter_state(interp);
        let opcode = interp.bytecode.opcode();
        let opcode_name = if let Some(op) = state::bytecode::opcode::OpCode::new(opcode) {
            format!("{op}")
        } else {
            format!("Unknown(0x{opcode:02x})")
        };

        self.events.push(InspectorEvent::Step(StepRecord {
            before: state,
            after: None,
            opcode_name,
        }));
    }

    fn step_end(&mut self, interp: &mut Interpreter<INTR>, _context: &mut CTX) {
        let state = Self::capture_interpreter_state(interp);

        if let Some(InspectorEvent::Step(record)) = self.events.last_mut() {
            record.after = Some(state);
        }
    }

    fn log(&mut self, _ctx: &mut CTX, log: Log) {
        self.events.push(InspectorEvent::Log(log));
    }

    fn call(&mut self, _ctx: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        self.call_depth += 1;
        self.events.push(InspectorEvent::Call {
            inputs: inputs.clone(),
            outcome: None,
        });
        None
    }

    fn call_end(&mut self, _ctx: &mut CTX, _inputs: &CallInputs, outcome: &mut CallOutcome) {
        self.call_depth -= 1;
        if let Some(InspectorEvent::Call {
            outcome: ref mut out,
            ..
        }) = self
            .events
            .iter_mut()
            .rev()
            .find(|e| matches!(e, InspectorEvent::Call { outcome: None, .. }))
        {
            *out = Some(outcome.clone());
        }
    }

    fn create(&mut self, _ctx: &mut CTX, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        self.events.push(InspectorEvent::Create {
            inputs: inputs.clone(),
            outcome: None,
        });
        None
    }

    fn create_end(&mut self, _ctx: &mut CTX, _inputs: &CreateInputs, outcome: &mut CreateOutcome) {
        if let Some(InspectorEvent::Create {
            outcome: ref mut out,
            ..
        }) = self
            .events
            .iter_mut()
            .rev()
            .find(|e| matches!(e, InspectorEvent::Create { outcome: None, .. }))
        {
            *out = Some(outcome.clone());
        }
    }

    fn selfdestruct(&mut self, contract: Address, beneficiary: Address, value: U256) {
        self.events.push(InspectorEvent::Selfdestruct {
            address: contract,
            beneficiary,
            value,
        });
    }
}

/// Default tests for EVM implementations.
#[cfg(feature = "std")]
pub mod default_tests {
    use super::*;
    use alloc::{string::ToString, vec, vec::Vec};
    use primitives::Bytes;
    use state::bytecode::opcode;

    /// Run default test suite on an EVM implementation.
    /// The execute function should set up the EVM, run the bytecode, and return the TestInspector.
    pub fn run_tests<F>(mut execute: F) -> Result<(), Vec<(&'static str, String)>>
    where
        F: FnMut(Bytes) -> Result<TestInspector, String>,
    {
        let mut failures = Vec::new();

        // Test basic stack operations: PUSH, ADD, MSTORE
        let stack_test = Bytes::from(vec![
            opcode::PUSH1,
            0x42,
            opcode::PUSH1,
            0x10,
            opcode::ADD,
            opcode::PUSH1,
            0x00,
            opcode::MSTORE,
            opcode::STOP,
        ]);

        match execute(stack_test) {
            Ok(inspector) => {
                if inspector.step_count < 5 {
                    failures.push(("stack_operations", "Not enough steps recorded".to_string()));
                }
            }
            Err(e) => failures.push(("stack_operations", e)),
        }

        // Test JUMP control flow
        let jump_test = Bytes::from(vec![
            opcode::PUSH1,
            0x05,
            opcode::JUMP,
            opcode::INVALID,
            opcode::INVALID,
            opcode::JUMPDEST,
            opcode::STOP,
        ]);

        match execute(jump_test) {
            Ok(inspector) => {
                let has_jump = inspector
                    .events
                    .iter()
                    .any(|e| matches!(e, InspectorEvent::Step(s) if s.opcode_name == "JUMP"));
                if !has_jump {
                    failures.push(("jump", "JUMP not recorded".to_string()));
                }
            }
            Err(e) => failures.push(("jump", e)),
        }

        // Test LOG0
        let log_test = Bytes::from(vec![
            opcode::PUSH1,
            0x20,
            opcode::PUSH1,
            0x00,
            opcode::LOG0,
            opcode::STOP,
        ]);

        match execute(log_test) {
            Ok(inspector) => {
                let has_log = inspector
                    .events
                    .iter()
                    .any(|e| matches!(e, InspectorEvent::Log(_)));
                if !has_log {
                    failures.push(("log", "LOG0 not recorded".to_string()));
                }
            }
            Err(e) => failures.push(("log", e)),
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}
