use std::sync::Arc;

use ethers_core::types::{Block, BlockId, TxHash, H160 as eH160, H256, U64 as eU64};
use ethers_providers::Middleware;
use tokio::runtime::{Handle, Runtime};

use crate::primitives::{AccountInfo, Address, Bytecode, B256, U256};
use crate::{Database, DatabaseRef};

use super::utils::HandleOrRuntime;

#[derive(Debug)]
pub struct EthersDB<M: Middleware> {
    client: Arc<M>,
    block_number: Option<BlockId>,
    rt: HandleOrRuntime,
}

impl<M: Middleware> EthersDB<M> {
    /// Create ethers db connector inputs are url and block on what we are basing our database (None for latest).
    ///
    /// Returns `None` if no tokio runtime is available or if the current runtime is a current-thread runtime.
    pub fn new(client: Arc<M>, block_number: Option<BlockId>) -> Option<Self> {
        let rt = match Handle::try_current() {
            Ok(handle) => match handle.runtime_flavor() {
                tokio::runtime::RuntimeFlavor::CurrentThread => return None,
                _ => HandleOrRuntime::Handle(handle),
            },
            Err(_) => return None,
        };

        if block_number.is_some() {
            Some(Self {
                client,
                block_number,
                rt,
            })
        } else {
            let mut instance = Self {
                client,
                block_number: None,
                rt,
            };
            instance.block_number = Some(BlockId::from(
                instance.block_on(instance.client.get_block_number()).ok()?,
            ));
            Some(instance)
        }
    }

    /// Create a new EthersDB instance, with a provider and a block (None for latest) and a runtime.
    ///
    /// Refer to [tokio::runtime::Builder] how to create a runtime if you are in synchronous world.
    /// If you are already using something like [tokio::main], call EthersDB::new instead.
    pub fn with_runtime(
        client: Arc<M>,
        block_number: Option<BlockId>,
        runtime: Runtime,
    ) -> Option<Self> {
        let rt = HandleOrRuntime::Runtime(runtime);
        let mut instance = Self {
            client,
            block_number,
            rt,
        };

        instance.block_number = Some(BlockId::from(
            instance.block_on(instance.client.get_block_number()).ok()?,
        ));
        Some(instance)
    }

    /// Create a new EthersDB instance, with a provider and a block (None for latest) and a handle.
    ///
    /// This generally allows you to pass any valid runtime handle, refer to [tokio::runtime::Handle] on how
    /// to obtain a handle. If you are already in asynchronous world, like [tokio::main], use EthersDB::new instead.
    pub fn with_handle(
        client: Arc<M>,
        block_number: Option<BlockId>,
        handle: Handle,
    ) -> Option<Self> {
        let rt = HandleOrRuntime::Handle(handle);
        let mut instance = Self {
            client,
            block_number,
            rt,
        };

        instance.block_number = Some(BlockId::from(
            instance.block_on(instance.client.get_block_number()).ok()?,
        ));
        Some(instance)
    }

    /// Internal utility function to call tokio feature and wait for output
    #[inline]
    fn block_on<F>(&self, f: F) -> F::Output
    where
        F: core::future::Future + Send,
        F::Output: Send,
    {
        self.rt.block_on(f)
    }

    /// set block number on which upcoming queries will be based
    #[inline]
    pub fn set_block_number(&mut self, block_number: BlockId) {
        self.block_number = Some(block_number);
    }
}

impl<M: Middleware> DatabaseRef for EthersDB<M> {
    type Error = M::Error;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let add = eH160::from(address.0 .0);

        let f = async {
            let nonce = self.client.get_transaction_count(add, self.block_number);
            let balance = self.client.get_balance(add, self.block_number);
            let code = self.client.get_code(add, self.block_number);
            tokio::join!(nonce, balance, code)
        };
        let (nonce, balance, code) = self.block_on(f);

        let balance = U256::from_limbs(balance?.0);
        let nonce = nonce?.as_u64();
        let bytecode = Bytecode::new_raw(code?.0.into());
        let code_hash = bytecode.hash_slow();
        Ok(Some(AccountInfo::new(balance, nonce, code_hash, bytecode)))
    }

    fn code_by_hash_ref(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        panic!("Should not be called. Code is already loaded");
        // not needed because we already load code with basic info
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        let add = eH160::from(address.0 .0);
        let index = H256::from(index.to_be_bytes());
        let slot_value: H256 =
            self.block_on(self.client.get_storage_at(add, index, self.block_number))?;
        Ok(U256::from_be_bytes(slot_value.to_fixed_bytes()))
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        let number = eU64::from(number);
        let block: Option<Block<TxHash>> =
            self.block_on(self.client.get_block(BlockId::from(number)))?;
        // If number is given, the block is supposed to be finalized so unwrap is safe too.
        Ok(B256::new(block.unwrap().hash.unwrap().0))
    }
}

impl<M: Middleware> Database for EthersDB<M> {
    type Error = M::Error;

    #[inline]
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        <Self as DatabaseRef>::basic_ref(self, address)
    }

    #[inline]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        <Self as DatabaseRef>::code_by_hash_ref(self, code_hash)
    }

    #[inline]
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        <Self as DatabaseRef>::storage_ref(self, address, index)
    }

    #[inline]
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        <Self as DatabaseRef>::block_hash_ref(self, number)
    }
}

// Run tests with `cargo test -- --nocapture` to see print statements
#[cfg(test)]
mod tests {
    use super::*;
    use ethers_providers::{Http, Provider};

    #[test]
    #[ignore = "flaky RPC"]
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

        let acc_info = ethersdb.basic_ref(address).unwrap().unwrap();

        // check if not empty
        assert!(acc_info.exists());
    }
}
