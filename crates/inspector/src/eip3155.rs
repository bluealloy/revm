use crate::inspectors::GasInspector;
use crate::Inspector;
use context::{Cfg, ContextTr, JournalTr, Transaction};
use interpreter::{
    interpreter_types::{Jumps, LoopControl, MemoryTr, RuntimeFlag, StackTr, SubRoutineStack},
    CallInputs, CallOutcome, CreateInputs, CreateOutcome, EOFCreateInputs, Interpreter,
    InterpreterResult, InterpreterTypes, Stack,
};
use primitives::{hex, HashMap, B256, U256};
use serde::Serialize;
use state::bytecode::opcode::OpCode;
use std::io::Write;

/// [EIP-3155](https://eips.ethereum.org/EIPS/eip-3155) tracer [Inspector].
pub struct TracerEip3155 {
    output: Box<dyn Write>,
    gas_inspector: GasInspector,
    /// Print summary of the execution.
    print_summary: bool,
    stack: Vec<U256>,
    pc: u64,
    section: Option<u64>,
    function_depth: Option<u64>,
    opcode: u8,
    gas: u64,
    refunded: i64,
    mem_size: usize,
    skip: bool,
    include_memory: bool,
    memory: Option<String>,
}

// # Output
// The CUT MUST output a `json` object for EACH operation.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Output<'a> {
    // Required fields:
    /// Program counter
    pc: u64,
    /// EOF code section
    #[serde(default, skip_serializing_if = "Option::is_none")]
    section: Option<u64>,
    /// OpCode
    op: u8,
    /// Gas left before executing this operation
    #[serde(serialize_with = "serde_hex_u64")]
    gas: u64,
    /// Gas cost of this operation
    #[serde(serialize_with = "serde_hex_u64")]
    gas_cost: u64,
    /// Array of all values on the stack
    stack: &'a [U256],
    /// Depth of the call stack
    depth: u64,
    /// Depth of the EOF function call stack
    #[serde(default, skip_serializing_if = "Option::is_none")]
    function_depth: Option<u64>,
    /// Data returned by the function call
    return_data: &'static str,
    /// Amount of **global** gas refunded
    #[serde(serialize_with = "serde_hex_u64")]
    refund: u64,
    /// Size of memory array
    #[serde(serialize_with = "serde_hex_u64")]
    mem_size: u64,

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
    #[serde(serialize_with = "serde_hex_u64")]
    gas_used: u64,
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

impl TracerEip3155 {
    /// Creates a new EIP-3155 tracer with the given output writer, by first wrapping it in a
    /// [`BufWriter`](std::io::BufWriter).
    pub fn buffered(output: impl Write + 'static) -> Self {
        Self::new(Box::new(std::io::BufWriter::new(output)))
    }

    /// Creates a new EIP-3155 tracer with a stdout output.
    pub fn new_stdout() -> Self {
        Self::buffered(std::io::stdout())
    }

    /// Creates a new EIP-3155 tracer with the given output writer.
    pub fn new(output: Box<dyn Write>) -> Self {
        Self {
            output,
            gas_inspector: GasInspector::new(),
            print_summary: true,
            include_memory: false,
            stack: Default::default(),
            memory: Default::default(),
            pc: 0,
            section: None,
            function_depth: None,
            opcode: 0,
            gas: 0,
            refunded: 0,
            mem_size: 0,
            skip: false,
        }
    }

