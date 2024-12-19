use alloy_provider::{network::Ethereum, ProviderBuilder, RootProvider};
use alloy_sol_types::{sol, SolCall, SolValue};
use alloy_transport_http::Http;
use anyhow::{anyhow, Result};
use database::{AlloyDB, BlockId, CacheDB};
use reqwest::{Client, Url};
use revm::{
    context_interface::{
        result::{ExecutionResult, InvalidHeader, InvalidTransaction, Output},
        JournalStateGetter, JournalStateGetterDBError, JournaledState,
    },
    database_interface::WrapDatabaseAsync,
    handler::EthExecution,
    precompile::PrecompileErrors,
    primitives::{address, keccak256, Address, Bytes, TxKind, U256},
    state::{AccountInfo, EvmStorageSlot},
    Context, EvmCommit, MainEvm,
};

mod handlers;
use handlers::{CustomEvm, CustomHandler, Erc20PostExecution, Erc20PreExecution, Erc20Validation};

type AlloyCacheDB =
    CacheDB<WrapDatabaseAsync<AlloyDB<Http<Client>, Ethereum, RootProvider<Http<Client>>>>>;

// Constants
pub const TOKEN: Address = address!("1234567890123456789012345678901234567890");
pub const TREASURY: Address = address!("0000000000000000000000000000000000000001");

#[tokio::main]
async fn main() -> Result<()> {
    // Set up the HTTP transport which is consumed by the RPC client.
    let rpc_url: Url = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27".parse()?;

    let client = ProviderBuilder::new().on_http(rpc_url);

    let alloy = WrapDatabaseAsync::new(AlloyDB::new(client, BlockId::latest())).unwrap();
    let mut cache_db = CacheDB::new(alloy);

    // Random empty account: From
    let account = address!("18B06aaF27d44B756FCF16Ca20C1f183EB49111f");
    // Random empty account: To
    let account_to = address!("0x21a4B6F62E51e59274b6Be1705c7c68781B87C77");

    let usdc = address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");

    // USDC has 6 decimals
    let hundred_tokens = U256::from(100_000_000_000_000_000u128);

    let balance_slot = keccak256((account, U256::from(3)).abi_encode()).into();

    cache_db.insert_account_storage(usdc, balance_slot, hundred_tokens);
    cache_db.insert_account_info(
        account,
        AccountInfo {
            nonce: 0,
            balance: hundred_tokens,
            code_hash: keccak256(Bytes::new()),
            code: None,
        },
    );

    let balance_before = balance_of(usdc, account, &mut cache_db).unwrap();

    // Transfer 100 tokens from account to account_to
    // Magic happens here with custom handlers
    transfer(account, account_to, hundred_tokens, usdc, &mut cache_db)?;

    let balance_after = balance_of(usdc, account, &mut cache_db)?;

    println!("Balance before: {balance_before}");
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
    CTX: JournalStateGetter,
    ERROR: From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalStateGetterDBError<CTX>>
        + From<PrecompileErrors>,
{
    let token_account = context.journal().load_account(TOKEN)?.data;

    let sender_balance_slot: U256 = keccak256((sender, U256::from(3)).abi_encode()).into();
    let sender_balance = token_account
        .storage
        .get(&sender_balance_slot)
        .expect("Balance slot not found")
        .present_value();

    if sender_balance < amount {
        return Err(ERROR::from(
            InvalidTransaction::MaxFeePerBlobGasNotSupported,
        ));
    }
    // Subtract the amount from the sender's balance
    let sender_new_balance = sender_balance.saturating_sub(amount);
    token_account.storage.insert(
        sender_balance_slot,
        EvmStorageSlot::new_changed(sender_balance, sender_new_balance),
    );

    // Add the amount to the recipient's balance
    let recipient_balance_slot: U256 = keccak256((recipient, U256::from(3)).abi_encode()).into();
    let recipient_balance = token_account
        .storage
        .get(&recipient_balance_slot)
        .expect("To balance slot not found")
        .present_value();
    let recipient_new_balance = recipient_balance.saturating_add(amount);
    token_account.storage.insert(
        recipient_balance_slot,
        EvmStorageSlot::new_changed(recipient_balance, recipient_new_balance),
    );

    Ok(())
}

fn balance_of(token: Address, address: Address, alloy_db: &mut AlloyCacheDB) -> Result<U256> {
    sol! {
        function balanceOf(address account) public returns (uint256);
    }

    let encoded = balanceOfCall { account: address }.abi_encode();

    let mut evm = MainEvm::new(
        Context::builder()
            .with_db(alloy_db)
            .modify_tx_chained(|tx| {
                // 0x1 because calling USDC proxy from zero address fails
                tx.caller = address!("0000000000000000000000000000000000000001");
                tx.transact_to = TxKind::Call(token);
                tx.data = encoded.into();
                tx.value = U256::from(0);
            }),
        CustomHandler::default(),
    );

    let ref_tx = evm.exec_commit().unwrap();
    let value = match ref_tx {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => value,
        result => return Err(anyhow!("'balanceOf' execution failed: {result:?}")),
    };

    let balance = <U256>::abi_decode(&value, false)?;

    Ok(balance)
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

    let mut evm = CustomEvm::new(
        Context::builder()
            .with_db(cache_db)
            .modify_tx_chained(|tx| {
                tx.caller = from;
                tx.transact_to = TxKind::Call(token);
                tx.data = encoded.into();
                tx.value = U256::from(0);
            }),
        CustomHandler::new(
            Erc20Validation::new(),
            Erc20PreExecution::new(),
            EthExecution::new(),
            Erc20PostExecution::new(),
        ),
    );
    let ref_tx = evm.exec_commit().unwrap();
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
