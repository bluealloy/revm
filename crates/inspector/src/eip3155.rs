use crate::{inspectors::GasInspector, Inspector};
use derive_where::derive_where;
use revm::{
    bytecode::opcode::OpCode,
    context::Cfg,
    context_interface::{CfgGetter, Journal, JournalStateGetter, Transaction, TransactionGetter},
    interpreter::{
        interpreter_types::{Jumps, LoopControl, MemoryTrait, StackTrait},
        CallInputs, CallOutcome, CreateInputs, CreateOutcome, Interpreter, InterpreterResult,
        InterpreterTypes, Stack,
    },
    primitives::{hex, HashMap, B256, U256},
};
use serde::Serialize;
use std::io::Write;

/// [EIP-3155](https://eips.ethereum.org/EIPS/eip-3155) tracer [Inspector].
#[derive_where(Debug; CTX, INTR)]
pub struct TracerEip3155<CTX, INTR> {
    #[derive_where(skip)]
    output: Box<dyn Write>,
    gas_inspector: GasInspector,
    /// Print summary of the execution.
    print_summary: bool,
    stack: Vec<U256>,
    pc: usize,
    opcode: u8,
    gas: u64,
    refunded: i64,
    mem_size: usize,
    skip: bool,
    include_memory: bool,
    memory: Option<String>,
    _phantom: std::marker::PhantomData<(CTX, INTR)>,
}

// # Output
// The CUT MUST output a `json` object for EACH operation.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Output {
    // Required fields:
    /// Program counter
    pc: u64,
    /// OpCode
    op: u8,
    /// Gas left before executing this operation
    gas: String,
    /// Gas cost of this operation
    gas_cost: String,
    /// Array of all values on the stack
    stack: Vec<String>,
    /// Depth of the call stack
    depth: u64,
    /// Data returned by the function call
    return_data: String,
    /// Amount of **global** gas refunded
    refund: String,
    /// Size of memory array
    mem_size: String,

    // Optional fields:
    /// Name of the operation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    op_name: Option<&'static str>,
    /// Description of an error (should contain revert reason if supported)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    /// Array of all allocated values
    #[serde(default, skip_serializing_if = "Option::is_none")]
    memory: Option<String>,
    /// Array of all stored values
    #[serde(default, skip_serializing_if = "Option::is_none")]
    storage: Option<HashMap<String, String>>,
    /// Array of values, Stack of the called function
    #[serde(default, skip_serializing_if = "Option::is_none")]
    return_stack: Option<Vec<String>>,
}

// # Summary and error handling
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Summary {
    // Required fields:
    /// Root of the state trie after executing the transaction
    state_root: String,
    /// Return values of the function
    output: String,
    /// All gas used by the transaction
    gas_used: String,
    /// Bool whether transaction was executed successfully
    pass: bool,

    // Optional fields:
    /// Time in nanoseconds needed to execute the transaction
    #[serde(default, skip_serializing_if = "Option::is_none")]
    time: Option<u128>,
    /// Name of the fork rules used for execution
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fork: Option<String>,
}

impl<CTX, INTR> TracerEip3155<CTX, INTR>
where
    CTX: CfgGetter + TransactionGetter,
    INTR:,
{
    /// Sets the writer to use for the output.
    pub fn set_writer(&mut self, writer: Box<dyn Write>) {
        self.output = writer;
    }

    /// Resets the Tracer to its initial state of [Self::new].
    /// This makes the inspector ready to be used again.
    pub fn clear(&mut self) {
        let Self {
            gas_inspector,
            stack,
            pc,
            opcode,
            gas,
            refunded,
            mem_size,
            skip,
            ..
        } = self;
        *gas_inspector = GasInspector::new();
        stack.clear();
        *pc = 0;
        *opcode = 0;
        *gas = 0;
        *refunded = 0;
        *mem_size = 0;
        *skip = false;
    }

    pub fn new(output: Box<dyn Write>) -> Self {
        Self {
            output,
            gas_inspector: GasInspector::new(),
            print_summary: true,
            include_memory: false,
            stack: Default::default(),
            memory: Default::default(),
            pc: 0,
            opcode: 0,
            gas: 0,
            refunded: 0,
            mem_size: 0,
            skip: false,
            _phantom: Default::default(),
        }
    }

    /// Don't include a summary at the end of the trace
    pub fn without_summary(mut self) -> Self {
        self.print_summary = false;
        self
    }

    /// Include a memory field for each step. This significantly increases processing time and output size.
    pub fn with_memory(mut self) -> Self {
        self.include_memory = true;
        self
    }

    fn write_value(&mut self, value: &impl serde::Serialize) -> std::io::Result<()> {
        serde_json::to_writer(&mut *self.output, value)?;
        self.output.write_all(b"\n")?;
        self.output.flush()
    }

    fn print_summary(&mut self, result: &InterpreterResult, context: &mut CTX) {
        if self.print_summary {
            let spec = context.cfg().spec().into();
            let gas_limit = context.tx().common_fields().gas_limit();
            let value = Summary {
                state_root: B256::ZERO.to_string(),
                output: result.output.to_string(),
                gas_used: hex_number(gas_limit - self.gas_inspector.gas_remaining()),
                pass: result.is_ok(),
                time: None,
                fork: Some(spec.to_string()),
            };
            let _ = self.write_value(&value);
        }
    }
}

