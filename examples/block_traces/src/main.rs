//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use alloy_consensus::Transaction;
use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_provider::{
    network::primitives::{BlockTransactions, BlockTransactionsKind},
    Provider, ProviderBuilder,
};
use database::{AlloyDB, CacheDB, StateBuilder};
use indicatif::ProgressBar;
use inspector::{inspectors::TracerEip3155, InspectorContext, InspectorEthFrame, InspectorMainEvm};
use revm::{
    database_interface::WrapDatabaseAsync,
    handler::{
        EthExecution, EthHandler, EthPostExecution, EthPreExecution, EthPrecompileProvider,
        EthValidation,
    },
    primitives::{TxKind, U256},
    Context, EvmCommit,
};
use std::io::BufWriter;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use std::{fs::OpenOptions, io::stdout};

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
        .get_block_by_number(
            BlockNumberOrTag::Number(block_number),
            BlockTransactionsKind::Full,
        )
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
    let mut evm = InspectorMainEvm::new(
        InspectorContext::new(
            Context::builder()
                .with_db(&mut state)
                .modify_block_chained(|b| {
                    b.number = U256::from(block.header.number);
                    b.beneficiary = block.header.beneficiary;
                    b.timestamp = U256::from(block.header.timestamp);

                    b.difficulty = block.header.difficulty;
                    b.gas_limit = U256::from(block.header.gas_limit);
                    b.basefee = block
                        .header
                        .base_fee_per_gas
                        .map(U256::from)
                        .unwrap_or_default();
                })
                .modify_cfg_chained(|c| {
                    c.chain_id = chain_id;
                }),
            TracerEip3155::new(Box::new(stdout())),
        ),
        EthHandler::new(
            EthValidation::new(),
            EthPreExecution::new(),
            EthExecution::<_, _, InspectorEthFrame<_, _, EthPrecompileProvider<_, _>>>::new(),
            EthPostExecution::new(),
        ),
    );

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
        evm.context.inner.modify_tx(|etx| {
            etx.caller = tx.from;
            etx.gas_limit = tx.gas_limit();
            etx.gas_price = U256::from(
                tx.gas_price
                    .unwrap_or(tx.inner.max_fee_per_gas()),
            );
            etx.value = tx.value();
            etx.data = tx.input().to_owned();
            etx.gas_priority_fee = tx.max_priority_fee_per_gas().map(U256::from);
            etx.chain_id = Some(chain_id);
            etx.nonce = tx.nonce();
            if let Some(access_list) = tx.access_list() {
                etx.access_list = access_list.to_owned();
            } else {
                etx.access_list = Default::default();
            }

            etx.transact_to = match tx.to() {
                Some(to_address) => TxKind::Call(to_address),
                None => TxKind::Create,
            };
        });

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
        evm.context.inspector.set_writer(Box::new(writer));

        let res = evm.exec_commit();

        if let Err(error) = res {
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
