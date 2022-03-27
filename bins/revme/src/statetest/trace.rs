use bytes::Bytes;
use primitive_types::H160;
pub use revm::Inspector;
use revm::{
    opcode::{self},
    spec_opcode_gas, CallInputs, CreateInputs, Database, EVMData, Gas, Return,
};

#[derive(Clone)]
pub struct CustomPrintTracer {
    /// We now batch continual gas_block in one go, that means we need to reduce it ifwe want to get
    /// correct gas remaining. Check revm/interp/contract/analyze for more information
    reduced_gas_block: u64,
    full_gas_block: u64,
    is_return: bool,
}

impl CustomPrintTracer {
    pub fn new() -> Self {
        Self {
            reduced_gas_block: 0,
            full_gas_block: 0,
            is_return: false,
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
        // Safety: casting. In analysis we are making this clame true that program counter will always
        // point to bytecode of the contract.
        let opcode = unsafe { *interp.program_counter };
        let opcode_str = opcode::OPCODE_JUMPMAP[opcode as usize];

        // calculate gas_block
        let infos = spec_opcode_gas(data.env.cfg.spec_id);
        let info = &infos[opcode as usize];

        println!("block_end:{} full_block:{:?} reduced_block:{:?}",info.is_gas_block_end(),self.full_gas_block,self.reduced_gas_block);
        println!(
            "depth:{}, PC:{}, gas:{:#x}({}), OPCODE: {:?}({:?})  refund:{:#x}({}) Stack:, Data:",
            interp.call_depth,
            interp.program_counter(),
            interp.gas.remaining()+self.full_gas_block-self.reduced_gas_block,
            interp.gas.remaining()+self.full_gas_block-self.reduced_gas_block,
            opcode_str.unwrap(),
            opcode,
            interp.gas.refunded(),
            interp.gas.refunded(),
            //interp.stack.data(),
            //hex::encode(interp.memory.data()),
        );

        if info.is_gas_block_end() {
            self.reduced_gas_block = 0;
            self.full_gas_block = interp.contract.gas_block(interp.program_counter());
        } else {
            self.reduced_gas_block += info.gas;
        }

        Return::Continue
    }

    fn step_end(
        &mut self,
        interp: &mut revm::Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
        _eval: revm::Return,
    ) -> Return {
        if self.is_return {

            // we are okey to decrement PC by one as it is return of call
            let previous_pc = interp.program_counter()-1;
            self.full_gas_block = interp.contract.gas_block(previous_pc);
            self.is_return = false;
        }
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
        self.is_return = true;
        (ret, remaining_gas, out)
    }

    fn create_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CreateInputs,
        ret: Return,
        address: Option<H160>,
        remaining_gas: Gas,
        out: Bytes,
    ) -> (Return, Option<H160>, Gas, Bytes) {
        self.is_return = true;
        (ret, address, remaining_gas, out)
    }
    fn call(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        inputs: &mut CallInputs,
        is_static: bool,
    ) -> (Return, Gas, Bytes) {
        println!(
            "SM CALL:   {:?},context:{:?}, is_static:{:?}, transfer:{:?}, input:{:?}",
            inputs.contract,
            inputs.context,
            is_static,
            inputs.transfer,
            hex::encode(&inputs.input),
        );
        (Return::Continue, Gas::new(0), Bytes::new())
    }

    fn create(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        inputs: &mut CreateInputs,
    ) -> (Return, Option<H160>, Gas, Bytes) {
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
        //, address: H160, target: H160) {
        println!("SELFDESTRUCT on "); //{:?} target: {:?}", address, target);
    }
}
