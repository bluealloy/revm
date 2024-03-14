//! GasIspector. Helper Inspector to calculate gas for others.

use revm_interpreter::CallOutcome;

use crate::{
    interpreter::{CallInputs, CreateInputs, CreateOutcome},
    primitives::db::Database,
    EvmContext, Inspector,
};

/// Helper [Inspector] that keeps track of gas.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GasInspector {
    gas_remaining: u64,
    last_gas_cost: u64,
}

impl GasInspector {
    pub fn gas_remaining(&self) -> u64 {
        self.gas_remaining
    }

    pub fn last_gas_cost(&self) -> u64 {
        self.last_gas_cost
    }
}

impl<DB: Database> Inspector<DB> for GasInspector {
    fn initialize_interp(
        &mut self,
        interp: &mut crate::interpreter::Interpreter,
        _context: &mut EvmContext<DB>,
    ) {
        self.gas_remaining = interp.gas.limit();
    }

    fn step_end(
        &mut self,
        interp: &mut crate::interpreter::Interpreter,
        _context: &mut EvmContext<DB>,
    ) {
        let last_gas_remaining =
            core::mem::replace(&mut self.gas_remaining, interp.gas.remaining());
        self.last_gas_cost = last_gas_remaining.saturating_sub(self.gas_remaining);
    }

    fn call_end(
        &mut self,
        _context: &mut EvmContext<DB>,
        _inputs: &CallInputs,
        mut outcome: CallOutcome,
    ) -> CallOutcome {
        if outcome.result.result.is_error() {
            outcome
                .result
                .gas
                .record_cost(outcome.result.gas.remaining());
            self.gas_remaining = 0;
        }
        outcome
    }

    fn create_end(
        &mut self,
        _context: &mut EvmContext<DB>,
        _inputs: &CreateInputs,
        outcome: CreateOutcome,
    ) -> CreateOutcome {
        outcome
    }
}

#[cfg(test)]
mod tests {

    use revm_interpreter::CallOutcome;
    use revm_interpreter::CreateOutcome;

    use crate::{
        inspectors::GasInspector,
        interpreter::{CallInputs, CreateInputs, Interpreter},
        primitives::Log,
        Database, EvmContext, Inspector,
    };

    #[derive(Default, Debug)]
    struct StackInspector {
        pc: usize,
        gas_inspector: GasInspector,
        gas_remaining_steps: Vec<(usize, u64)>,
    }

    impl<DB: Database> Inspector<DB> for StackInspector {
        fn initialize_interp(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
            self.gas_inspector.initialize_interp(interp, context);
        }

        fn step(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
            self.pc = interp.program_counter();
            self.gas_inspector.step(interp, context);
        }

        fn log(&mut self, context: &mut EvmContext<DB>, log: &Log) {
            self.gas_inspector.log(context, log);
        }

        fn step_end(&mut self, interp: &mut Interpreter, context: &mut EvmContext<DB>) {
            self.gas_inspector.step_end(interp, context);
            self.gas_remaining_steps
                .push((self.pc, self.gas_inspector.gas_remaining()));
        }

        fn call(
            &mut self,
            context: &mut EvmContext<DB>,
            call: &mut CallInputs,
        ) -> Option<CallOutcome> {
            self.gas_inspector.call(context, call)
        }

        fn call_end(
            &mut self,
            context: &mut EvmContext<DB>,
            inputs: &CallInputs,
            outcome: CallOutcome,
        ) -> CallOutcome {
            self.gas_inspector.call_end(context, inputs, outcome)
        }

        fn create(
            &mut self,
            context: &mut EvmContext<DB>,
            call: &mut CreateInputs,
        ) -> Option<CreateOutcome> {
            self.gas_inspector.create(context, call);
            None
        }

        fn create_end(
            &mut self,
            context: &mut EvmContext<DB>,
            inputs: &CreateInputs,
            outcome: CreateOutcome,
        ) -> CreateOutcome {
            self.gas_inspector.create_end(context, inputs, outcome)
        }
    }

    #[test]
    fn test_gas_inspector() {
        use crate::{
            db::BenchmarkDB,
            inspector::inspector_handle_register,
            interpreter::opcode,
            primitives::{address, Bytecode, Bytes, TransactTo},
            Evm,
        };

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

        let mut evm: Evm<'_, StackInspector, BenchmarkDB> = Evm::builder()
            .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
            .with_external_context(StackInspector::default())
            .modify_tx_env(|tx| {
                tx.clear();
                tx.caller = address!("1000000000000000000000000000000000000000");
                tx.transact_to =
                    TransactTo::Call(address!("0000000000000000000000000000000000000000"));
                tx.gas_limit = 21100;
            })
            .append_handler_register(inspector_handle_register)
            .build();

        // run evm.
        evm.transact().unwrap();

        let inspector = evm.into_context().external;

        // starting from 100gas
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
