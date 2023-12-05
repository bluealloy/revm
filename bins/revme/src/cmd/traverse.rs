use ethers_core::types::{Block, Transaction};
use ethers_providers::{Http, Middleware, Provider, ProviderError};
use indicatif::ProgressBar;
use revm::db::{CacheDB, EmptyDB};
use revm::primitives::{Address, Bytes, Env, TxEnv, U256};
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
    #[error("evm error")]
    EvmError,
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
    client: &Arc<Provider<Http>>,
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

fn exec_tx(evm: &mut EVM<CacheDB<EmptyDB>>, tx: Transaction) -> Result<(), TestError> {
    let mut tx_env = TxEnv::default();
    tx_env.caller = Address::from_slice(tx.from.as_bytes());
    tx_env.gas_limit = tx.gas.as_u64();
    tx_env.gas_price = U256::from(tx.gas_price.map(|g| g.as_u64()).unwrap_or_default());
    tx_env.value = U256::from(tx.value.as_u64());
    tx_env.data = Bytes::from(tx.input.to_vec());
    tx_env.nonce = Some(tx.nonce.as_u64());
    tx_env.chain_id = tx.chain_id.map(|id| id.as_u64());
    tx_env.gas_priority_fee = tx.max_priority_fee_per_gas.map(|g| U256::from(g.as_u64()));
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
    evm.env.tx = tx_env;
    let res = evm.transact().map_err(|_| TestError {
        name: "traverse".to_string(),
        kind: TestErrorKind::EvmError,
    })?;
    println!("{:?}", res);
    Ok(())
}

async fn driver(cmd: &Cmd) -> Result<(), TestError> {
    let provider = Arc::new(Provider::<Http>::try_from(cmd.rpc.as_str()).unwrap());
    let env = setup_traversal(&provider, cmd.start_block, cmd.end_block).await?;

    let console_bar = Arc::new(ProgressBar::new(env.range()));
    let elapsed = Arc::new(Mutex::new(std::time::Duration::ZERO));

    let mut evm = EVM::new();
    evm.database(CacheDB::new(EmptyDB::default()));
    evm.env = Env::default();

    for block_number in env.start..=env.end {
        let block = fetch_block_with_txs(&provider, block_number).await?;
        println!("{:?}", block);
        evm.env.block.number = block
            .number
            .map(|n| U256::from(n.as_u64()))
            .unwrap_or_default();
        evm.env.block.timestamp = U256::from(block.timestamp.as_u128());
        evm.env.block.difficulty = U256::from(block.difficulty.as_u64());
        evm.env.block.gas_limit = U256::from(block.gas_limit.as_u64());
        evm.env.block.coinbase = block
            .author
            .map(|a| Address::from_slice(a.as_bytes()))
            .unwrap_or_default();
        evm.env.block.basefee = block
            .base_fee_per_gas
            .map(|g| U256::from(g.as_u128()))
            .unwrap_or_default();
        for tx in block.transactions {
            println!("Executing tx: {:?}", tx);
            exec_tx(&mut evm, tx)?;
        }
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
