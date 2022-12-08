use crate::{
    bits::{B160, B256},
    evm_impl::EVMData,
    opcode, spec_opcode_gas, CallInputs, CreateInputs, Database, Gas, Interpreter, Return,
};
use auto_impl::auto_impl;
use bytes::Bytes;

#[auto_impl(&mut, Box)]
pub trait Inspector<DB: Database> {
    /// Called Before the interpreter is initialized.
    ///
    /// If anything other than [Return::Continue] is returned then execution of the interpreter is
    /// skipped.
    fn initialize_interp(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        Return::Continue
    }

    /// Called on each step of the interpreter.
    ///
    /// Information about the current execution, including the memory, stack and more is available
    /// on `interp` (see [Interpreter]).
    ///
    /// # Example
    ///
    /// To get the current opcode, use `interp.current_opcode()`.
    fn step(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        Return::Continue
    }

    /// Called when a log is emitted.
    fn log(
        &mut self,
        _evm_data: &mut EVMData<'_, DB>,
        _address: &B160,
        _topics: &[B256],
        _data: &Bytes,
    ) {
    }

    /// Called after `step` when the instruction has been executed.
    ///
    /// Returning anything other than [Return::Continue] alters the execution of the interpreter.
    fn step_end(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
        _eval: Return,
    ) -> Return {
        Return::Continue
    }

    /// Called whenever a call to a contract is about to start.
    ///
    /// Returning anything other than [Return::Continue] overrides the result of the call.
    fn call(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &mut CallInputs,
        _is_static: bool,
    ) -> (Return, Gas, Bytes) {
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    /// Called when a call to a contract has concluded.
    ///
    /// Returning anything other than the values passed to this function (`(ret, remaining_gas,
    /// out)`) will alter the result of the call.
    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CallInputs,
        remaining_gas: Gas,
        ret: Return,
        out: Bytes,
        _is_static: bool,
    ) -> (Return, Gas, Bytes) {
        (ret, remaining_gas, out)
    }

    /// Called when a contract is about to be created.
    ///
    /// Returning anything other than [Return::Continue] overrides the result of the creation.
    fn create(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &mut CreateInputs,
    ) -> (Return, Option<B160>, Gas, Bytes) {
        (Return::Continue, None, Gas::new(0), Bytes::default())
    }

    /// Called when a contract has been created.
    ///
    /// Returning anything other than the values passed to this function (`(ret, remaining_gas,
    /// address, out)`) will alter the result of the create.
    fn create_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CreateInputs,
        ret: Return,
        address: Option<B160>,
        remaining_gas: Gas,
        out: Bytes,
    ) -> (Return, Option<B160>, Gas, Bytes) {
        (ret, address, remaining_gas, out)
    }

    /// Called when a contract has been self-destructed.
    fn selfdestruct(&mut self) {}
}

#[derive(Clone, Copy)]
pub struct NoOpInspector();

impl<DB: Database> Inspector<DB> for NoOpInspector {}

#[derive(Clone, Copy, Debug, Default)]
pub struct GasInspector {
    /// We now batch continual gas_block in one go, that means we need to reduce it if we want
    /// to get correct gas remaining. Check revm/interp/contract/analyze for more information
    reduced_gas_block: u64,
    full_gas_block: u64,
    was_return: bool,
    was_jumpi: Option<usize>,

    gas_remaining: u64,
}

impl GasInspector {
    pub fn gas_remaining(&self) -> u64 {
        self.gas_remaining
    }
}

impl<DB: Database> Inspector<DB> for GasInspector {
    fn initialize_interp(
        &mut self,
        interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        self.full_gas_block = interp.contract.first_gas_block();
        self.gas_remaining = interp.gas.limit();
        Return::Continue
    }

    // get opcode by calling `interp.contract.opcode(interp.program_counter())`.
    // all other information can be obtained from interp.
    fn step(
        &mut self,
        interp: &mut Interpreter,
        data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        let op = interp.current_opcode();

        // calculate gas_block
        let infos = spec_opcode_gas(data.env.cfg.spec_id);
        let info = &infos[op as usize];

        let pc = interp.program_counter();
        if op == opcode::JUMPI {
            self.reduced_gas_block += info.get_gas() as u64;
            self.was_jumpi = Some(pc);
        } else if info.is_gas_block_end() {
            self.reduced_gas_block = 0;
            self.full_gas_block = interp.contract.gas_block(pc);
        } else {
            self.reduced_gas_block += info.get_gas() as u64;
        }

        Return::Continue
    }

