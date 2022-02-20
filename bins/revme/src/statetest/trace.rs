use bytes::Bytes;
use primitive_types::{H160, U256};
pub use revm::Inspector;
use revm::{opcode, spec_opcode_gas, Database, EVMData, Gas, Return, CallInputs};

#[derive(Clone)]
pub struct CustomPrintTracer {
    /// We now batch continual gas_block in one go, that means we need to reduce it ifwe want to get
    /// correct gas remaining. Check revm/interp/contract/analize for more information
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
    fn initialize_interp(
        &mut self,
        interp: &mut revm::Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        self.full_gas_block = interp.contract.first_gas_block();
        Return::Continue
    }

    // get opcode by calling `interp.contract.opcode(interp.program_counter())`.
    // all other information can be obtained from interp.
    fn step(
        &mut self,
        interp: &mut revm::Interpreter,
        data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> Return {
        // Safety: casting. In analazis we are making this clame true that program counter will always
        // point to bytecode of the contract.
        let opcode = unsafe { *interp.program_counter };
        let opcode_str = opcode::OPCODE_JUMPMAP[opcode as usize];

        // calculate gas_block
        let infos = spec_opcode_gas(data.env.cfg.spec_id);
        let info = &infos[opcode as usize];

        println!(
            "depth:{}, PC:{}, gas:{:#x}({}), OPCODE: {:?}({:?})  refund:{:#x}({}) Stack:{:?}, Data:",
            interp.call_depth,
            interp.program_counter(),
            interp.gas.remaining()+self.full_gas_block-self.reduced_gas_block,
            interp.gas.remaining()+self.full_gas_block-self.reduced_gas_block,
            opcode_str.unwrap(),
            opcode,
            interp.gas.refunded(),
            interp.gas.refunded(),
            interp.stack.data(),
            //hex::encode(interp.memory.data()),
        );

        if info.gas_block_end {
            self.reduced_gas_block = 0;
            self.full_gas_block = interp.contract.gas_block(interp.program_counter());
        } else {
            self.reduced_gas_block += info.gas;
        }

        Return::Continue
    }

    fn step_end(
        &mut self,
        _interp: &mut revm::Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
        _eval: revm::Return,
    ) -> Return {
        Return::Continue
    }

    fn call(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        inputs: &CallInputs,
        is_static: bool,
    ) -> (Return, Gas, Bytes) {
        println!(
            "SM CALL:   {:?},context:{:?}, is_static:{:?}, transfer:{:?}, input:{:?}",
            inputs.code_address,
            inputs.context,
            is_static,
            inputs.transfer,
            hex::encode(&inputs.input),
        );
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CallInputs,
        _remaining_gas: Gas,
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
