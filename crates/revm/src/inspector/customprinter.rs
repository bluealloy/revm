//! Custom print inspector, it has step level information of execution.
//! It is a great tool if some debugging is needed.
//!
use crate::interpreter::{opcode, CallInputs, CreateInputs, Gas, InstructionResult, Interpreter};
use crate::primitives::{Address, Bytes};
use crate::{inspectors::GasInspector, Database, EVMData, Inspector};
#[derive(Clone, Default)]
pub struct CustomPrintTracer {
    gas_inspector: GasInspector,
}

impl<DB: Database> Inspector<DB> for CustomPrintTracer {
    fn initialize_interp(
        &mut self,
        interp: &mut Interpreter,
        data: &mut EVMData<'_, DB>,
    ) -> InstructionResult {
        self.gas_inspector.initialize_interp(interp, data);
        InstructionResult::Continue
    }

    // get opcode by calling `interp.contract.opcode(interp.program_counter())`.
    // all other information can be obtained from interp.
    fn step(&mut self, interp: &mut Interpreter, data: &mut EVMData<'_, DB>) -> InstructionResult {
        let opcode = interp.current_opcode();
        let opcode_str = opcode::OPCODE_JUMPMAP[opcode as usize];

        let gas_remaining = self.gas_inspector.gas_remaining();

        println!(
            "depth:{}, PC:{}, gas:{:#x}({}), OPCODE: {:?}({:?})  refund:{:#x}({}) Stack:{:?}, Data size:{}",
            data.journaled_state.depth(),
            interp.program_counter(),
            gas_remaining,
            gas_remaining,
            opcode_str.unwrap_or("UNKNOWN"),
            opcode,
            interp.gas.refunded(),
            interp.gas.refunded(),
            interp.stack.data(),
            interp.memory.data().len(),
        );

        self.gas_inspector.step(interp, data);

        InstructionResult::Continue
    }

    fn step_end(
        &mut self,
        interp: &mut Interpreter,
        data: &mut EVMData<'_, DB>,
        eval: InstructionResult,
    ) -> InstructionResult {
        self.gas_inspector.step_end(interp, data, eval);
        InstructionResult::Continue
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

    fn create_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &CreateInputs,
        ret: InstructionResult,
        address: Option<Address>,
        remaining_gas: Gas,
        out: Bytes,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        self.gas_inspector
            .create_end(data, inputs, ret, address, remaining_gas, out.clone());
        (ret, address, remaining_gas, out)
    }

    fn call(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        inputs: &mut CallInputs,
    ) -> (InstructionResult, Gas, Bytes) {
        println!(
            "SM CALL:   {:?},context:{:?}, is_static:{:?}, transfer:{:?}, input_size:{:?}",
            inputs.contract,
            inputs.context,
            inputs.is_static,
            inputs.transfer,
            inputs.input.len(),
        );
        (InstructionResult::Continue, Gas::new(0), Bytes::new())
    }

    fn create(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        inputs: &mut CreateInputs,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        println!(
            "CREATE CALL: caller:{:?}, scheme:{:?}, value:{:?}, init_code:{:?}, gas:{:?}",
            inputs.caller, inputs.scheme, inputs.value, inputs.init_code, inputs.gas_limit
        );
        (InstructionResult::Continue, None, Gas::new(0), Bytes::new())
    }

    fn selfdestruct(&mut self, contract: Address, target: Address) {
        println!("SELFDESTRUCT on {contract:?} refund target: {target:?}");
    }
}

#[cfg(test)]
mod test {

    #[cfg(not(feature = "no_gas_measuring"))]
    #[test]
    fn gas_calculation_underflow() {
        use crate::primitives::{address, bytes};

        // https://github.com/bluealloy/revm/issues/277
        // checks this use case
        let mut evm = crate::new();
        let mut database = crate::InMemoryDB::default();
        let code = bytes!("5b597fb075978b6c412c64d169d56d839a8fe01b3f4607ed603b2c78917ce8be1430fe6101e8527ffe64706ecad72a2f5c97a95e006e279dc57081902029ce96af7edae5de116fec610208527f9fc1ef09d4dd80683858ae3ea18869fe789ddc365d8d9d800e26c9872bac5e5b6102285260276102485360d461024953601661024a53600e61024b53607d61024c53600961024d53600b61024e5360b761024f5360596102505360796102515360a061025253607261025353603a6102545360fb61025553601261025653602861025753600761025853606f61025953601761025a53606161025b53606061025c5360a661025d53602b61025e53608961025f53607a61026053606461026153608c6102625360806102635360d56102645360826102655360ae61026653607f6101e8610146610220677a814b184591c555735fdcca53617f4d2b9134b29090c87d01058e27e962047654f259595947443b1b816b65cdb6277f4b59c10a36f4e7b8658f5a5e6f5561");

        let acc_info = crate::primitives::AccountInfo {
            balance: "0x100c5d668240db8e00".parse().unwrap(),
            code_hash: crate::primitives::keccak256(&code),
            code: Some(crate::primitives::Bytecode::new_raw(code.clone())),
            nonce: 1,
        };
        let callee = address!("5fdcca53617f4d2b9134b29090c87d01058e27e9");
        database.insert_account_info(callee, acc_info);
        evm.database(database);
        evm.env.tx.caller = address!("5fdcca53617f4d2b9134b29090c87d01058e27e0");
        evm.env.tx.transact_to = crate::primitives::TransactTo::Call(callee);
        evm.env.tx.data = crate::primitives::Bytes::new();
        evm.env.tx.value = crate::primitives::U256::ZERO;
        let _ = evm.inspect_commit(super::CustomPrintTracer::default());
    }
}
