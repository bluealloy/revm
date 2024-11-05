use safer_ffi::prelude::*;
use core::ffi::c_char;
use core::ffi::c_void;
use core::ptr;
use std::boxed::Box;


use alloy_transport_http::Http;

use alloy_provider::{Provider, ProviderBuilder, ReqwestProvider};
use database::{AlloyDB, CacheDB, State, StateBuilder};
use crate::database_interface::async_db::WrapDatabaseAsync;
use alloy_eips::{BlockId};
use alloy_provider::network::Ethereum;
use alloy_sol_types::private::Address;
use reqwest::Client;
use primitives::Bytes;
use database::InMemoryDB;
use primitives::alloy_primitives::BlockNumber;
use primitives::{TxKind, U256};
use crate::evm::Evm;
use wiring::default::TxEnv;
use crate::EvmWiring;
use wiring::result::{EVMError, EVMResult, EVMResultGeneric, ExecutionResult, ResultAndState};
use wiring::{EthereumWiring};
use crate::specification::hardfork::SpecId;

// FFI-friendly enum for #[derive(Debug)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum Hardfork {
    Istanbul,
    Berlin,
    London,
}

// Define a struct for call parameters if necessary
#[derive_ReprC]
#[repr(C)]
pub struct CallParams {
    pub from: [u8; 20], //Address in bytes
    pub to: [u8; 20],   // Address in bytes
    pub data: *const u8, // Pointer to data
    pub data_len: usize, // Length of data
    pub gas: u64,        // Gas limit
    pub value: u64,
}

// Define a struct to hold the result
#[repr(C)]
pub struct REVMResult {
    pub status_code: i32,
    pub output: *const c_char,
}

// // Function to create an Evm instance based on hardfork and in-memory database
// #[ffi_export]
fn create_evm(hard_fork: u8, rpc_url: char_p::Ref<'_>)
              -> *mut c_void {
    // Set up an in-memory database
    let db = InMemoryDB::default();
    let rpc_url = rpc_url.to_str();

    // Map the FFI-friendly Hardfork to SpecId
    let spec_id = SpecId::try_from_u8(hard_fork).unwrap_or_else(|| { SpecId::CANCUN });

    // Create the EvmHandler with the specified hardfork
    let handler = EthereumWiring::handler(spec_id);
    // Convert the C string to a Rust string


    let client = ProviderBuilder::new().on_http(rpc_url.parse().unwrap());

    let block_number = match client.get_block_number().as_ready() {
        None => { BlockNumber::from(0u64) }
        Some(res) => {
            let i = res.as_ref().unwrap();
            *i
        }
    };
    let state_db = WrapDatabaseAsync::new(AlloyDB::new(client, BlockId::from(block_number))).unwrap();
    let cache_db: CacheDB<_> = CacheDB::new(state_db);
    let mut state = StateBuilder::new_with_database(cache_db).build();

    // let mainnet = EvmHandler::<'_, EthereumWiring<EmptyDB,()>>::mainnet_with_spec(SpecId::CANCUN);


    let mut evm: Evm<'_, EthereumWiring<&mut State<CacheDB<WrapDatabaseAsync<AlloyDB<Http<Client>, Ethereum, ReqwestProvider>>>>, ()>> = Evm::builder().
        with_handler(handler).
        with_db(&mut state).build();
    // Set up the EVM context


    // Return a raw pointer to the Evm instance
    Box::into_raw(Box::new(evm)) as *mut c_void
}

#[derive_ReprC]
#[repr(C)]
pub struct CallResult {
    pub success: bool,
    pub gas_used: u64,
    pub output_data: *const u8,
    pub output_len: usize,
}

impl From<EVMResult<EthereumWiring<&mut State<CacheDB<WrapDatabaseAsync<AlloyDB<Http<Client>, Ethereum, ReqwestProvider>>>>, ()>>> for CallResult {
    fn from(result: EVMResult<EthereumWiring<&mut State<CacheDB<WrapDatabaseAsync<AlloyDB<Http<Client>, Ethereum, ReqwestProvider>>>>, ()>>) -> Self {
        match result {
            Ok(execution_result) => {
                let output = execution_result.result.output().unwrap().to_vec();
                let output_len = output.len();
                let output_data = output.into_boxed_slice().as_ptr();

                Self {
                    success: true,
                    gas_used: execution_result.result.gas_used(),
                    output_data,
                    output_len,
                }
            }
            Err(err) => {
                Self {
                    success: false,
                    gas_used: 0,
                    output_data: ptr::null(),
                    output_len: 0,
                }
            }
        }
    }
}
#[ffi_export]
fn evm_call(evm_ptr: *mut c_void, params: CallParams) -> repr_c::Box<CallResult> {
    // Safety: Convert raw pointer to a mutable reference to Evm
    let mut evm = unsafe { *(evm_ptr as *mut Evm<EthereumWiring<&mut State<CacheDB<WrapDatabaseAsync<AlloyDB<Http<Client>, Ethereum, ReqwestProvider>>>>, ()>>) };
    let res = evm.transact().unwrap();
    // Parse the address and data from params
    let to = if let Some(to_slice) = params.to.as_ref() {
        TxKind::Call(Address::from_slice(to_slice))
    } else {
        TxKind::Create
    };
    let caller = Address::from_slice(&params.from);
    let data = unsafe { std::slice::from_raw_parts(params.data, params.data_len) };
    let calldata = Bytes::from(data.to_vec());
    // Set up TxEnv for the transaction
    let tx_env = TxEnv {
        caller, // Set the caller if available
        gas_limit: params.gas.into(),
        value: U256::from(params.value),
        transact_to: to,
        data: Bytes::from(calldata),
        ..Default::default()
    };
    evm = evm.modify().with_tx_env(tx_env).build();
    let result = evm.transact().unwrap();
    // Return the result in an FFI-compatible structure
    repr_c::Box::new(CallResult::from(result))
}



