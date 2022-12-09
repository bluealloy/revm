use bytes::Bytes;
pub use revm::Inspector;
use revm::{
    bits::B160,
    opcode::{self},
    CallInputs, CreateInputs, Database, EVMData, Gas, GasInspector, Return,
};
#[derive(Clone)]
pub struct CustomPrintTracer {
    gas_inspector: GasInspector,
}

impl CustomPrintTracer {
    pub fn new() -> Self {
        Self {
            gas_inspector: GasInspector::default(),
        }
    }
}

impl<DB: Database> Inspector<DB> for CustomPrintTracer {
    fn initialize_interp(
        &mut self,
        interp: &mut revm::Interpreter,
        data: &mut EVMData<'_, DB>,
        is_static: bool,
    ) -> Return {
        self.gas_inspector
            .initialize_interp(interp, data, is_static);
        Return::Continue
    }

    // get opcode by calling `interp.contract.opcode(interp.program_counter())`.
    // all other information can be obtained from interp.
    fn step(
        &mut self,
        interp: &mut revm::Interpreter,
        data: &mut EVMData<'_, DB>,
        is_static: bool,
    ) -> Return {
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

        self.gas_inspector.step(interp, data, is_static);

        Return::Continue
    }

    fn step_end(
        &mut self,
        interp: &mut revm::Interpreter,
        data: &mut EVMData<'_, DB>,
        is_static: bool,
        eval: revm::Return,
    ) -> Return {
        self.gas_inspector.step_end(interp, data, is_static, eval);
        Return::Continue
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

    fn create_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &CreateInputs,
        ret: Return,
        address: Option<B160>,
        remaining_gas: Gas,
        out: Bytes,
    ) -> (Return, Option<B160>, Gas, Bytes) {
        self.gas_inspector
            .create_end(data, inputs, ret, address, remaining_gas, out.clone());
        (ret, address, remaining_gas, out)
    }

    fn call(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        inputs: &mut CallInputs,
        is_static: bool,
    ) -> (Return, Gas, Bytes) {
        println!(
            "SM CALL:   {:?},context:{:?}, is_static:{:?}, transfer:{:?}, input_size:{:?}",
            inputs.contract,
            inputs.context,
            is_static,
            inputs.transfer,
            inputs.input.len(),
        );
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn create(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        inputs: &mut CreateInputs,
    ) -> (Return, Option<B160>, Gas, Bytes) {
        println!(
            "CREATE CALL: caller:{:?}, scheme:{:?}, value:{:?}, init_code:{:?}, gas:{:?}",
            inputs.caller,
            inputs.scheme,
            inputs.value,
            hex::encode(&inputs.init_code),
            inputs.gas_limit
        );
        (Return::Continue, None, Gas::new(0), Bytes::new())
    }

    fn selfdestruct(&mut self) {
        //, address: B160, target: B160) {
        println!("SELFDESTRUCT on "); //{:?} target: {:?}", address, target);
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use hex_literal::hex;
    use revm::{common::keccak256, Bytecode, TransactTo, U256};

    #[test]
    fn gas_calculation_underflow() {
        // https://github.com/bluealloy/revm/issues/277
        // checks this usecase
        let mut evm = revm::new();
        let mut database = revm::InMemoryDB::default();
        let code: Bytes = hex::decode("5b597fb075978b6c412c64d169d56d839a8fe01b3f4607ed603b2c78917ce8be1430fe6101e8527ffe64706ecad72a2f5c97a95e006e279dc57081902029ce96af7edae5de116fec610208527f9fc1ef09d4dd80683858ae3ea18869fe789ddc365d8d9d800e26c9872bac5e5b6102285260276102485360d461024953601661024a53600e61024b53607d61024c53600961024d53600b61024e5360b761024f5360596102505360796102515360a061025253607261025353603a6102545360fb61025553601261025653602861025753600761025853606f61025953601761025a53606161025b53606061025c5360a661025d53602b61025e53608961025f53607a61026053606461026153608c6102625360806102635360d56102645360826102655360ae61026653607f6101e8610146610220677a814b184591c555735fdcca53617f4d2b9134b29090c87d01058e27e962047654f259595947443b1b816b65cdb6277f4b59c10a36f4e7b8658f5a5e6f5561").unwrap().into();

        let acc_info = revm::AccountInfo {
            balance: "0x100c5d668240db8e00".parse().unwrap(),
            code_hash: keccak256(&code),
            code: Some(Bytecode::new_raw(code.clone())),
            nonce: "1".parse().unwrap(),
        };
        let callee = hex!("5fdcca53617f4d2b9134b29090c87d01058e27e9");
        database.insert_account_info(B160(callee), acc_info);
        evm.database(database);
        evm.env.tx.caller = B160(hex!("5fdcca53617f4d2b9134b29090c87d01058e27e0"));
        evm.env.tx.transact_to = TransactTo::Call(B160(callee));
        evm.env.tx.data = Bytes::from(hex::decode("").unwrap());
        evm.env.tx.value = U256::ZERO;
        evm.inspect_commit(CustomPrintTracer::new());
    }
}
