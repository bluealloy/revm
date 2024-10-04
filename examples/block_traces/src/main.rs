//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_provider::{network::primitives::BlockTransactions, Provider, ProviderBuilder};
use database::{AlloyDB, CacheDB, StateBuilder};
use indicatif::ProgressBar;
use inspector::{inspector_handle_register, inspectors::TracerEip3155};
use revm::{
    database_interface::WrapDatabaseAsync,
    primitives::{TxKind, U256},
    wiring::EthereumWiring,
    Evm,
};
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

struct FlushWriter {
    writer: Arc<Mutex<BufWriter<std::fs::File>>>,
}

impl FlushWriter {
    fn new(writer: Arc<Mutex<BufWriter<std::fs::File>>>) -> Self {
        Self { writer }
    }
}

impl Write for FlushWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.lock().unwrap().flush()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up the HTTP transport which is consumed by the RPC client.
    let rpc_url = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27".parse()?;

    // create ethers client and wrap it in Arc<M>
    let client = ProviderBuilder::new().on_http(rpc_url);

    // Params
    let chain_id: u64 = 1;
    let block_number = 10889447;

    // Fetch the transaction-rich block
    let block = match client
        .get_block_by_number(BlockNumberOrTag::Number(block_number), true)
        .await
    {
        Ok(Some(block)) => block,
        Ok(None) => anyhow::bail!("Block not found"),
        Err(error) => anyhow::bail!("Error: {:?}", error),
    };
    println!("Fetched block number: {}", block.header.number);
    let previous_block_number = block_number - 1;

    // Use the previous block state as the db with caching
    let prev_id: BlockId = previous_block_number.into();
    // SAFETY: This cannot fail since this is in the top-level tokio runtime

    let state_db = WrapDatabaseAsync::new(AlloyDB::new(client, prev_id)).unwrap();
    let cache_db: CacheDB<_> = CacheDB::new(state_db);
    let mut state = StateBuilder::new_with_database(cache_db).build();
    let mut evm = Evm::<EthereumWiring<_, _>>::builder()
        .with_db(&mut state)
        .with_external_context(TracerEip3155::new(Box::new(std::io::stdout())))
        .modify_block_env(|b| {
            b.number = U256::from(block.header.number);
            b.coinbase = block.header.miner;
            b.timestamp = U256::from(block.header.timestamp);

            b.difficulty = block.header.difficulty;
            b.gas_limit = U256::from(block.header.gas_limit);
            b.basefee = block
                .header
                .base_fee_per_gas
                .map(U256::from)
                .unwrap_or_default();
        })
        .modify_cfg_env(|c| {
            c.chain_id = chain_id;
        })
        .append_handler_register(inspector_handle_register)
        .build();

    let txs = block.transactions.len();
    println!("Found {txs} transactions.");

    let console_bar = Arc::new(ProgressBar::new(txs as u64));
    let start = Instant::now();

    // Create the traces directory if it doesn't exist
    std::fs::create_dir_all("traces").expect("Failed to create traces directory");

    // Fill in CfgEnv
    let BlockTransactions::Full(transactions) = block.transactions else {
        panic!("Wrong transaction type")
    };

    for tx in transactions {
        evm = evm
            .modify()
            .modify_tx_env(|etx| {
                etx.caller = tx.from;
                etx.gas_limit = tx.gas;
                etx.gas_price = U256::from(
                    tx.gas_price
                        .unwrap_or(tx.max_fee_per_gas.unwrap_or_default()),
                );
                etx.value = tx.value;
                etx.data = tx.input.0.into();
                etx.gas_priority_fee = tx.max_priority_fee_per_gas.map(U256::from);
                etx.chain_id = Some(chain_id);
                etx.nonce = tx.nonce;
                if let Some(access_list) = tx.access_list {
                    etx.access_list = access_list;
                } else {
                    etx.access_list = Default::default();
                }

                etx.transact_to = match tx.to {
                    Some(to_address) => TxKind::Call(to_address),
                    None => TxKind::Create,
                };
            })
            .build();

        // Construct the file writer to write the trace to
        let tx_number = tx.transaction_index.unwrap_or_default();
        let file_name = format!("traces/{}.json", tx_number);
        let write = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_name);
        let inner = Arc::new(Mutex::new(BufWriter::new(
            write.expect("Failed to open file"),
        )));
        let writer = FlushWriter::new(Arc::clone(&inner));

        // Inspect and commit the transaction to the EVM
        evm.context.external.set_writer(Box::new(writer));
        if let Err(error) = evm.transact_commit() {
            println!("Got error: {:?}", error);
        }

        // Flush the file writer
        inner.lock().unwrap().flush().expect("Failed to flush file");

        console_bar.inc(1);
    }

    console_bar.finish_with_message("Finished all transactions.");

    let elapsed = start.elapsed();
    println!(
        "Finished execution. Total CPU time: {:.6}s",
        elapsed.as_secs_f64()
    );

    Ok(())
}
