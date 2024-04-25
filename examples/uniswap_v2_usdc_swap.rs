use alloy_provider::ProviderBuilder;
use alloy_sol_types::{sol, SolCall, SolValue};
use revm::{
    db::{AlloyDB, CacheDB, EmptyDB, EmptyDBTyped},
    primitives::{
        address, keccak256, AccountInfo, Address, Bytes, ExecutionResult, Output, TransactTo, U256,
    },
    Database, Evm,
};
use std::ops::Div;
use std::{convert::Infallible, sync::Arc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ProviderBuilder::new()
        .on_reqwest_http(
            "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
                .parse()
                .unwrap(),
        )
        .unwrap();
    let client = Arc::new(client);
    let mut alloydb = AlloyDB::new(client, None);
    let mut cache_db = CacheDB::new(EmptyDB::default());

    // Random empty account
    let account = address!("18B06aaF27d44B756FCF16Ca20C1f183EB49111f");

    let weth = address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
    let usdc = address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
    let usdc_weth_pair = address!("B4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc");
    let uniswap_v2_router = address!("7a250d5630b4cf539739df2c5dacb4c659f2488d");

    // USDC uses a proxy pattern so we have to fetch implementation address
    let usdc_impl_slot: U256 = "0x7050c9e0f4ca769c69bd3a8ef740bc37934f8e2c036e5a723fd8ee048ed3f8c3"
        .parse()
        .unwrap();
    let usdc_impl_raw = alloydb.storage(usdc, usdc_impl_slot).unwrap();
    let usdc_impl: Address = format!("{:x}", usdc_impl_raw)[24..].parse().unwrap();

    // populate basic data
    for addr in [weth, usdc, usdc_weth_pair, uniswap_v2_router, usdc_impl] {
        let acc_info = alloydb.basic(addr).unwrap().unwrap();
        cache_db.insert_account_info(addr, acc_info);
    }

    cache_db
        .insert_account_storage(usdc, usdc_impl_slot, usdc_impl_raw)
        .unwrap();

    // populate WETH balance for USDC-WETH pair
    let weth_balance_slot = U256::from(3);

    let pair_weth_balance_slot = keccak256((usdc_weth_pair, weth_balance_slot).abi_encode());

    let value = alloydb
        .storage(weth, pair_weth_balance_slot.into())
        .unwrap();
    cache_db
        .insert_account_storage(weth, pair_weth_balance_slot.into(), value)
        .unwrap();

    // populate USDC balance for USDC-WETH pair
    let usdc_balance_slot = U256::from(9);

    let pair_usdc_balance_slot = keccak256((usdc_weth_pair, usdc_balance_slot).abi_encode());

    let value = alloydb
        .storage(usdc, pair_usdc_balance_slot.into())
        .unwrap();
    cache_db
        .insert_account_storage(usdc, pair_usdc_balance_slot.into(), value)
        .unwrap();

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

    // populate UniswapV2 pair slots
    // 6 - token0
    // 7 - token1
    // 8 - (reserve0, reserve1, blockTimestampLast)
    // 12 - unlocked

    let usdc_weth_pair_address = usdc_weth_pair;
    let pair_acc_info = alloydb.basic(usdc_weth_pair_address).unwrap().unwrap();
    cache_db.insert_account_info(usdc_weth_pair_address, pair_acc_info);

    for i in [6, 7, 8, 12] {
        let storage_slot = U256::from(i);
        let value = alloydb
            .storage(usdc_weth_pair_address, storage_slot)
            .unwrap();
        cache_db
            .insert_account_storage(usdc_weth_pair_address, storage_slot, value)
            .unwrap();
    }

    let acc_weth_balance_before = balance_of(weth, account, &mut cache_db).await?;
    println!("WETH balance before swap: {}", acc_weth_balance_before);
    let acc_usdc_balance_before = balance_of(usdc, account, &mut cache_db).await?;
    println!("USDC balance before swap: {}", acc_usdc_balance_before);

    let (reserve0, reserve1) = get_reserves(usdc_weth_pair, &mut cache_db).await?;

    let amount_in = one_ether.div(U256::from(10));

    // calculate USDC amount out
    let amount_out = get_amount_out(amount_in, reserve1, reserve0, &mut cache_db).await?;

    // transfer WETH to USDC-WETH pair
    transfer(account, usdc_weth_pair, amount_in, weth, &mut cache_db).await?;

    // execute low-level swap without using UniswapV2 router
    swap(
        account,
        usdc_weth_pair,
        account,
        amount_out,
        true,
        &mut cache_db,
    )
    .await?;

    let acc_weth_balance_after = balance_of(weth, account, &mut cache_db).await?;
    println!("WETH balance after swap: {}", acc_weth_balance_after);
    let acc_usdc_balance_after = balance_of(usdc, account, &mut cache_db).await?;
    println!("USDC balance after swap: {}", acc_usdc_balance_after);

    Ok(())
}

