use bytes::Bytes;
use primitive_types::{H160, U256};
pub use revm::Inspector;
use revm::{opcode, spec_opcode_gas, Database, EVMData, Gas, Return};

#[derive(Clone)]
pub struct CustomPrintTracer {
    /// We now batch continual gas_block in one go, that means we need to reduce it ifwe want to get
    /// correct gas remaining. Check revm/machine/contract/analize for more information
    reduced_gas_block: u64,
    full_gas_block: u64,
}

impl CustomPrintTracer {
    pub fn new() -> Self {
        Self {
            reduced_gas_block: 0,
            full_gas_block: 0,
        }
    }
}

impl<DB: Database> Inspector<DB> for CustomPrintTracer {
    fn initialize_machine(
        &mut self,
        machine: &mut revm::Machine,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        self.full_gas_block = machine.contract.first_gas_block();
        Return::Continue
    }

    // get opcode by calling `machine.contract.opcode(machine.program_counter())`.
    // all other information can be obtained from machine.
    fn step(
        &mut self,
        machine: &mut revm::Machine,
        data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        // Safety: casting. In analazis we are making this clame true that program counter will always
        // point to bytecode of the contract.
        let opcode = unsafe { *machine.program_counter };
        let opcode_str = opcode::OPCODE_JUMPMAP[opcode as usize];

        // calculate gas_block
        let infos = spec_opcode_gas(data.env.cfg.spec_id);
        let info = &infos[opcode as usize];

        println!(
            "depth:{}, PC:{}, gas:{:#x}({}), OPCODE: {:?}({:?})  refund:{:#x}({}) Stack:{:?}, Data:",
            machine.call_depth,
            machine.program_counter(),
            machine.gas.remaining()+self.full_gas_block-self.reduced_gas_block,
            machine.gas.remaining()+self.full_gas_block-self.reduced_gas_block,
            opcode_str.unwrap(),
            opcode,
            machine.gas.refunded(),
            machine.gas.refunded(),
            machine.stack.data(),
            //hex::encode(machine.memory.data()),
        );

        if info.gas_block_end {
            self.reduced_gas_block = 0;
            self.full_gas_block = machine.contract.gas_block(machine.program_counter());
        } else {
            self.reduced_gas_block += info.gas;
        }

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

    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _call: H160,
        _context: &revm::CallContext,
        _transfer: &revm::Transfer,
        _input: &Bytes,
        _gas_limit: u64,
        _remaining_gas: u64,
        _ret: Return,
        _out: &Bytes,
        _is_static: bool,
    ) {
    }

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

    fn create_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _caller: H160,
        _scheme: &revm::CreateScheme,
        _value: U256,
        _init_code: &Bytes,
        _ret: Return,
        _address: Option<H160>,
        _gas_limit: u64,
        _remaining_gas: u64,
        _out: &Bytes,
    ) {
    }

    fn selfdestruct(&mut self) {
        //, address: H160, target: H160) {
        println!("SELFDESTRUCT on "); //{:?} target: {:?}", address, target);
    }
}