    /// Sets the writer to use for the output.
    pub fn set_writer(&mut self, writer: Box<dyn Write>) {
        self.output = writer;
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

    /// Resets the tracer to its initial state of [`Self::new`].
    ///
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

    fn print_summary(&mut self, result: &InterpreterResult, context: &mut impl ContextTr) {
        if !self.print_summary {
            return;
        }
        let spec = context.cfg().spec().into();
        let gas_limit = context.tx().gas_limit();
        let value = Summary {
            state_root: B256::ZERO.to_string(),
            output: result.output.to_string(),
            gas_used: gas_limit - self.gas_inspector.gas_remaining(),
            pass: result.is_ok(),
            time: None,
            fork: Some(spec.to_string()),
        };
        let _ = self.write_value(&value);
    }

    fn write_value(&mut self, value: &impl serde::Serialize) -> std::io::Result<()> {
        write_value(&mut *self.output, value)
    }
}

pub trait CloneStack {
    fn clone_into(&self, stack: &mut Vec<U256>);
}

impl CloneStack for Stack {
    fn clone_into(&self, stack: &mut Vec<U256>) {
        stack.extend_from_slice(self.data());
    }
}

impl<CTX, INTR> Inspector<CTX, INTR> for TracerEip3155
where
    CTX: ContextTr,
    INTR: InterpreterTypes<Stack: StackTr + CloneStack>,
{
    fn initialize_interp(&mut self, interp: &mut Interpreter<INTR>, _: &mut CTX) {
        self.gas_inspector.initialize_interp(interp.control.gas());
    }

    fn step(&mut self, interp: &mut Interpreter<INTR>, _: &mut CTX) {
        self.gas_inspector.step(interp.control.gas());
        self.stack.clear();
        interp.stack.clone_into(&mut self.stack);
        self.memory = if self.include_memory {
            Some(hex::encode_prefixed(
                interp.memory.slice(0..interp.memory.size()).as_ref(),
            ))
        } else {
            None
        };
        self.pc = interp.bytecode.pc() as u64;
        self.section = if interp.runtime_flag.is_eof() {
            Some(interp.sub_routine.routine_idx() as u64)
        } else {
            None
        };
        self.function_depth = if interp.runtime_flag.is_eof() {
            Some(interp.sub_routine.len() as u64 + 1)
        } else {
            None
        };
        self.opcode = interp.bytecode.opcode();
        self.mem_size = interp.memory.size();
        self.gas = interp.control.gas().remaining();
        self.refunded = interp.control.gas().refunded();
    }

    fn step_end(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        self.gas_inspector.step_end(interp.control.gas_mut());
        if self.skip {
            self.skip = false;
            return;
        }

        let value = Output {
            pc: self.pc,
            section: self.section,
            op: self.opcode,
            gas: self.gas,
            gas_cost: self.gas_inspector.last_gas_cost(),
            stack: &self.stack,
            depth: context.journal().depth() as u64,
            function_depth: self.function_depth,
            return_data: "0x",
            refund: self.refunded as u64,
            mem_size: self.mem_size as u64,

            op_name: OpCode::new(self.opcode).map(|i| i.as_str()),
            error: (!interp.control.instruction_result().is_ok())
                .then(|| format!("{:?}", interp.control.instruction_result())),
            memory: self.memory.take(),
            storage: None,
            return_stack: None,
        };
        let _ = write_value(&mut self.output, &value);
    }

    fn call_end(&mut self, context: &mut CTX, _: &CallInputs, outcome: &mut CallOutcome) {
        self.gas_inspector.call_end(outcome);

        if context.journal().depth() == 0 {
            self.print_summary(&outcome.result, context);
            let _ = self.output.flush();
            // Clear the state if we are at the top level.
            self.clear();
        }
    }

    fn create_end(&mut self, context: &mut CTX, _: &CreateInputs, outcome: &mut CreateOutcome) {
        self.gas_inspector.create_end(outcome);

        if context.journal().depth() == 0 {
            self.print_summary(&outcome.result, context);
            let _ = self.output.flush();
            // Clear the state if we are at the top level.
            self.clear();
        }
    }

    fn eofcreate_end(
        &mut self,
        context: &mut CTX,
        _: &EOFCreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        self.gas_inspector.create_end(outcome);

        if context.journal().depth() == 0 {
            self.print_summary(&outcome.result, context);
            let _ = self.output.flush();
            // Clear the state if we are at the top level.
            self.clear();
        }
    }
}

fn write_value(
    output: &mut dyn std::io::Write,
    value: &impl serde::Serialize,
) -> std::io::Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    output.write_all(b"\n")
}

fn serde_hex_u64<S: serde::Serializer>(n: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("{:#x}", *n))
}
