//! GasIspector. Helper Inspector to calculate gas for others.

use crate::{EvmContext, EvmWiring, Inspector};
use interpreter::{CallInputs, CallOutcome, CreateInputs, CreateOutcome, Interpreter};

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

impl<EvmWiringT: EvmWiring> Inspector<EvmWiringT> for GasInspector {
    fn initialize_interp(
        &mut self,
        interp: &mut Interpreter,
        _context: &mut EvmContext<EvmWiringT>,
    ) {
        self.gas_remaining = interp.gas.limit();
    }

    fn step(&mut self, interp: &mut Interpreter, _context: &mut EvmContext<EvmWiringT>) {
        self.gas_remaining = interp.gas.remaining();
    }

    fn step_end(&mut self, interp: &mut Interpreter, _context: &mut EvmContext<EvmWiringT>) {
        let remaining = interp.gas.remaining();
        self.last_gas_cost = self.gas_remaining.saturating_sub(remaining);
        self.gas_remaining = remaining;
    }

    fn call_end(
        &mut self,
        _context: &mut EvmContext<EvmWiringT>,
        _inputs: &CallInputs,
        mut outcome: CallOutcome,
    ) -> CallOutcome {
        if outcome.result.result.is_error() {
            outcome.result.gas.spend_all();
            self.gas_remaining = 0;
        }
        outcome
    }

    fn create_end(
        &mut self,
        _context: &mut EvmContext<EvmWiringT>,
        _inputs: &CreateInputs,
        mut outcome: CreateOutcome,
    ) -> CreateOutcome {
        if outcome.result.result.is_error() {
            outcome.result.gas.spend_all();
            self.gas_remaining = 0;
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inspector::inspector_handle_register, Evm, EvmWiring};
    use bytecode::Bytecode;
    use database::BenchmarkDB;
    use interpreter::{opcode, Interpreter};
    use primitives::{address, Bytes, Log, TxKind};
    use wiring::{DefaultEthereumWiring, EthereumWiring, EvmWiring as PrimitiveEvmWiring};

    type TestEvmWiring = DefaultEthereumWiring;

    #[derive(Default, Debug)]
    struct StackInspector {
        pc: usize,
        gas_inspector: GasInspector,
        gas_remaining_steps: Vec<(usize, u64)>,
    }

    impl<EvmWiringT: EvmWiring> Inspector<EvmWiringT> for StackInspector {
        fn initialize_interp(
            &mut self,
            interp: &mut Interpreter,
            context: &mut EvmContext<EvmWiringT>,
        ) {
            self.gas_inspector.initialize_interp(interp, context);
        }

        fn step(&mut self, interp: &mut Interpreter, context: &mut EvmContext<EvmWiringT>) {
            self.pc = interp.program_counter();
            self.gas_inspector.step(interp, context);
        }

        fn log(
            &mut self,
            interp: &mut Interpreter,
            context: &mut EvmContext<EvmWiringT>,
            log: &Log,
        ) {
            self.gas_inspector.log(interp, context, log);
        }

        fn step_end(&mut self, interp: &mut Interpreter, context: &mut EvmContext<EvmWiringT>) {
            self.gas_inspector.step_end(interp, context);
            self.gas_remaining_steps
                .push((self.pc, self.gas_inspector.gas_remaining()));
        }

        fn call(
            &mut self,
            context: &mut EvmContext<EvmWiringT>,
            call: &mut CallInputs,
        ) -> Option<CallOutcome> {
            self.gas_inspector.call(context, call)
        }

        fn call_end(
            &mut self,
            context: &mut EvmContext<EvmWiringT>,
            inputs: &CallInputs,
            outcome: CallOutcome,
        ) -> CallOutcome {
            self.gas_inspector.call_end(context, inputs, outcome)
        }

        fn create(
            &mut self,
            context: &mut EvmContext<EvmWiringT>,
            call: &mut CreateInputs,
        ) -> Option<CreateOutcome> {
            self.gas_inspector.create(context, call);
            None
        }

        fn create_end(
            &mut self,
            context: &mut EvmContext<EvmWiringT>,
            inputs: &CreateInputs,
            outcome: CreateOutcome,
        ) -> CreateOutcome {
            self.gas_inspector.create_end(context, inputs, outcome)
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

        let mut evm = Evm::<EthereumWiring<BenchmarkDB, StackInspector>>::builder()
            .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
            .with_default_ext_ctx()
            .modify_tx_env(|tx| {
                *tx = <TestEvmWiring as PrimitiveEvmWiring>::Transaction::default();

                tx.caller = address!("1000000000000000000000000000000000000000");
                tx.transact_to = TxKind::Call(address!("0000000000000000000000000000000000000000"));
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
