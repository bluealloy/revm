use crate::{
    inspectors::GasInspector,
    interpreter::{
        opcode, CallInputs, CreateInputs, Interpreter, InterpreterResult, SharedMemory, Stack,
    },
    primitives::{db::Database, hex, Address, U256},
    EvmContext, Inspector,
};
use core::ops::Range;
use serde_json::json;
use std::io::Write;

/// [EIP-3155](https://eips.ethereum.org/EIPS/eip-3155) tracer [Inspector].
pub struct TracerEip3155 {
    output: Box<dyn Write>,
    gas_inspector: GasInspector,

    #[allow(dead_code)]
    trace_mem: bool,
    #[allow(dead_code)]
    trace_return_data: bool,

    stack: Stack,
    pc: usize,
    opcode: u8,
    gas: u64,
    mem_size: usize,
    #[allow(dead_code)]
    memory: Option<SharedMemory>,
    skip: bool,
}

impl TracerEip3155 {
    pub fn new(output: Box<dyn Write>, trace_mem: bool, trace_return_data: bool) -> Self {
        Self {
            output,
            gas_inspector: GasInspector::default(),
            trace_mem,
            trace_return_data,
            stack: Stack::new(),
            pc: 0,
            opcode: 0,
            gas: 0,
            mem_size: 0,
            memory: None,
            skip: false,
        }
    }
}

impl<DB: Database> Inspector<DB> for TracerEip3155 {
    fn initialize_interp(&mut self, interp: &mut Interpreter, context: &mut EvmContext<'_, DB>) {
        self.gas_inspector.initialize_interp(interp, context);
    }

    // get opcode by calling `interp.contract.opcode(interp.program_counter())`.
    // all other information can be obtained from interp.
    fn step(&mut self, interp: &mut Interpreter, context: &mut EvmContext<'_, DB>) {
        self.gas_inspector.step(interp, context);
        self.stack = interp.stack.clone();
        self.pc = interp.program_counter();
        self.opcode = interp.current_opcode();
        self.mem_size = interp.shared_memory.len();
        self.gas = interp.gas.remaining();
    }

    fn step_end(&mut self, interp: &mut Interpreter, context: &mut EvmContext<'_, DB>) {
        self.gas_inspector.step_end(interp, context);
        if self.skip {
            self.skip = false;
            return;
        };

        self.print_log_line(context.journaled_state.depth());
    }

    fn call(
        &mut self,
        _context: &mut EvmContext<'_, DB>,
        _inputs: &mut CallInputs,
    ) -> Option<(InterpreterResult, Range<usize>)> {
        None
    }

    fn call_end(
        &mut self,
        context: &mut EvmContext<'_, DB>,
        result: InterpreterResult,
    ) -> InterpreterResult {
        let result = self.gas_inspector.call_end(context, result);
        if context.journaled_state.depth() == 0 {
            let log_line = json!({
                //stateroot
                "output": format!("0x{}", hex::encode(result.output.as_ref())),
                "gasUsed": format!("0x{:x}", self.gas_inspector.gas_remaining()),
                //time
                //fork
            });

            writeln!(self.output, "{}", serde_json::to_string(&log_line).unwrap())
                .expect("If output fails we can ignore the logging");
        }
        result
    }

    fn create(
        &mut self,
        _context: &mut EvmContext<'_, DB>,
        _inputs: &mut CreateInputs,
    ) -> Option<(InterpreterResult, Option<Address>)> {
        None
    }

    fn create_end(
        &mut self,
        context: &mut EvmContext<'_, DB>,
        result: InterpreterResult,
        address: Option<Address>,
    ) -> (InterpreterResult, Option<Address>) {
        self.gas_inspector.create_end(context, result, address)
    }
}

impl TracerEip3155 {
    fn print_log_line(&mut self, depth: u64) {
        let short_stack: Vec<String> = self.stack.data().iter().map(|&b| short_hex(b)).collect();
        let log_line = json!({
            "depth": depth,
            "pc": self.pc,
            "opName": opcode::OPCODE_JUMPMAP[self.opcode as usize],
            "op": self.opcode,
            "gas": format!("0x{:x}", self.gas),
            "gasCost": format!("0x{:x}", self.gas_inspector.last_gas_cost()),
            //memory?
            "memSize": self.mem_size,
            "stack": short_stack,
            //returnData
            //refund
            //error
            //storage
            //returnStack
        });

        writeln!(self.output, "{}", serde_json::to_string(&log_line).unwrap())
            .expect("If output fails we can ignore the logging");
    }
}

fn short_hex(b: U256) -> String {
    let s = hex::encode(b.to_be_bytes::<32>());
    let s = s.trim_start_matches('0');
    if s.is_empty() {
        "0x0".to_string()
    } else {
        format!("0x{s}")
    }
}