    fn step_end(
        &mut self,
        interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
        _eval: Return,
    ) -> Return {
        let pc = interp.program_counter();
        if let Some(was_pc) = self.was_jumpi {
            if let Some(new_pc) = pc.checked_sub(1) {
                if was_pc == new_pc {
                    self.reduced_gas_block = 0;
                    self.full_gas_block = interp.contract.gas_block(was_pc);
                }
            }
            self.was_jumpi = None;
        } else if self.was_return {
            // we are ok to decrement PC by one as it is return of call
            let previous_pc = pc - 1;
            self.reduced_gas_block = 0;
            self.full_gas_block = interp.contract.gas_block(previous_pc);
            self.was_return = false;
        }
        self.gas_remaining =
            interp.gas.remaining() + (self.full_gas_block - self.reduced_gas_block);
        Return::Continue
    }

    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CallInputs,
        remaining_gas: Gas,
        ret: Return,
        out: Bytes,
        _is_static: bool,
    ) -> (Return, Gas, Bytes) {
        self.was_return = true;
        (ret, remaining_gas, out)
    }

    fn create_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CreateInputs,
        ret: Return,
        address: Option<B160>,
        remaining_gas: Gas,
        out: Bytes,
    ) -> (Return, Option<B160>, Gas, Bytes) {
        self.was_return = true;
        (ret, address, remaining_gas, out)
    }
}

#[cfg(test)]
mod tests {
    use crate::db::BenchmarkDB;
    use crate::{
        opcode, Bytecode, CallInputs, CreateInputs, Database, EVMData, Gas, GasInspector,
        Inspector, Interpreter, OpCode, Return, TransactTo, B160, B256,
    };
    use bytes::Bytes;
    use hex_literal::hex;

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
        ) -> Return {
            self.gas_inspector
                .initialize_interp(interp, data, is_static);
            Return::Continue
        }

        fn step(
            &mut self,
            interp: &mut Interpreter,
            data: &mut EVMData<'_, DB>,
            is_static: bool,
        ) -> Return {
            self.pc = interp.program_counter();
            self.gas_inspector.step(interp, data, is_static);
            Return::Continue
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
            eval: Return,
        ) -> Return {
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
        ) -> (Return, Gas, Bytes) {
            self.gas_inspector.call(data, call, is_static);

            (Return::Continue, Gas::new(call.gas_limit), Bytes::new())
        }

        fn call_end(
            &mut self,
            data: &mut EVMData<'_, DB>,
            inputs: &CallInputs,
            remaining_gas: Gas,
            ret: Return,
            out: Bytes,
            is_static: bool,
        ) -> (Return, Gas, Bytes) {
            self.gas_inspector
                .call_end(data, inputs, remaining_gas, ret, out.clone(), is_static);
            (ret, remaining_gas, out)
        }

        fn create(
            &mut self,
            data: &mut EVMData<'_, DB>,
            call: &mut CreateInputs,
        ) -> (Return, Option<B160>, Gas, Bytes) {
            self.gas_inspector.create(data, call);

            (
                Return::Continue,
                None,
                Gas::new(call.gas_limit),
                Bytes::new(),
            )
        }

        fn create_end(
            &mut self,
            data: &mut EVMData<'_, DB>,
            inputs: &CreateInputs,
            status: Return,
            address: Option<B160>,
            gas: Gas,
            retdata: Bytes,
        ) -> (Return, Option<B160>, Gas, Bytes) {
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
        let (result, state) = evm.inspect(&mut inspector);
        println!("{result:?} {state:?} {inspector:?}");

        for (pc, gas) in inspector.gas_remaining_steps {
            println!(
                "{pc} {} {gas:?}",
                OpCode::try_from_u8(bytecode.bytes()[pc]).unwrap().as_str(),
            );
        }
    }
}
