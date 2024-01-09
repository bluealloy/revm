//! GasIspector. Helper Inspector to calculate gas for others.

use crate::{
    interpreter::InterpreterResult,
    primitives::{db::Database, Address},
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
        let last_gas = core::mem::replace(&mut self.gas_remaining, interp.gas.remaining());
        self.last_gas_cost = last_gas.saturating_sub(self.last_gas_cost);
    }

    fn call_end(
        &mut self,
        _context: &mut EvmContext<DB>,
        mut result: InterpreterResult,
    ) -> InterpreterResult {
        if result.result.is_error() {
            result.gas.record_cost(result.gas.remaining());
            self.gas_remaining = 0;
        }
        result
    }

    fn create_end(
        &mut self,
        _context: &mut EvmContext<DB>,
        result: InterpreterResult,
        address: Option<Address>,
    ) -> (InterpreterResult, Option<Address>) {
        (result, address)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        inspector::GetInspector,
        inspectors::GasInspector,
        interpreter::{CallInputs, CreateInputs, Interpreter, InterpreterResult},
        primitives::{Address, Log},
        Database, EvmContext, Inspector,
    };
    use core::ops::Range;

    #[derive(Default, Debug)]
    struct StackInspector {
        pc: usize,
        gas_inspector: GasInspector,
        gas_remaining_steps: Vec<(usize, u64)>,
    }

    impl<DB: Database> GetInspector<'_, DB> for StackInspector {
        fn get_inspector(&mut self) -> &mut dyn Inspector<DB> {
            self
        }
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
        ) -> Option<(InterpreterResult, Range<usize>)> {
            self.gas_inspector.call(context, call)
        }

        fn call_end(
            &mut self,
            context: &mut EvmContext<DB>,
            result: InterpreterResult,
        ) -> InterpreterResult {
            self.gas_inspector.call_end(context, result)
        }

        fn create(
            &mut self,
            context: &mut EvmContext<DB>,
            call: &mut CreateInputs,
        ) -> Option<(InterpreterResult, Option<Address>)> {
            self.gas_inspector.create(context, call);
            None
        }

        fn create_end(
            &mut self,
            context: &mut EvmContext<DB>,
            result: InterpreterResult,
            address: Option<Address>,
        ) -> (InterpreterResult, Option<Address>) {
            self.gas_inspector.create_end(context, result, address)
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
