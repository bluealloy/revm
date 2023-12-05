use ethers_core::types::{Block, BlockId, Transaction, H160};
use ethers_providers::{Http, Middleware, Provider, ProviderError};
use indicatif::ProgressBar;
use revm::db::{CacheDB, EmptyDB};
use revm::primitives::{AccountInfo, Address, Bytecode, Bytes, Env, TxEnv, U256};
use revm::EVM;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Test {name} failed: {kind}")]
pub struct TestError {
    pub name: String,
    pub kind: TestErrorKind,
}

#[derive(Debug, Error)]
pub enum TestErrorKind {
    #[error("evm error {0:?}")]
    EvmError(String),
    #[error("End block number ({end_block:?}) is greater than latest block number ({latest})")]
    EndBlockTooHigh { end_block: Option<u64>, latest: u64 },
    #[error("Provider error: {0:?}")]
    ProviderError(#[from] ProviderError),
    #[error("Provider missing block number {0:?}")]
    ProviderMissingBlockNumber(u64),
}

impl From<ProviderError> for TestError {
    fn from(e: ProviderError) -> Self {
        Self {
            name: "traverse".to_string(),
            kind: TestErrorKind::ProviderError(e),
        }
    }
}

/// Traverse command
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// RPC URL
    #[structopt(short = "r", long, required = true)]
    pub rpc: String,
    /// Optional block number to start from
    #[structopt(short = "s", long)]
    pub start_block: Option<u64>,
    /// Optional block number to end at
    #[structopt(short = "e", long)]
    pub end_block: Option<u64>,
    /// Output results in JSON format.
    #[structopt(long)]
    pub json: bool,
}

#[derive(Debug)]
struct TraversalEnv {
    /// The starting block number
    start: u64,
    /// The ending block number
    end: u64,
}

impl TraversalEnv {
    fn range(&self) -> u64 {
        self.end - self.start
    }
}

/// Setup the traversal
async fn setup_traversal(
    client: &Arc<Provider<Http>>,
    start_block: Option<u64>,
    end_block: Option<u64>,
) -> Result<TraversalEnv, TestError> {
    let latest = client.get_block_number().await.map_err(TestError::from)?;
    let latest = latest.as_u64();
    if end_block.map(|b| b > latest).unwrap_or(false) {
        return Err(TestError {
            name: "traverse".to_string(),
            kind: TestErrorKind::EndBlockTooHigh { end_block, latest },
        });
    }
    let start = start_block.unwrap_or(0);
    let end = end_block.unwrap_or(latest);
    Ok(TraversalEnv { start, end })
}

/// Fetch a block with transactions
async fn fetch_block_with_txs(
    client: Arc<Provider<Http>>,
    block_number: u64,
) -> Result<Block<Transaction>, TestError> {
    let block = client
        .get_block_with_txs(block_number)
        .await
        .map_err(TestError::from)?;
    let block = block.ok_or_else(|| TestError {
        name: "traverse".to_string(),
        kind: TestErrorKind::ProviderMissingBlockNumber(block_number),
    })?;
    Ok(block)
}

async fn account_info_at_block(
    client: Arc<Provider<Http>>,
    address: Address,
    block_number: u64,
) -> Result<AccountInfo, TestError> {
    let add = H160::from(address.0 .0);
    let block_number = Some(BlockId::from(block_number));
    let nonce = client.get_transaction_count(add, block_number).await;
    let balance = client.get_balance(add, block_number).await;
    let code = client.get_code(add, block_number).await;
    let bytecode = code.unwrap_or_else(|e| panic!("ethers get code error: {e:?}"));
    let bytecode = Bytecode::new_raw(bytecode.0.into());
    let code_hash = bytecode.hash_slow();
    Ok(AccountInfo::new(
        U256::from_limbs(
            balance
                .unwrap_or_else(|e| panic!("ethers get balance error: {e:?}"))
                .0,
        ),
        nonce
            .unwrap_or_else(|e| panic!("ethers get nonce error: {e:?}"))
            .as_u64(),
        code_hash,
        bytecode,
    ))
}

