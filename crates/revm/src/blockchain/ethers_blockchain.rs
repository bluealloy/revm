use std::sync::Arc;

use crate::{B256, KECCAK_EMPTY, U256};

use ethers_core::types::{BlockId, U64 as eU64};
use ethers_providers::Middleware;
use tokio::runtime::{Handle, Runtime};

use super::Blockchain;

pub struct EthersBlockchain<M>
where
    M: Middleware,
{
    client: Arc<M>,
    runtime: Option<Runtime>,
    block_number: Option<BlockId>,
}

impl<M> EthersBlockchain<M>
where
    M: Middleware,
{
    /// create ethers db connector inputs are url and block on what we are basing our database (None for latest)
    pub fn new(client: Arc<M>, block_number: Option<u64>) -> Option<Self> {
        let runtime = Handle::try_current()
            .is_err()
            .then(|| Runtime::new().unwrap());

        let client = client;

        let mut out = Self {
            client,
            runtime,
            block_number: None,
        };
        let bnum = if let Some(block_number) = block_number {
            block_number.into()
        } else {
            out.block_on(out.client.get_block_number()).ok()?
        };

        out.block_number = Some(BlockId::from(bnum));
        Some(out)
    }

    /// internal utility function to call tokio feature and wait for output
    fn block_on<F: core::future::Future>(&self, f: F) -> F::Output {
        match &self.runtime {
            Some(runtime) => runtime.block_on(f),
            None => futures::executor::block_on(f),
        }
    }
}

impl<M> Blockchain for EthersBlockchain<M>
where
    M: Middleware,
{
    type Error = ();

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        // saturate usize
        if number > U256::from(u64::MAX) {
            return Ok(KECCAK_EMPTY);
        }
        let number = eU64::from(u64::try_from(number).unwrap());
        let f = async {
            self.client
                .get_block(BlockId::from(number))
                .await
                .ok()
                .flatten()
        };
        Ok(B256(self.block_on(f).unwrap().hash.unwrap().0))
    }
}

/// Run tests with `cargo test -- --nocapture` to see print statements
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use ethers_providers::{Http, Provider};

    #[test]
    fn can_get_block_hash() {
        let client = Provider::<Http>::try_from(
            "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        )
        .unwrap();
        let client = Arc::new(client);

        let mut ethers_blockchain = EthersBlockchain::new(
            Arc::clone(&client), // public infura mainnet
            None,
        )
        .unwrap();

        // block number to test
        let block_num = U256::from(16148323);
        let block_hash = ethers_blockchain.block_hash(block_num).unwrap();

        // https://etherscan.io/block/16148323
        let actual =
            B256::from_str("0xc133a5a4ceef2a6b5cd6fc682e49ca0f8fce3f18da85098c6a15f8e0f6f4c2cf")
                .unwrap();

        assert_eq!(block_hash, actual);
    }
}
