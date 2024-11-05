// use std::sync::Arc;
// use ethers_providers::{Http, Provider, Middleware};
// use ethers_core::types::{BlockId, BlockNumberOrTag, U256};
// use evm::{
//     Evm, Context, State, StateBuilder, CacheDB, 
//     database_interface::WrapDatabaseAsync, 
//     precompiles::EthereumWiring,
// };
// 
// // FFI-friendly enum for specifying the hardfork
// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// pub enum Hardfork {
//     Istanbul,
//     Berlin,
//     London,
// }
// 
// #[ffi_export]
// fn create_evm(
//     hardfork: Hardfork, 
//     rpc_url: char_p::Ref<'_>, 
//     block_number: u64,
// ) -> repr_c::Box<Evm<'static, EthereumWiring<CacheDB<WrapDatabaseAsync<Provider<Http>>, ()>>>> {
//     // Convert the FFI-compatible char_p::Ref to a Rust &str
//     let rpc_url = rpc_url.to_str();
// 
//     // Set up the HTTP transport for the RPC client
//     let provider = Provider::<Http>::try_from(rpc_url).expect("Failed to create provider");
//     let client = Arc::new(provider);
// 
//     // Fetch the block synchronously
//     let block = match client
//         .get_block_by_number(BlockNumberOrTag::Number(block_number))
//         .expect("Failed to send request")
//     {
//         Some(block) => block,
//         None => {
//             eprintln!("Block not found");
//             return repr_c::Box::null();
//         }
//     };
// 
//     let previous_block_number = block_number - 1;
//     let prev_id: BlockId = previous_block_number.into();
// 
//     // Create the state database with caching
//     let state_db = WrapDatabaseAsync::new(AlloyDB::new(client.clone(), prev_id))
//         .expect("Failed to create state database");
//     let cache_db: CacheDB<_> = CacheDB::new(state_db);
//     let mut state = StateBuilder::new_with_database(cache_db).build();
// 
//     // Create the EVM instance
//     let mut evm = Evm::<EthereumWiring<_, _>>::builder()
//         .with_db(&mut state)
//         .modify_block_env(|b| {
//             b.number = U256::from(block.header.number);
//             b.coinbase = block.header.miner;
//             b.timestamp = U256::from(block.header.timestamp);
//             b.difficulty = block.header.difficulty;
//             b.gas_limit = U256::from(block.header.gas_limit);
//             b.basefee = block.header.base_fee_per_gas.map(U256::from).unwrap_or_default();
//         })
//         .modify_cfg_env(|c| {
//             c.chain_id = 1; // Set the chain ID to 1 (Ethereum Mainnet)
//         })
//         .build();
// 
//     // Return the EVM instance wrapped in repr_c::Box
//     repr_c::Box::new(evm)
// }