// todo: fix this so the account info is only loaded if it's not already in the db
// todo: fix this so all accounts and storage is loaded for the txs
async fn exec_tx(
    evm: Arc<Mutex<EVM<CacheDB<EmptyDB>>>>,
    client: Arc<Provider<Http>>,
    bn: u64,
    tx: Transaction,
) -> Result<(), TestError> {
    let mut tx_env = TxEnv::default();
    tx_env.caller = Address::from_slice(tx.from.as_bytes());
    let bn = bn.checked_sub(1).unwrap_or(0);
    let sender = account_info_at_block(Arc::clone(&client), tx_env.caller, bn).await?;
    {
        let mut temp = evm.lock().unwrap();
        let db = temp.db.as_mut().unwrap();
        db.insert_account_info(tx_env.caller, sender);
    }
    let dest = tx
        .to
        .map(|a| Address::from_slice(a.as_bytes()))
        .unwrap_or_default();
    let info = account_info_at_block(Arc::clone(&client), dest, bn).await?;
    {
        let mut temp = evm.lock().unwrap();
        let db = temp.db.as_mut().unwrap();
        db.insert_account_info(dest, info);
    }
    tx_env.gas_limit = tx.gas.as_u64();
    tx_env.gas_price = U256::from(tx.gas_price.map(|g| g.as_u128()).unwrap_or_default());
    tx_env.value = U256::from(tx.value.as_u128());
    tx_env.data = Bytes::from(tx.input.to_vec());
    tx_env.nonce = Some(tx.nonce.as_u64());
    tx_env.chain_id = tx.chain_id.map(|id| id.as_u64());
    tx_env.gas_priority_fee = tx.max_priority_fee_per_gas.map(|g| U256::from(g.as_u128()));
    tx_env.access_list = tx
        .access_list
        .map(|al| {
            al.0.into_iter()
                .map(|ali| {
                    (
                        Address::from_slice(ali.address.as_bytes()),
                        ali.storage_keys
                            .into_iter()
                            .map(|slot| U256::from_be_slice(slot.as_bytes()))
                            .collect(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    {
        let mut evm = evm.lock().unwrap();
        evm.env.tx = tx_env;
        evm.transact().map_err(|e| TestError {
            name: "traverse".to_string(),
            kind: TestErrorKind::EvmError(format!("{:?}", e)),
        })?;
    }
    Ok(())
}

async fn driver(cmd: &Cmd) -> Result<(), TestError> {
    let provider = Arc::new(Provider::<Http>::try_from(cmd.rpc.as_str()).unwrap());
    let env = setup_traversal(&provider, cmd.start_block, cmd.end_block).await?;

    let console_bar = Arc::new(ProgressBar::new(env.range()));
    let elapsed = Arc::new(Mutex::new(std::time::Duration::ZERO));

    let evm = Arc::new(Mutex::new(EVM::new()));
    {
        let mut init = evm.lock().unwrap();
        init.database(CacheDB::new(EmptyDB::default()));
        init.env = Env::default();
    }

    for block_number in env.start..=env.end {
        let block = fetch_block_with_txs(Arc::clone(&provider), block_number).await?;
        {
            let mut block_env = evm.lock().unwrap();
            block_env.env.block.number = block
                .number
                .map(|n| U256::from(n.as_u64()))
                .unwrap_or_default();
            block_env.env.block.timestamp = U256::from(block.timestamp.as_u128());
            block_env.env.block.difficulty = U256::from(block.difficulty.as_u64());
            block_env.env.block.gas_limit = U256::from(block.gas_limit.as_u64());
            block_env.env.block.coinbase = block
                .author
                .map(|a| Address::from_slice(a.as_bytes()))
                .unwrap_or_default();
            block_env.env.block.basefee = block
                .base_fee_per_gas
                .map(|g| U256::from(g.as_u128()))
                .unwrap_or_default();
        }

        for tx in block.transactions {
            let evm_clone = Arc::clone(&evm);
            let cloned_provider = Arc::clone(&provider);
            let _ = exec_tx(evm_clone, cloned_provider, block_number, tx).await?;
        }
        // todo: fix async logic so this only increments after all txs in block are executed
        console_bar.inc(1);
    }

    console_bar.finish_with_message(format!(
        "Finished executing blocks {}..{} in {:?}",
        env.start, env.end, elapsed
    ));

    Ok(())
}

/// Run traverse command.
pub fn run(cmd: &Cmd) -> Result<(), TestError> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(driver(cmd))
}
