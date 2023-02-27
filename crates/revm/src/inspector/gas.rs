//! GasIspector. Helper Inspector to calculte gas for others.
//!
use crate::interpreter::{CallInputs, CreateInputs, Gas, InstructionResult};
use crate::primitives::{db::Database, Bytes, B160};
use crate::{evm_impl::EVMData, Inspector};

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
        interp: &mut crate::interpreter::Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> InstructionResult {
        self.gas_remaining = interp.gas.limit();
        InstructionResult::Continue
    }

    // get opcode by calling `interp.contract.opcode(interp.program_counter())`.
    // all other information can be obtained from interp.

    #[cfg(not(feature = "no_gas_measuring"))]
    fn step(
        &mut self,
        _interp: &mut crate::interpreter::Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> InstructionResult {
        InstructionResult::Continue
    }

    #[cfg(not(feature = "no_gas_measuring"))]
    fn step_end(
        &mut self,
        interp: &mut crate::interpreter::Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
        _eval: InstructionResult,
    ) -> InstructionResult {
        let last_gas = self.gas_remaining;
        self.gas_remaining = interp.gas.remaining();
        if last_gas > self.gas_remaining {
            self.last_gas_cost = last_gas - self.gas_remaining;
        } else {
            self.last_gas_cost = 0;
        }
        InstructionResult::Continue
    }

    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CallInputs,
        remaining_gas: Gas,
        ret: InstructionResult,
        out: Bytes,
        _is_static: bool,
    ) -> (InstructionResult, Gas, Bytes) {
        (ret, remaining_gas, out)
    }

    fn create_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CreateInputs,
        ret: InstructionResult,
        address: Option<B160>,
        remaining_gas: Gas,
        out: Bytes,
    ) -> (InstructionResult, Option<B160>, Gas, Bytes) {
        (ret, address, remaining_gas, out)
    }
}

#[cfg(test)]
mod tests {
    use crate::db::BenchmarkDB;
    use crate::interpreter::{
        opcode, CallInputs, CreateInputs, Gas, InstructionResult, Interpreter, OpCode,
    };
    use crate::primitives::{
        hex_literal::hex, Bytecode, Bytes, ResultAndState, TransactTo, B160, B256,
    };
    use crate::{inspectors::GasInspector, Database, EVMData, Inspector};

    #[derive(Default, Debug)]
    struct StackInspector {
        pc: usize,
        gas_inspector: GasInspector,
        gas_remaining_steps: Vec<(usize, u64)>,
    }

    impl<DB: Database> Inspector<DB> for StackInspector {
        fn initialize_interp(
            &mut self,
            interp: &mut Interpreter,
            data: &mut EVMData<'_, DB>,
            is_static: bool,
        ) -> InstructionResult {
            self.gas_inspector
                .initialize_interp(interp, data, is_static);
            InstructionResult::Continue
        }

        fn step(
            &mut self,
            interp: &mut Interpreter,
            data: &mut EVMData<'_, DB>,
            is_static: bool,
        ) -> InstructionResult {
            self.pc = interp.program_counter();
            self.gas_inspector.step(interp, data, is_static);
            InstructionResult::Continue
        }

        fn log(
            &mut self,
            evm_data: &mut EVMData<'_, DB>,
            address: &B160,
            topics: &[B256],
            data: &Bytes,
        ) {
            self.gas_inspector.log(evm_data, address, topics, data);
        }

        fn step_end(
            &mut self,
            interp: &mut Interpreter,
            data: &mut EVMData<'_, DB>,
            is_static: bool,
            eval: InstructionResult,
        ) -> InstructionResult {
            self.gas_inspector.step_end(interp, data, is_static, eval);
            self.gas_remaining_steps
                .push((self.pc, self.gas_inspector.gas_remaining()));
            eval
        }

        fn call(
            &mut self,
            data: &mut EVMData<'_, DB>,
            call: &mut CallInputs,
            is_static: bool,
        ) -> (InstructionResult, Gas, Bytes) {
            self.gas_inspector.call(data, call, is_static);

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
            is_static: bool,
        ) -> (InstructionResult, Gas, Bytes) {
            self.gas_inspector
                .call_end(data, inputs, remaining_gas, ret, out.clone(), is_static);
            (ret, remaining_gas, out)
        }

        fn create(
            &mut self,
            data: &mut EVMData<'_, DB>,
            call: &mut CreateInputs,
        ) -> (InstructionResult, Option<B160>, Gas, Bytes) {
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
            address: Option<B160>,
            gas: Gas,
            retdata: Bytes,
        ) -> (InstructionResult, Option<B160>, Gas, Bytes) {
            self.gas_inspector
                .create_end(data, inputs, status, address, gas, retdata.clone());
            (status, address, gas, retdata)
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

        let mut evm = crate::new();
        evm.database(BenchmarkDB::new_bytecode(bytecode.clone()));
        evm.env.tx.caller = B160(hex!("1000000000000000000000000000000000000000"));
        evm.env.tx.transact_to =
            TransactTo::Call(B160(hex!("0000000000000000000000000000000000000000")));
        evm.env.tx.gas_limit = 21100;

        let mut inspector = StackInspector::default();
        let ResultAndState { result, state } = evm.inspect(&mut inspector).unwrap();
        println!("{result:?} {state:?} {inspector:?}");

        for (pc, gas) in inspector.gas_remaining_steps {
            println!(
                "{pc} {} {gas:?}",
                OpCode::try_from_u8(bytecode.bytes()[pc]).unwrap().as_str(),
            );
        }
    }
}
