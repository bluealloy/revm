



pub use revm::{Control,Inspector};

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
            "depth:{}, PC:{}, gas:{:#x}({}), OPCODE: {:?}({:?})  refund:{:#x}({}) Stack.len:{:?}, Data:",
            machine.call_depth,
            machine.program_counter(),
            machine.gas.remaining(),
            machine.gas.remaining(),
            opcode,
            opcode as u8,
            machine.gas.refunded(),
            machine.gas.refunded(),
            machine.stack.data(),
            // hex::encode(machine.memory.data()),
        );
    }

    fn eval(&mut self, eval: &mut Control, machine: &mut revm::Machine) {
        
    }

    fn sload(&mut self, address: &primitive_types::H160, slot: &primitive_types::H256, value: &primitive_types::H256, is_cold: bool) {
        
    }

    fn sstore(
        &mut self,
        address: primitive_types::H160,
        slot: primitive_types::H256,
        new_value: primitive_types::H256,
        old_value: primitive_types::H256,
        original_value: primitive_types::H256,
        is_cold: bool,
    ) {
    }

    fn call(
        &mut self,
        call: primitive_types::H160,
        context: &revm::CallContext,
        transfer: &Option<revm::Transfer>,
        input: &bytes::Bytes,
        gas_limit: u64,
        is_static: bool,
    ) {
        println!(
            "SM CALL:   {:?},context:{:?}, is_static:{:?}, transfer:{:?}, input:{:?}",
            call, context, is_static, transfer, input,
        );
    }

    fn call_return(&mut self) {
        
    }

    fn create(
        &mut self,
        caller: primitive_types::H160,
        scheme: &revm::CreateScheme,
        value: primitive_types::U256,
        init_code: &bytes::Bytes,
        gas: u64,
    ) {
    }

    fn create_return(&mut self) {
        
    }

    fn selfdestruct(&mut self) {
        
    }

}