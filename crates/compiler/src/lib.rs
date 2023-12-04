use fluentbase_sdk::evm::{
    contract_read_address, contract_read_bytecode, contract_read_caller, contract_read_env,
    contract_read_hash, contract_read_input, contract_read_value, ContractInput,
};
#[no_std]
use fluentbase_sdk::{SysPlatformSDK, SDK};
use revm_interpreter::opcode::make_instruction_table;
use revm_interpreter::{Contract, FluentHost, InstructionResult, Interpreter, SharedMemory};
use revm_precompile::primitives::{
    Address, Bytecode, Bytes, Env, LondonSpec, TransactTo, B256, U256,
};

#[no_mangle]
extern "C" fn main() {
    // read input
    let input = contract_read_input();
    let bytecode = contract_read_bytecode();
    let hash = contract_read_hash();
    let address = contract_read_address();
    let caller = contract_read_caller();
    let value = contract_read_value();
    // init contract
    let contract = Contract::new(
        Bytes::from(input),
        Bytecode::new_raw(Bytes::from(bytecode)),
        B256::from(hash),
        Address::from(address),
        Address::from(caller),
        U256::from(value),
    );
    // read env input (we use json for testing purposes)
    let env = {
        let json_env = contract_read_env();
        let env: Env = serde_json::from_slice(&json_env.as_slice()).unwrap();
        env
    };
    let mut shared_memory = SharedMemory::new();
    let (return_code, return_offset, return_len) = {
        let mut vm = Interpreter::new(
            Box::new(contract),
            env.tx.gas_limit,
            false,
            &mut shared_memory,
        );
        let mut fluent_host = FluentHost::new(env);
        let return_code = vm.run(
            &make_instruction_table::<FluentHost, LondonSpec>(),
            &mut fluent_host,
        );
        (return_code, vm.return_offset, vm.return_len)
    };
    let return_data = shared_memory.slice(return_offset, return_len);
    if return_data.len() > 0 {
        SDK::sys_write(return_data);
    }
    match return_code {
        InstructionResult::Continue
        | InstructionResult::Stop
        | InstructionResult::Return
        | InstructionResult::SelfDestruct => {}
        _ => {
            SDK::sys_halt(-1);
        }
    }
}
