#![allow(dead_code)]

use crate::{inspectors::GasInspector, Database, EvmContext, Inspector};
use core::ops::AddAssign;
use revm_interpreter::Interpreter;
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct OpcodeCounterInspector {
    gas_inspector: GasInspector,
    opcode_count: BTreeMap<u8, usize>,
    total_opcode_count: usize,
    opcode_cost: BTreeMap<u8, u64>,
}

impl OpcodeCounterInspector {
    fn weighted_average_cost(&self) -> u64 {
        let mut sum = 0;
        for (opcode, count) in self.opcode_count.iter() {
            let gas_cost = self.opcode_cost.get(opcode).unwrap();
            sum += gas_cost * (*count as u64 / self.total_opcode_count as u64);
        }
        sum
    }
}

impl<DB: Database> Inspector<DB> for OpcodeCounterInspector {
    fn initialize_interp(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
        self.gas_inspector.initialize_interp(interp, context);
    }

    fn step(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
        self.gas_inspector.step(interp, context);
    }

    fn step_end(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
        self.gas_inspector.step_end(interp, context);
        // increase opcode count
        let opcode = interp.current_opcode();
        if self.opcode_count.contains_key(&opcode) {
            self.opcode_count.get_mut(&opcode).unwrap().add_assign(1);
        } else {
            self.opcode_count.insert(opcode, 1);
        }
        self.total_opcode_count += 1;
        // remember opcode cost
        let gas_cost = self.gas_inspector.last_gas_cost();
        self.opcode_cost.insert(opcode, gas_cost);
    }
}
