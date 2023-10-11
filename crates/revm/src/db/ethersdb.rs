use crate::primitives::{AccountInfo, Address, Bytecode, B256, KECCAK_EMPTY, U256};
use crate::{Database, DatabaseRef};
use ethers_core::types::{BlockId, H160 as eH160, H256, U64 as eU64};
use ethers_providers::Middleware;
use std::sync::Arc;
use tokio::runtime::{Handle, Runtime};

#[derive(Debug)]
pub struct EthersDB<M: Middleware> {
    client: Arc<M>,
    runtime: Option<Runtime>,
    block_number: Option<BlockId>,
}

impl<M: Middleware> EthersDB<M> {
    /// create ethers db connector inputs are url and block on what we are basing our database (None for latest)
    pub fn new(client: Arc<M>, block_number: Option<BlockId>) -> Option<Self> {
        let runtime = Handle::try_current()
            .is_err()
            .then(|| Runtime::new().unwrap());

        let client = client;

        let mut out = Self {
            client,
            runtime,
            block_number: None,
        };

        out.block_number = if block_number.is_some() {
            block_number
        } else {
            Some(BlockId::from(
                out.block_on(out.client.get_block_number()).ok()?,
            ))
        };

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

impl<M: Middleware> DatabaseRef for EthersDB<M> {
    type Error = ();

    fn basic(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let add = eH160::from(address.0 .0);

        let f = async {
            let nonce = self.client.get_transaction_count(add, self.block_number);
            let balance = self.client.get_balance(add, self.block_number);
            let code = self.client.get_code(add, self.block_number);
            tokio::join!(nonce, balance, code)
        };
        let (nonce, balance, code) = self.block_on(f);
        // panic on not getting data?
        let bytecode = code.unwrap_or_else(|e| panic!("ethers get code error: {e:?}"));
        let bytecode = Bytecode::new_raw(bytecode.0.into());
        let code_hash = bytecode.hash_slow();
        Ok(Some(AccountInfo::new(
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
        )))
    }

    fn code_by_hash(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        panic!("Should not be called. Code is already loaded");
        // not needed because we already load code with basic info
    }

    fn storage(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        let add = eH160::from(address.0 .0);
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

    fn block_hash(&self, number: U256) -> Result<B256, Self::Error> {
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
        Ok(B256::new(self.block_on(f).unwrap().hash.unwrap().0))
    }
}

impl<M: Middleware> Database for EthersDB<M> {
    type Error = ();

    #[inline]
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        <Self as DatabaseRef>::basic(self, address)
    }

    #[inline]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        <Self as DatabaseRef>::code_by_hash(self, code_hash)
    }

    #[inline]
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        <Self as DatabaseRef>::storage(self, address, index)
    }

    #[inline]
    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        <Self as DatabaseRef>::block_hash(self, number)
    }
}

// Run tests with `cargo test -- --nocapture` to see print statements
#[cfg(test)]
mod tests {
    use super::*;
    use ethers_core::types::U256 as eU256;
    use ethers_providers::{Http, Provider};
    use std::str::FromStr;

    #[test]
    fn can_get_basic() {
        let client = Provider::<Http>::try_from(
            "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        )
        .unwrap();
        let client = Arc::new(client);

        let ethersdb = EthersDB::new(
            Arc::clone(&client), // public infura mainnet
            Some(BlockId::from(16148323)),
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

        let ethersdb = EthersDB::new(
            Arc::clone(&client), // public infura mainnet
            Some(BlockId::from(16148323)),
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

    #[test]
    fn can_get_block_hash() {
        let client = Provider::<Http>::try_from(
            "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        )
        .unwrap();
        let client = Arc::new(client);

        let ethersdb = EthersDB::new(
            Arc::clone(&client), // public infura mainnet
            None,
        )
        .unwrap();

        // block number to test
        let block_num = U256::from(16148323);
        let block_hash = ethersdb.block_hash(block_num).unwrap();

        // https://etherscan.io/block/16148323
        let actual =
            B256::from_str("0xc133a5a4ceef2a6b5cd6fc682e49ca0f8fce3f18da85098c6a15f8e0f6f4c2cf")
                .unwrap();

        assert_eq!(block_hash, actual);
    }
}
