//! GasIspector. Helper Inspector to calculate gas for others.
use interpreter::{CallOutcome, CreateOutcome, Gas};

/// Helper that keeps track of gas.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct GasInspector {
    gas_remaining: u64,
    last_gas_cost: u64,
}

impl Default for GasInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl GasInspector {
    /// Returns the remaining gas.
    #[inline]
    pub fn gas_remaining(&self) -> u64 {
        self.gas_remaining
    }

    /// Returns the last gas cost.
    #[inline]
    pub fn last_gas_cost(&self) -> u64 {
        self.last_gas_cost
    }

    /// Create a new gas inspector.
    pub fn new() -> Self {
        Self {
            gas_remaining: 0,
            last_gas_cost: 0,
        }
    }

    /// Sets remaining gas to gas limit.
    #[inline]
    pub fn initialize_interp(&mut self, gas: &Gas) {
        self.gas_remaining = gas.limit();
    }

    /// Sets the remaining gas.
    #[inline]
    pub fn step(&mut self, gas: &Gas) {
        self.gas_remaining = gas.remaining();
    }

    /// calculate last gas cost and remaining gas.
    #[inline]
    pub fn step_end(&mut self, gas: &mut Gas) {
        let remaining = gas.remaining();
        self.last_gas_cost = self.gas_remaining.saturating_sub(remaining);
        self.gas_remaining = remaining;
    }

    /// Spend all gas if call failed.
    #[inline]
    pub fn call_end(&mut self, outcome: &mut CallOutcome) {
        if outcome.result.result.is_error() {
            outcome.result.gas.spend_all();
            self.gas_remaining = 0;
        }
    }

    /// Spend all gas if create failed.
    #[inline]
    pub fn create_end(&mut self, outcome: &mut CreateOutcome) {
        if outcome.result.result.is_error() {
            outcome.result.gas.spend_all();
            self.gas_remaining = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InspectEvm, Inspector};
    use context::Context;
    use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
    use handler::{MainBuilder, MainContext};
    use interpreter::{
        interpreter_types::{Jumps, LoopControl},
        CallInputs, CreateInputs, Interpreter, InterpreterTypes,
    };
    use primitives::{Bytes, TxKind};
    use state::bytecode::{opcode, Bytecode};

    #[derive(Default, Debug)]
    struct StackInspector {
        pc: usize,
        gas_inspector: GasInspector,
        gas_remaining_steps: Vec<(usize, u64)>,
    }

    impl<CTX, INTR: InterpreterTypes> Inspector<CTX, INTR> for StackInspector {
        fn initialize_interp(&mut self, interp: &mut Interpreter<INTR>, _context: &mut CTX) {
            self.gas_inspector.initialize_interp(interp.control.gas());
        }

        fn step(&mut self, interp: &mut Interpreter<INTR>, _context: &mut CTX) {
            self.pc = interp.bytecode.pc();
            self.gas_inspector.step(interp.control.gas());
        }

        fn step_end(&mut self, interp: &mut Interpreter<INTR>, _context: &mut CTX) {
            self.gas_inspector.step_end(interp.control.gas_mut());
            self.gas_remaining_steps
                .push((self.pc, self.gas_inspector.gas_remaining()));
        }

        fn call_end(&mut self, _c: &mut CTX, _i: &CallInputs, outcome: &mut CallOutcome) {
            self.gas_inspector.call_end(outcome)
        }

        fn create_end(&mut self, _c: &mut CTX, _i: &CreateInputs, outcome: &mut CreateOutcome) {
            self.gas_inspector.create_end(outcome)
        }
    }

    #[test]
    fn test_gas_inspector() {
        let contract_data: Bytes = Bytes::from(vec![
            opcode::PUSH1,
            0x1,
            opcode::PUSH1,
            0xb,
            opcode::JUMPI,
            opcode::PUSH1,
            0x1,
            opcode::PUSH1,
            0x1,
            opcode::PUSH1,
            0x1,
            opcode::JUMPDEST,
            opcode::STOP,
        ]);
        let bytecode = Bytecode::new_raw(contract_data);

        let ctx = Context::mainnet()
            .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
            .modify_tx_chained(|tx| {
                tx.caller = BENCH_CALLER;
                tx.kind = TxKind::Call(BENCH_TARGET);
                tx.gas_limit = 21100;
            });

        let mut evm = ctx.build_mainnet_with_inspector(StackInspector::default());

        // Run evm.
        evm.inspect_replay().unwrap();

        let inspector = &evm.data.inspector;

        // Starting from 100gas
        let steps = vec![
            // push1 -3
            (0, 97),
            // push1 -3
            (2, 94),
            // jumpi -10
            (4, 84),
            // jumpdest 1
            (11, 83),
            // stop 0
            (12, 83),
        ];

        assert_eq!(inspector.gas_remaining_steps, steps);
    }
}
