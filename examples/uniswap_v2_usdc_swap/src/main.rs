//! Example of uniswap getReserves() call emulation.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use alloy_eips::BlockId;
use alloy_provider::{network::Ethereum, ProviderBuilder, RootProvider};
use alloy_sol_types::{sol, SolCall, SolValue};
use alloy_transport_http::Http;
use anyhow::{anyhow, Result};
use database::{AlloyDB, CacheDB};
use reqwest::Client;
use revm::{
    database_interface::WrapDatabaseAsync,
    primitives::{address, keccak256, Address, Bytes, TxKind, U256},
    state::AccountInfo,
    wiring::{
        result::{ExecutionResult, Output},
        EthereumWiring,
    },
    Evm,
};
use std::ops::Div;

type AlloyCacheDB =
    CacheDB<WrapDatabaseAsync<AlloyDB<Http<Client>, Ethereum, RootProvider<Http<Client>>>>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up the HTTP transport which is consumed by the RPC client.
    let rpc_url = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27".parse()?;

    // create ethers client and wrap it in Arc<M>
    let client = ProviderBuilder::new().on_http(rpc_url);

    let alloy = WrapDatabaseAsync::new(AlloyDB::new(client, BlockId::latest())).unwrap();
    let mut cache_db = CacheDB::new(alloy);

    // Random empty account
    let account = address!("18B06aaF27d44B756FCF16Ca20C1f183EB49111f");

    let weth = address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
    let usdc = address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
    let usdc_weth_pair = address!("B4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc");

    let weth_balance_slot = U256::from(3);

    // give our test account some fake WETH and ETH
    let one_ether = U256::from(1_000_000_000_000_000_000u128);
    let hashed_acc_balance_slot = keccak256((account, weth_balance_slot).abi_encode());
    cache_db
        .insert_account_storage(weth, hashed_acc_balance_slot.into(), one_ether)
        .unwrap();

    let acc_info = AccountInfo {
        nonce: 0_u64,
        balance: one_ether,
        code_hash: keccak256(Bytes::new()),
        code: None,
    };
    cache_db.insert_account_info(account, acc_info);

    let acc_weth_balance_before = balance_of(weth, account, &mut cache_db)?;
    println!("WETH balance before swap: {}", acc_weth_balance_before);
    let acc_usdc_balance_before = balance_of(usdc, account, &mut cache_db)?;
    println!("USDC balance before swap: {}", acc_usdc_balance_before);

    let (reserve0, reserve1) = get_reserves(usdc_weth_pair, &mut cache_db)?;

    let amount_in = one_ether.div(U256::from(10));

    // calculate USDC amount out
    let amount_out = get_amount_out(amount_in, reserve1, reserve0, &mut cache_db).await?;

    // transfer WETH to USDC-WETH pair
    transfer(account, usdc_weth_pair, amount_in, weth, &mut cache_db)?;

    // execute low-level swap without using UniswapV2 router
    swap(
        account,
        usdc_weth_pair,
        account,
        amount_out,
        true,
        &mut cache_db,
    )?;

    let acc_weth_balance_after = balance_of(weth, account, &mut cache_db)?;
    println!("WETH balance after swap: {}", acc_weth_balance_after);
    let acc_usdc_balance_after = balance_of(usdc, account, &mut cache_db)?;
    println!("USDC balance after swap: {}", acc_usdc_balance_after);

    println!("OK");
    Ok(())
}

fn balance_of(token: Address, address: Address, alloy_db: &mut AlloyCacheDB) -> Result<U256> {
    sol! {
        function balanceOf(address account) public returns (uint256);
    }

    let encoded = balanceOfCall { account: address }.abi_encode();

    let mut evm = Evm::<EthereumWiring<&mut AlloyCacheDB, ()>>::builder()
        .with_db(alloy_db)
        .with_default_ext_ctx()
        .modify_tx_env(|tx| {
            // 0x1 because calling USDC proxy from zero address fails
            tx.caller = address!("0000000000000000000000000000000000000001");
            tx.transact_to = TxKind::Call(token);
            tx.data = encoded.into();
            tx.value = U256::from(0);
        })
        .build();

    let ref_tx = evm.transact().unwrap();
    let result = ref_tx.result;

    let value = match result {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => value,
        result => return Err(anyhow!("'balanceOf' execution failed: {result:?}")),
    };

    let balance = <U256>::abi_decode(&value, false)?;

    Ok(balance)
}