async fn balance_of(
    token: Address,
    address: Address,
    cache_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> anyhow::Result<u128> {
    sol! {
        function balanceOf(address account) public returns (uint256);
    }

    let encoded = balanceOfCall { account: address }.abi_encode();

    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            // 0x1 because calling USDC proxy from zero address fails
            tx.caller = address!("0000000000000000000000000000000000000001");
            tx.transact_to = TransactTo::Call(token);
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
        result => panic!("'balance_of' execution failed: {result:?}"),
    };

    let balance: u128 = <u128>::abi_decode(&value, false)?;

    Ok(balance)
}

async fn get_amount_out(
    amount_in: U256,
    reserve_in: U256,
    reserve_out: U256,
    cache_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> anyhow::Result<U256> {
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

    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            tx.caller = address!("0000000000000000000000000000000000000000");
            tx.transact_to = TransactTo::Call(uniswap_v2_router);
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
        result => panic!("'get_amount_out' execution failed: {result:?}"),
    };

    let result = <u128>::abi_decode(&value, false)?;

    Ok(U256::from(result))
}

async fn get_reserves(
    pair_address: Address,
    cache_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> anyhow::Result<(U256, U256)> {
    sol! {
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    }

    let encoded = getReservesCall {}.abi_encode();

    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            tx.caller = address!("0000000000000000000000000000000000000000");
            tx.transact_to = TransactTo::Call(pair_address);
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
        result => panic!("'get_reserves' execution failed: {result:?}"),
    };

    let (reserve0, reserve1, _): (u128, u128, u32) =
        <(u128, u128, u32)>::abi_decode(&value, false)?;

    Ok((U256::from(reserve0), U256::from(reserve1)))
}

async fn swap(
    from: Address,
    pool_address: Address,
    target: Address,
    amount_out: U256,
    is_token0: bool,
    cache_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> anyhow::Result<()> {
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

    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            tx.caller = from;
            tx.transact_to = TransactTo::Call(pool_address);
            tx.data = encoded.into();
            tx.value = U256::from(0);
        })
        .build();

    let ref_tx = evm.transact_commit().unwrap();

    match ref_tx {
        ExecutionResult::Success { .. } => {}
        result => panic!("'swap' execution failed: {result:?}"),
    };

    Ok(())
}

async fn transfer(
    from: Address,
    to: Address,
    amount: U256,
    token: Address,
    cache_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> anyhow::Result<()> {
    sol! {
        function transfer(address to, uint amount) external returns (bool);
    }

    let encoded = transferCall { to, amount }.abi_encode();

    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            tx.caller = from;
            tx.transact_to = TransactTo::Call(token);
            tx.data = encoded.into();
            tx.value = U256::from(0);
        })
        .build();

    let ref_tx = evm.transact_commit().unwrap();
    let result: bool = match ref_tx {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => {
            let success: bool = match <bool>::abi_decode(&value, false) {
                Ok(balance) => balance,
                Err(e) => panic!("'transfer' decode failed: {:?}", e),
            };
            success
        }
        result => panic!("'transfer' execution failed: {result:?}"),
    };

    if !result {
        panic!("transfer failed");
    }

    Ok(())
}