pub trait CloneStack {
    fn clone_from(&self) -> Vec<U256>;
}

impl CloneStack for Stack {
    fn clone_from(&self) -> Vec<U256> {
        self.data().to_vec()
    }
}

impl<CTX, INTR> Inspector for TracerEip3155<CTX, INTR>
where
    CTX: CfgGetter + TransactionGetter + JournalStateGetter,
    INTR: InterpreterTypes<Stack: StackTrait + CloneStack>,
{
    type Context = CTX;
    type InterpreterTypes = INTR;

    fn initialize_interp(&mut self, interp: &mut Interpreter<INTR>, _: &mut CTX) {
        self.gas_inspector.initialize_interp(interp.control.gas());
    }

    fn step(&mut self, interp: &mut Interpreter<INTR>, _: &mut CTX) {
        self.gas_inspector.step(interp.control.gas());
        self.stack = interp.stack.clone_from();
        self.memory = if self.include_memory {
            Some(hex::encode_prefixed(
                interp.memory.slice(0..usize::MAX).as_ref(),
            ))
        } else {
            None
        };
        self.pc = interp.bytecode.pc();
        self.opcode = interp.bytecode.opcode();
        self.mem_size = interp.memory.size();
        self.gas = interp.control.gas().remaining();
        self.refunded = interp.control.gas().refunded();
    }

    fn step_end(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        self.gas_inspector.step_end(interp.control.gas());
        if self.skip {
            self.skip = false;
            return;
        }

        let value = Output {
            pc: self.pc as u64,
            op: self.opcode,
            gas: hex_number(self.gas),
            gas_cost: hex_number(self.gas_inspector.last_gas_cost()),
            stack: self.stack.iter().map(hex_number_u256).collect(),
            depth: context.journal().depth() as u64,
            return_data: "0x".to_string(),
            refund: hex_number(self.refunded as u64),
            mem_size: self.mem_size.to_string(),

            op_name: OpCode::new(self.opcode).map(|i| i.as_str()),
            error: if !interp.control.instruction_result().is_ok() {
                Some(format!("{:?}", interp.control.instruction_result()))
            } else {
                None
            },
            memory: self.memory.take(),
            storage: None,
            return_stack: None,
        };
        let _ = self.write_value(&value);
    }

    fn call_end(&mut self, context: &mut CTX, _: &CallInputs, outcome: &mut CallOutcome) {
        self.gas_inspector.call_end(outcome);

        if context.journal().depth() == 0 {
            self.print_summary(&outcome.result, context);
            // Clear the state if we are at the top level
            self.clear();
        }
    }

    fn create_end(&mut self, context: &mut CTX, _: &CreateInputs, outcome: &mut CreateOutcome) {
        self.gas_inspector.create_end(outcome);

        if context.journal().depth() == 0 {
            self.print_summary(&outcome.result, context);

            // Clear the state if we are at the top level
            self.clear();
        }
    }
}

fn hex_number(uint: u64) -> String {
    format!("0x{uint:x}")
}

fn hex_number_u256(b: &U256) -> String {
    let s = hex::encode(b.to_be_bytes::<32>());
    let s = s.trim_start_matches('0');
    if s.is_empty() {
        "0x0".to_string()
    } else {
        format!("0x{s}")
    }
}