async fn get_amount_out(
    amount_in: U256,
    reserve_in: U256,
    reserve_out: U256,
    cache_db: &mut AlloyCacheDB,
) -> Result<U256> {
    let uniswap_v2_router = address!("7a250d5630b4cf539739df2c5dacb4c659f2488d");
    sol! {
        function getAmountOut(uint amountIn, uint reserveIn, uint reserveOut) external pure returns (uint amountOut);
    }

    let encoded = getAmountOutCall {
        amountIn: amount_in,
        reserveIn: reserve_in,
        reserveOut: reserve_out,
    }
    .abi_encode();

    let mut evm = Evm::<EthereumWiring<&mut AlloyCacheDB, ()>>::builder()
        .with_db(cache_db)
        .with_default_ext_ctx()
        .modify_tx_env(|tx| {
            tx.caller = address!("0000000000000000000000000000000000000000");
            tx.transact_to = TxKind::Call(uniswap_v2_router);
            tx.data = encoded.into();
            tx.value = U256::from(0);
        })
        .build();

    let ref_tx = evm.transact().unwrap();
    let result = ref_tx.result;

    let value = match result {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => value,
        result => return Err(anyhow!("'getAmountOut' execution failed: {result:?}")),
    };

    let amount_out = <U256>::abi_decode(&value, false)?;

    Ok(amount_out)
}

fn get_reserves(pair_address: Address, cache_db: &mut AlloyCacheDB) -> Result<(U256, U256)> {
    sol! {
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    }

    let encoded = getReservesCall {}.abi_encode();

    let mut evm = Evm::<EthereumWiring<&mut AlloyCacheDB, ()>>::builder()
        .with_db(cache_db)
        .with_default_ext_ctx()
        .modify_tx_env(|tx| {
            tx.caller = address!("0000000000000000000000000000000000000000");
            tx.transact_to = TxKind::Call(pair_address);
            tx.data = encoded.into();
            tx.value = U256::from(0);
        })
        .build();

    let ref_tx = evm.transact().unwrap();
    let result = ref_tx.result;

    let value = match result {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => value,
        result => return Err(anyhow!("'getReserves' execution failed: {result:?}")),
    };

    let (reserve0, reserve1, _) = <(U256, U256, u32)>::abi_decode(&value, false)?;

    Ok((reserve0, reserve1))
}

fn swap(
    from: Address,
    pool_address: Address,
    target: Address,
    amount_out: U256,
    is_token0: bool,
    cache_db: &mut AlloyCacheDB,
) -> Result<()> {
    sol! {
        function swap(uint amount0Out, uint amount1Out, address target, bytes callback) external;
    }

    let amount0_out = if is_token0 { amount_out } else { U256::from(0) };
    let amount1_out = if is_token0 { U256::from(0) } else { amount_out };

    let encoded = swapCall {
        amount0Out: amount0_out,
        amount1Out: amount1_out,
        target,
        callback: Bytes::new(),
    }
    .abi_encode();

    let mut evm = Evm::<EthereumWiring<&mut AlloyCacheDB, ()>>::builder()
        .with_db(cache_db)
        .with_default_ext_ctx()
        .modify_tx_env(|tx| {
            tx.caller = from;
            tx.transact_to = TxKind::Call(pool_address);
            tx.data = encoded.into();
            tx.value = U256::from(0);
            tx.nonce = 1;
        })
        .build();

    let ref_tx = evm.transact_commit().unwrap();

    match ref_tx {
        ExecutionResult::Success { .. } => {}
        result => return Err(anyhow!("'swap' execution failed: {result:?}")),
    };

    Ok(())
}

fn transfer(
    from: Address,
    to: Address,
    amount: U256,
    token: Address,
    cache_db: &mut AlloyCacheDB,
) -> Result<()> {
    sol! {
        function transfer(address to, uint amount) external returns (bool);
    }

    let encoded = transferCall { to, amount }.abi_encode();

    let mut evm = Evm::<EthereumWiring<&mut AlloyCacheDB, ()>>::builder()
        .with_db(cache_db)
        .with_default_ext_ctx()
        .modify_tx_env(|tx| {
            tx.caller = from;
            tx.transact_to = TxKind::Call(token);
            tx.data = encoded.into();
            tx.value = U256::from(0);
        })
        .build();

    let ref_tx = evm.transact_commit().unwrap();
    let success: bool = match ref_tx {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => <bool>::abi_decode(&value, false)?,
        result => return Err(anyhow!("'transfer' execution failed: {result:?}")),
    };

    if !success {
        return Err(anyhow!("'transfer' failed"));
    }

    Ok(())
}
