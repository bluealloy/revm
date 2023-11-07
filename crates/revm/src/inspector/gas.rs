//! GasIspector. Helper Inspector to calculate gas for others.

use crate::interpreter::{CallInputs, CreateInputs, Gas, InstructionResult};
use crate::primitives::{db::Database, Address, Bytes};
use crate::{EVMData, Inspector};

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
    #[cfg(not(feature = "no_gas_measuring"))]
    fn initialize_interp(
        &mut self,
        interp: &mut crate::interpreter::Interpreter<'_>,
        _data: &mut EVMData<'_, DB>,
    ) {
        self.gas_remaining = interp.gas.limit();
    }

    #[cfg(not(feature = "no_gas_measuring"))]
    fn step_end(
        &mut self,
        interp: &mut crate::interpreter::Interpreter<'_>,
        _data: &mut EVMData<'_, DB>,
    ) {
        let last_gas = core::mem::replace(&mut self.gas_remaining, interp.gas.remaining());
        self.last_gas_cost = last_gas.saturating_sub(self.last_gas_cost);
    }

    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CallInputs,
        mut remaining_gas: Gas,
        ret: InstructionResult,
        out: Bytes,
    ) -> (InstructionResult, Gas, Bytes) {
        if ret.is_error() {
            remaining_gas.record_cost(remaining_gas.remaining());
            self.gas_remaining = 0;
            (ret, remaining_gas, out)
        } else {
            (ret, remaining_gas, out)
        }
    }

    fn create_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CreateInputs,
        ret: InstructionResult,
        address: Option<Address>,
        remaining_gas: Gas,
        out: Bytes,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        (ret, address, remaining_gas, out)
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::{CallInputs, CreateInputs, Gas, InstructionResult, Interpreter};
    use crate::primitives::{Address, Bytes, B256};
    use crate::{inspectors::GasInspector, Database, EVMData, Inspector};

    #[derive(Default, Debug)]
    struct StackInspector {
        pc: usize,
        gas_inspector: GasInspector,
        gas_remaining_steps: Vec<(usize, u64)>,
    }

    impl<DB: Database> Inspector<DB> for StackInspector {
        fn initialize_interp(&mut self, interp: &mut Interpreter<'_>, data: &mut EVMData<'_, DB>) {
            self.gas_inspector.initialize_interp(interp, data);
        }

        fn step(&mut self, interp: &mut Interpreter<'_>, data: &mut EVMData<'_, DB>) {
            self.pc = interp.program_counter();
            self.gas_inspector.step(interp, data);
        }

        fn log(
            &mut self,
            evm_data: &mut EVMData<'_, DB>,
            address: &Address,
            topics: &[B256],
            data: &Bytes,
        ) {
            self.gas_inspector.log(evm_data, address, topics, data);
        }

        fn step_end(&mut self, interp: &mut Interpreter<'_>, data: &mut EVMData<'_, DB>) {
            self.gas_inspector.step_end(interp, data);
            self.gas_remaining_steps
                .push((self.pc, self.gas_inspector.gas_remaining()));
        }

        fn call(
            &mut self,
            data: &mut EVMData<'_, DB>,
            call: &mut CallInputs,
        ) -> (InstructionResult, Gas, Bytes) {
            self.gas_inspector.call(data, call);

            (
                InstructionResult::Continue,
                Gas::new(call.gas_limit),
                Bytes::new(),
            )
        }

        fn call_end(
            &mut self,
            data: &mut EVMData<'_, DB>,
            inputs: &CallInputs,
            remaining_gas: Gas,
            ret: InstructionResult,
            out: Bytes,
        ) -> (InstructionResult, Gas, Bytes) {
            self.gas_inspector
                .call_end(data, inputs, remaining_gas, ret, out.clone());
            (ret, remaining_gas, out)
        }

        fn create(
            &mut self,
            data: &mut EVMData<'_, DB>,
            call: &mut CreateInputs,
        ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
            self.gas_inspector.create(data, call);

            (
                InstructionResult::Continue,
                None,
                Gas::new(call.gas_limit),
                Bytes::new(),
            )
        }

        fn create_end(
            &mut self,
            data: &mut EVMData<'_, DB>,
            inputs: &CreateInputs,
            status: InstructionResult,
            address: Option<Address>,
            gas: Gas,
            retdata: Bytes,
        ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
            self.gas_inspector
                .create_end(data, inputs, status, address, gas, retdata.clone());
            (status, address, gas, retdata)
        }
    }

    #[test]
    #[cfg(not(feature = "optimism"))]
    fn test_gas_inspector() {
        use crate::db::BenchmarkDB;
        use crate::interpreter::opcode;
        use crate::primitives::{address, Bytecode, Bytes, TransactTo};

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

        let mut evm = crate::new();
        evm.database(BenchmarkDB::new_bytecode(bytecode.clone()));
        evm.env.tx.caller = address!("1000000000000000000000000000000000000000");
        evm.env.tx.transact_to =
            TransactTo::Call(address!("0000000000000000000000000000000000000000"));
        evm.env.tx.gas_limit = 21100;

        let mut inspector = StackInspector::default();
        evm.inspect(&mut inspector).unwrap();

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
