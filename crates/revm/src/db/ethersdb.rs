use std::sync::Arc;

use crate::{interpreter::bytecode::Bytecode, AccountInfo, Database, B160, B256, U256};

use ethers_core::types::{BlockId, H160 as eH160, H256};
use ethers_providers::Middleware;
use tokio::runtime::{Handle, Runtime};

pub struct EthersDB<M>
where
    M: Middleware,
{
    client: Arc<M>,
    runtime: Option<Runtime>,
    block_number: Option<BlockId>,
}

impl<M> EthersDB<M>
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

impl<M> Database for EthersDB<M>
where
    M: Middleware,
{
    type Error = ();

    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        let add = eH160::from(address.0);

        let f = async {
            let nonce = self.client.get_transaction_count(add, self.block_number);
            let balance = self.client.get_balance(add, self.block_number);
            let code = self.client.get_code(add, self.block_number);
            tokio::join!(nonce, balance, code)
        };
        let (nonce, balance, code) = self.block_on(f);
        // panic on not getting data?
        Ok(Some(AccountInfo::new(
            U256::from_limbs(
                balance
                    .unwrap_or_else(|e| panic!("ethers get balance error: {e:?}"))
                    .0,
            ),
            nonce
                .unwrap_or_else(|e| panic!("ethers get nonce error: {e:?}"))
                .as_u64(),
            Bytecode::new_raw(
                code.unwrap_or_else(|e| panic!("ethers get code error: {e:?}"))
                    .0,
            ),
        )))
    }

    fn code_by_hash(&mut self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        panic!("Should not be called. Code is already loaded");
        // not needed because we already load code with basic info
    }

    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        let add = eH160::from(address.0);
        let index = H256::from(index.to_be_bytes());
        let f = async {
            let storage = self
                .client
                .get_storage_at(add, index, self.block_number)
                .await
                .unwrap();
            U256::from_be_bytes(storage.to_fixed_bytes())
        };
        Ok(self.block_on(f))
    }
}

/// Run tests with `cargo test -- --nocapture` to see print statements
#[cfg(test)]
mod tests {
    use super::*;
    use ethers_core::types::U256 as eU256;
    use ethers_providers::{Http, Provider};

    #[test]
    fn can_get_basic() {
        let client = Provider::<Http>::try_from(
            "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        )
        .unwrap();
        let client = Arc::new(client);

        let mut ethersdb = EthersDB::new(
            Arc::clone(&client), // public infura mainnet
            Some(16148323),
        )
        .unwrap();

        // ETH/USDT pair on Uniswap V2
        let address = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852"
            .parse::<eH160>()
            .unwrap();
        let address = address.as_fixed_bytes().into();

        let acc_info = ethersdb.basic(address).unwrap().unwrap();

        // check if not empty
        assert!(acc_info.exists());
    }

    #[test]
    fn can_get_storage() {
        let client = Provider::<Http>::try_from(
            "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        )
        .unwrap();
        let client = Arc::new(client);

        let mut ethersdb = EthersDB::new(
            Arc::clone(&client), // public infura mainnet
            Some(16148323),
        )
        .unwrap();

        // ETH/USDT pair on Uniswap V2
        let address = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852"
            .parse::<eH160>()
            .unwrap();
        let address = address.as_fixed_bytes().into();

        // select test index
        let index = U256::from(5);
        let storage = ethersdb.storage(address, index).unwrap();

        // https://etherscan.io/address/0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852#readContract
        // storage[5] -> factory: address
        let actual = U256::from_limbs(eU256::from("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").0);

        assert_eq!(storage, actual);
    }
}
