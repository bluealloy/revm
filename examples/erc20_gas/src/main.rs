//! Example of a custom handler for ERC20 gas calculation.
//!
//! Gas is going to be deducted from ERC20 token.

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use alloy_provider::{network::Ethereum, DynProvider, Provider, ProviderBuilder};
use alloy_sol_types::SolValue;
use anyhow::Result;
use exec::transact_erc20evm_commit;
use revm::{
    context_interface::{
        result::{InvalidHeader, InvalidTransaction},
        ContextTr, JournalTr,
    },
    database::{AlloyDB, BlockId, CacheDB},
    database_interface::WrapDatabaseAsync,
    primitives::{
        address, hardfork::SpecId, keccak256, Address, StorageValue, TxKind, KECCAK_EMPTY, U256,
    },
    state::AccountInfo,
    Context, Database, MainBuilder, MainContext,
};

pub mod exec;
pub mod handler;

type AlloyCacheDB = CacheDB<WrapDatabaseAsync<AlloyDB<Ethereum, DynProvider>>>;

// Constants
pub const TOKEN: Address = address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
pub const TREASURY: Address = address!("0000000000000000000000000000000000000001");

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the Alloy provider and database
    let rpc_url = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27";
    let provider = ProviderBuilder::new().connect(rpc_url).await?.erased();

    let alloy_db = WrapDatabaseAsync::new(AlloyDB::new(provider, BlockId::latest())).unwrap();
    let mut cache_db = CacheDB::new(alloy_db);

    // Random empty account: From
    let account = address!("18B06aaF27d44B756FCF16Ca20C1f183EB49111f");
    // Random empty account: To
    let account_to = address!("21a4B6F62E51e59274b6Be1705c7c68781B87C77");

    // USDC has 6 decimals
    let hundred_tokens = U256::from(100_000_000_000_000_000u128);

    let balance_slot = erc_address_storage(account);
    println!("Balance slot: {balance_slot}");
    cache_db
        .insert_account_storage(TOKEN, balance_slot, hundred_tokens * StorageValue::from(2))
        .unwrap();
    cache_db.insert_account_info(
        account,
        AccountInfo {
            nonce: 0,
            balance: hundred_tokens * U256::from(2),
            code_hash: KECCAK_EMPTY,
            code: None,
        },
    );

    let balance_before = balance_of(account, &mut cache_db).unwrap();
    println!("Balance before: {balance_before}");

    // Transfer 100 tokens from account to account_to
    // Magic happens here with custom handlers
    transfer(account, account_to, hundred_tokens, &mut cache_db)?;

    let balance_after = balance_of(account, &mut cache_db)?;
    println!("Balance after: {balance_after}");

    Ok(())
}

/// Helpers
pub fn token_operation<CTX, ERROR>(
    context: &mut CTX,
    sender: Address,
    recipient: Address,
    amount: U256,
) -> Result<(), ERROR>
where
    CTX: ContextTr,
    ERROR: From<InvalidTransaction> + From<InvalidHeader> + From<<CTX::Db as Database>::Error>,
{
    let sender_balance_slot = erc_address_storage(sender);
    let sender_balance = context.journal().sload(TOKEN, sender_balance_slot)?.data;

    if sender_balance < amount {
        return Err(ERROR::from(
            InvalidTransaction::MaxFeePerBlobGasNotSupported,
        ));
    }
    // Subtract the amount from the sender's balance
    let sender_new_balance = sender_balance.saturating_sub(amount);
    context
        .journal()
        .sstore(TOKEN, sender_balance_slot, sender_new_balance)?;

    // Add the amount to the recipient's balance
    let recipient_balance_slot = erc_address_storage(recipient);
    let recipient_balance = context.journal().sload(TOKEN, recipient_balance_slot)?.data;

    let recipient_new_balance = recipient_balance.saturating_add(amount);
    context
        .journal()
        .sstore(TOKEN, recipient_balance_slot, recipient_new_balance)?;

    Ok(())
}

fn balance_of(address: Address, alloy_db: &mut AlloyCacheDB) -> Result<StorageValue> {
    let slot = erc_address_storage(address);
    alloy_db.storage(TOKEN, slot).map_err(From::from)
}

fn transfer(from: Address, to: Address, amount: U256, cache_db: &mut AlloyCacheDB) -> Result<()> {
    let mut ctx = Context::mainnet()
        .with_db(cache_db)
        .modify_cfg_chained(|cfg| {
            cfg.spec = SpecId::CANCUN;
        })
        .modify_tx_chained(|tx| {
            tx.caller = from;
            tx.kind = TxKind::Call(to);
            tx.value = amount;
            tx.gas_price = 2;
        })
        .modify_block_chained(|b| {
            b.basefee = 1;
        })
        .build_mainnet();

    transact_erc20evm_commit(&mut ctx).unwrap();

    Ok(())
}

pub fn erc_address_storage(address: Address) -> U256 {
    keccak256((address, U256::from(4)).abi_encode()).into()
}
