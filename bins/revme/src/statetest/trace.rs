use bytes::Bytes;
use primitive_types::{H160, U256};
pub use revm::Inspector;
use revm::{opcode, Database, EVMData, Gas, Return};

#[derive(Clone)]
pub struct CustomPrintTracer {}

impl<DB: Database> Inspector<DB> for CustomPrintTracer {
    // get opcode by calling `machine.contract.opcode(machine.program_counter())`.
    // all other information can be obtained from machine.
    fn step(
        &mut self,
        machine: &mut revm::Machine,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        let opcode = match machine.contract.code.get(machine.program_counter()) {
            Some(opcode) => opcode,
            None => return Return::Continue,
        };
        let opcode_str = opcode::OPCODE_JUMPMAP[*opcode as usize];
        //if self.
        println!(
            "depth:{}, PC:{}, gas:{:#x}({}), OPCODE: {:?}({:?})  refund:{:#x}({}) Stack:{:?}, Data:",
            machine.call_depth,
            machine.program_counter(),
            machine.gas.remaining(),
            machine.gas.remaining(),
            opcode_str.unwrap(),
            opcode,
            machine.gas.refunded(),
            machine.gas.refunded(),
            machine.stack.data(),
            //hex::encode(machine.memory.data()),
        );
        Return::Continue
    }

    // fn load_account(&mut self, address: &H160) {
    //     println!("ACCOUNT LOADED:{:?}", address);
    // }

    fn step_end(&mut self, _eval: revm::Return, _machine: &mut revm::Machine) -> Return {
        Return::Continue
    }

    // fn sload(&mut self, address: &H160, slot: &U256, value: &U256, is_cold: bool) {
    //     println!(
    //         "sload: is_cold({}) {}[{:?}]={:?}",
    //         is_cold, address, slot, value
    //     );
    // }

    // fn sstore(
    //     &mut self,
    //     address: H160,
    //     slot: U256,
    //     new_value: U256,
    //     old_value: U256,
    //     original_value: U256,
    //     is_cold: bool,
    // ) {
    //     println!(
    //         "sstore: is_cold({}) {}[{:?}] {:?}(original:{:?}) => {:?}",
    //         is_cold, address, slot, old_value, original_value, new_value
    //     );
    // }

    fn call(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        call: H160,
        context: &revm::CallContext,
        transfer: &revm::Transfer,
        input: &bytes::Bytes,
        _gas_limit: u64,
        is_static: bool,
    ) -> (Return, Gas, Bytes) {
        println!(
            "SM CALL:   {:?},context:{:?}, is_static:{:?}, transfer:{:?}, input:{:?}",
            call,
            context,
            is_static,
            transfer,
            hex::encode(input),
        );
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn call_end(&mut self) {}

    fn create(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        caller: H160,
        scheme: &revm::CreateScheme,
        value: U256,
        init_code: &bytes::Bytes,
        gas: u64,
    ) -> (Return, Option<H160>, Gas, Bytes) {
        println!(
            "CREATE CALL: caller:{:?}, scheme:{:?}, value:{:?}, init_code:{:?}, gas:{:?}",
            caller,
            scheme,
            value,
            hex::encode(init_code),
            gas
        );
        (Return::Continue, None, Gas::new(0), Bytes::new())
    }

    fn create_end(&mut self) {}

    fn selfdestruct(&mut self) {
        //, address: H160, target: H160) {
        println!("SELFDESTRUCT on "); //{:?} target: {:?}", address, target);
    }
}
