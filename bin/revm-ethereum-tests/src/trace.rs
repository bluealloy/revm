use primitive_types::{H160, H256, U256};
pub use revm::{Control, Inspector};

#[derive(Clone)]
pub struct CustomPrintTracer {}

impl Inspector for CustomPrintTracer {
    // get opcode by calling `machine.contract.opcode(machine.program_counter())`.
    // all other information can be obtained from machine.
    fn step(&mut self, machine: &mut revm::Machine) {
        let opcode = match machine.contract.opcode(machine.program_counter()) {
            Ok(opcode) => opcode,
            Err(_) => return,
        };
        //if self.
        println!(
            "depth:{}, PC:{}, gas:{:#x}({}), OPCODE: {:?}({:?})  refund:{:#x}({}) Stack:{:?}, Data:",
            machine.call_depth,
            machine.program_counter(),
            machine.gas.remaining(),
            machine.gas.remaining(),
            opcode,
            opcode as u8,
            machine.gas.refunded(),
            machine.gas.refunded(),
            machine.stack.data(),
            //hex::encode(machine.memory.data()),
        );
    }

    fn load_account(&mut self, address: &H160) {
        println!("ACCOUNT LOADED:{:?}", address);
    }

    fn eval(&mut self, eval: &mut Control, machine: &mut revm::Machine) {}

    fn sload(&mut self, address: &H160, slot: &H256, value: &H256, is_cold: bool) {
        println!(
            "sload: is_cold({}) {}[{:?}]={:?}",
            is_cold, address, slot, value
        );
    }

    fn sstore(
        &mut self,
        address: H160,
        slot: H256,
        new_value: H256,
        old_value: H256,
        original_value: H256,
        is_cold: bool,
    ) {
        println!(
            "sstore: is_cold({}) {}[{:?}] {:?}(original:{:?}) => {:?}",
            is_cold, address, slot, old_value, original_value, new_value
        );
    }

    fn call(
        &mut self,
        call: H160,
        context: &revm::CallContext,
        transfer: &Option<revm::Transfer>,
        input: &bytes::Bytes,
        gas_limit: u64,
        is_static: bool,
    ) {
        println!(
            "SM CALL:   {:?},context:{:?}, is_static:{:?}, transfer:{:?}, input:{:?}",
            call,
            context,
            is_static,
            transfer,
            hex::encode(input),
        );
    }

    fn call_return(&mut self) {}

    fn create(
        &mut self,
        caller: H160,
        scheme: &revm::CreateScheme,
        value: U256,
        init_code: &bytes::Bytes,
        gas: u64,
    ) {
        println!(
            "CREATE CALL: caller:{:?}, scheme:{:?}, value:{:?}, init_code:{:?}, gas:{:?}",
            caller,
            scheme,
            value,
            hex::encode(init_code),
            gas
        );
    }

    fn create_return(&mut self, address: H256) {
        println!("CREATE Address:{:?}", address);
    }

    fn selfdestruct(&mut self){//, address: H160, target: H160) {
        println!("SELFDESTRUCT on ");//{:?} target: {:?}", address, target);
    }
}
