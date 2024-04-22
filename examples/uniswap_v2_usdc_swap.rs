use ethers_contract::BaseContract;
use ethers_core::{
    abi,
    abi::parse_abi,
    types::{Bytes, H160, U256},
    utils::to_checksum,
};
use ethers_providers::{Http, Provider};
use revm::{
    db::{CacheDB, EmptyDB, EmptyDBTyped, EthersDB},
    primitives::{
        address, keccak256, AccountInfo, ExecutionResult, Output, TransactTo, U256 as rU256,
    },
    Database, Evm,
};
use revm_precompile::Address;
use std::{convert::Infallible, sync::Arc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Provider::<Http>::try_from(
        "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
    )?;

    let client = Arc::new(client);
    let mut ethersdb = EthersDB::new(Arc::clone(&client), None).unwrap();
    let mut cache_db = CacheDB::new(EmptyDB::default());

    // Random empty account
    let account: H160 = "0x18B06aaF27d44B756FCF16Ca20C1f183EB49111f"
        .parse()
        .unwrap();
    let weth: H160 = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
        .parse()
        .unwrap();
    let usdc: H160 = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        .parse()
        .unwrap();
    let usdc_weth_pair: H160 = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"
        .parse()
        .unwrap();
    let uniswap_v2_router: H160 = "0x7a250d5630b4cf539739df2c5dacb4c659f2488d"
        .parse()
        .unwrap();

    // USDC uses a proxy pattern so we have to fetch implementation address
    let usdc_impl_slot: rU256 =
        "0x7050c9e0f4ca769c69bd3a8ef740bc37934f8e2c036e5a723fd8ee048ed3f8c3"
            .parse()
            .unwrap();
    let usdc_impl_raw = ethersdb.storage(to_address(usdc), usdc_impl_slot).unwrap();
    let usdc_impl: H160 = format!("{:x}", usdc_impl_raw)[24..].parse().unwrap();

    // populate basic data
    for addr in [weth, usdc, usdc_weth_pair, uniswap_v2_router, usdc_impl] {
        let addr = to_address(addr);
        let acc_info = ethersdb.basic(addr).unwrap().unwrap();
        cache_db.insert_account_info(addr, acc_info);
    }

    cache_db
        .insert_account_storage(to_address(usdc), usdc_impl_slot, usdc_impl_raw)
        .unwrap();

    // populate WETH balance for USDC-WETH pair
    let weth_balance_slot = U256::from(3);

    let pair_weth_balance_slot = keccak256(abi::encode(&[
        abi::Token::Address(usdc_weth_pair),
        abi::Token::Uint(weth_balance_slot),
    ]));

    let value = ethersdb
        .storage(to_address(weth), pair_weth_balance_slot.into())
        .unwrap();
    cache_db
        .insert_account_storage(to_address(weth), pair_weth_balance_slot.into(), value)
        .unwrap();

    // populate USDC balance for USDC-WETH pair
    let usdc_balance_slot = U256::from(9);

    let pair_usdc_balance_slot = keccak256(abi::encode(&[
        abi::Token::Address(usdc_weth_pair),
        abi::Token::Uint(usdc_balance_slot),
    ]));

    let value = ethersdb
        .storage(to_address(usdc), pair_usdc_balance_slot.into())
        .unwrap();
    cache_db
        .insert_account_storage(to_address(usdc), pair_usdc_balance_slot.into(), value)
        .unwrap();

    // give our test account some fake WETH and ETH
    let one_ether = rU256::from(1_000_000_000_000_000_000u128);
    let hashed_acc_balance_slot = keccak256(abi::encode(&[
        abi::Token::Address(account),
        abi::Token::Uint(weth_balance_slot),
    ]));
    cache_db
        .insert_account_storage(to_address(weth), hashed_acc_balance_slot.into(), one_ether)
        .unwrap();

    let acc_info = AccountInfo {
        nonce: 0_u64,
        balance: one_ether,
        code_hash: keccak256(Bytes::new()),
        code: None,
    };
    cache_db.insert_account_info(to_address(account), acc_info);

    // populate UniswapV2 pair slots
    let usdc_weth_pair_address = to_address(usdc_weth_pair);
    let pair_acc_info = ethersdb.basic(usdc_weth_pair_address).unwrap().unwrap();
    cache_db.insert_account_info(usdc_weth_pair_address, pair_acc_info);
    for i in 0..=12 {
        let storage_slot = rU256::from(i);
        let value = ethersdb
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

    let amount_in = U256::from_dec_str("100000000000000000").unwrap(); // 1/10 ETH

    // calculate USDC amount out
    let amount_out = get_amount_out(amount_in, reserve1, reserve0, &mut cache_db).await?;

    // transfer WETH to USDC-WETH pair
    transfer(account, usdc_weth_pair, amount_in, weth, &mut cache_db).await?;

    // exeucte low-level swap without using UniswapV2 router
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
    token: H160,
    address: H160,
    cache_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> anyhow::Result<u128> {
    let abi = BaseContract::from(parse_abi(&[
        "function balanceOf(address) public returns (uint256)",
    ])?);

    let encoded_balance = abi.encode("balanceOf", address)?;

    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            // 0x1 because calling USDC proxy from zero address fails
            tx.caller = address!("0000000000000000000000000000000000000001");
            tx.transact_to = TransactTo::Call(to_address(token));
            tx.data = encoded_balance.0.into();
            tx.value = rU256::from(0);
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

    let balance: u128 = abi.decode_output("balanceOf", value)?;
    Ok(balance)
}

async fn get_amount_out(
    amount_in: U256,
    reserve_in: U256,
    reserve_out: U256,
    cache_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> anyhow::Result<U256> {
    let uniswap_v2_router: H160 = "0x7a250d5630b4cf539739df2c5dacb4c659f2488d"
        .parse()
        .unwrap();
    let router_abi = BaseContract::from(parse_abi(&[
      "function getAmountOut(uint amountIn, uint reserveIn, uint reserveOut) external pure returns (uint amountOut)"
    ])?);

    let encoded = router_abi.encode("getAmountOut", (amount_in, reserve_in, reserve_out))?;

    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            tx.caller = address!("0000000000000000000000000000000000000000");
            tx.transact_to = TransactTo::Call(to_address(uniswap_v2_router));
            tx.data = encoded.0.into();
            tx.value = rU256::from(0);
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

    let amount_out: u128 = router_abi.decode_output("getAmountOut", value)?;
    Ok(U256::from(amount_out))
}

async fn get_reserves(
    pair_address: H160,
    cache_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> anyhow::Result<(U256, U256)> {
    let abi = BaseContract::from(parse_abi(&[
        "function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)"
    ])?);

    let encoded = abi.encode("getReserves", ())?;
    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            tx.caller = address!("0000000000000000000000000000000000000000");
            tx.transact_to = TransactTo::Call(to_address(pair_address));
            tx.data = encoded.0.into();
            tx.value = rU256::from(0);
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

    let (reserve0, reserve1, _): (u128, u128, u32) = abi.decode_output("getReserves", value)?;
    Ok((U256::from(reserve0), U256::from(reserve1)))
}

async fn swap(
    from: H160,
    pool_address: H160,
    target: H160,
    amount_out: U256,
    is_token0: bool,
    cache_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> anyhow::Result<()> {
    let abi = BaseContract::from(parse_abi(&[
        "function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external",
    ])?);

    let from = to_address(from);
    let pool_address = to_address(pool_address);

    let amount0_out = if is_token0 { amount_out } else { U256::from(0) };
    let amount1_out = if is_token0 { U256::from(0) } else { amount_out };

    let encoded = abi.encode("swap", (amount0_out, amount1_out, target, Bytes::new()))?;
    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            tx.caller = from;
            tx.transact_to = TransactTo::Call(pool_address);
            tx.data = encoded.0.into();
            tx.value = rU256::from(0);
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
    from: H160,
    to: H160,
    amount: U256,
    token: H160,
    cache_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> anyhow::Result<()> {
    let abi = BaseContract::from(parse_abi(&[
        "function transfer(address to, uint amount) returns (bool)",
    ])?);

    let from = to_address(from);

    let encoded = abi.encode("transfer", (to, amount))?;

    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            tx.caller = from;
            tx.transact_to = TransactTo::Call(to_address(token));
            tx.data = encoded.0.into();
            tx.value = rU256::from(0);
        })
        .build();

    let ref_tx = evm.transact_commit().unwrap();
    let result: bool = match ref_tx {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => {
            let success: bool = abi.decode_output("transfer", value)?;
            success
        }
        result => panic!("'transfer' execution failed: {result:?}"),
    };

    if !result {
        panic!("transfer failed");
    }

    Ok(())
}

fn to_address(h160: H160) -> Address {
    Address::parse_checksummed(to_checksum(&h160, None), None).unwrap()
}
