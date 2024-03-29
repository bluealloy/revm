use crate::{
    inspectors::GasInspector,
    interpreter::{opcode, CallInputs, CallOutcome, Interpreter},
    primitives::{db::Database, hex, HashMap, B256, U256},
    EvmContext, Inspector,
};
use serde::Serialize;
use std::io::Write;

/// [EIP-3155](https://eips.ethereum.org/EIPS/eip-3155) tracer [Inspector].
pub struct TracerEip3155 {
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

impl TracerEip3155 {
    /// Sets the writer to use for the output.
    pub fn set_writer(&mut self, writer: Box<dyn Write>) {
        self.output = writer;
    }
}

impl TracerEip3155 {
    pub fn new(output: Box<dyn Write>) -> Self {
        Self {
            output,
            gas_inspector: GasInspector::default(),
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
}

impl<DB: Database> Inspector<DB> for TracerEip3155 {
    fn initialize_interp(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
        self.gas_inspector.initialize_interp(interp, context);
    }

    fn step(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
        self.gas_inspector.step(interp, context);
        self.stack = interp.stack.data().clone();
        self.memory = if self.include_memory {
            Some(hex::encode_prefixed(interp.shared_memory.context_memory()))
        } else {
            None
        };
        self.pc = interp.program_counter();
        self.opcode = interp.current_opcode();
        self.mem_size = interp.shared_memory.len();
        self.gas = interp.gas.remaining();
        self.refunded = interp.gas.refunded();
    }

    fn step_end(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
        self.gas_inspector.step_end(interp, context);
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
            depth: context.journaled_state.depth(),
            return_data: "0x".to_string(),
            refund: hex_number(self.refunded as u64),
            mem_size: self.mem_size.to_string(),

            op_name: opcode::OPCODE_JUMPMAP[self.opcode as usize],
            error: if !interp.instruction_result.is_ok() {
                Some(format!("{:?}", interp.instruction_result))
            } else {
                None
            },
            memory: self.memory.take(),
            storage: None,
            return_stack: None,
        };
        let _ = self.write_value(&value);
    }

    fn call_end(
        &mut self,
        context: &mut EvmContext<DB>,
        inputs: &CallInputs,
        outcome: CallOutcome,
    ) -> CallOutcome {
        let outcome = self.gas_inspector.call_end(context, inputs, outcome);
        if self.print_summary && context.journaled_state.depth() == 0 {
            let spec_name: &str = context.spec_id().into();
            let value = Summary {
                state_root: B256::ZERO.to_string(),
                output: outcome.result.output.to_string(),
                gas_used: hex_number(inputs.gas_limit - self.gas_inspector.gas_remaining()),
                pass: outcome.result.is_ok(),

                time: None,
                fork: Some(spec_name.to_string()),
            };
            let _ = self.write_value(&value);
        }
        outcome
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
