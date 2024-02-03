#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use fluentbase_sdk::evm::{ExecutionContext, U256};
use revm_interpreter::opcode::make_instruction_table;
use revm_interpreter::{Contract, FluentHost, InstructionResult, Interpreter, SharedMemory};
use revm_precompile::primitives::{BlockEnv, Bytecode, CfgEnv, Env, LondonSpec, TxEnv, LONDON};

#[no_mangle]
extern "C" fn main() {
    // read input
    let mut ctx = ExecutionContext::default();
    // init contract
    let contract = Contract::new(
        ExecutionContext::contract_input(),
        Bytecode::new_raw(ExecutionContext::contract_bytecode()),
        ExecutionContext::contract_code_hash(),
        ExecutionContext::contract_address(),
        ExecutionContext::contract_caller(),
        ExecutionContext::contract_value(),
    );
    // read env input (we use json for testing purposes)
    let mut cfg_env = CfgEnv::default();
    cfg_env.chain_id = ExecutionContext::env_chain_id().clone();
    cfg_env.spec_id = LONDON;
    let env = Env {
        cfg: cfg_env,
        block: BlockEnv {
            number: U256::from(ExecutionContext::block_number().clone()),
            coinbase: ExecutionContext::block_coinbase().clone(),
            timestamp: U256::from(ExecutionContext::block_timestamp().clone()),
            gas_limit: U256::from(ExecutionContext::block_gas_limit().clone()),
            basefee: U256::from(ExecutionContext::block_base_fee().clone()),
            difficulty: U256::from(ExecutionContext::block_difficulty().clone()),
            ..Default::default()
        },
        tx: TxEnv {
            ..Default::default()
        },
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
        // for log in fluent_host.log.iter() {
        //     ctx.emit_log(log.topics.clone(), log.data.to_vec());
        // }
        (return_code, vm.return_offset, vm.return_len)
    };
    let return_data = shared_memory.slice(return_offset, return_len);
    if return_data.len() > 0 {
        // ctx.emit_return(return_data);
    }
    match return_code {
        InstructionResult::Continue
        | InstructionResult::Stop
        | InstructionResult::Return
        | InstructionResult::SelfDestruct => {}
        _ => {
            ctx.exit(-1);
        }
    }
    ctx.exit(0);
}
