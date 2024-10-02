pub use alloy_eips::BlockId;
use alloy_provider::{
    network::{BlockResponse, HeaderResponse},
    Network, Provider,
};
use alloy_transport::{Transport, TransportError};
use database_interface::async_db::DatabaseAsyncRef;
use primitives::{Address, B256, U256};
use state::{AccountInfo, Bytecode};

/// An alloy-powered REVM [database_interface::Database].
///
/// When accessing the database, it'll use the given provider to fetch the corresponding account's data.
#[derive(Debug)]
pub struct AlloyDB<T: Transport + Clone, N: Network, P: Provider<T, N>> {
    /// The provider to fetch the data from.
    provider: P,
    /// The block number on which the queries will be based on.
    block_number: BlockId,
    _marker: std::marker::PhantomData<fn() -> (T, N)>,
}

impl<T: Transport + Clone, N: Network, P: Provider<T, N>> AlloyDB<T, N, P> {
    /// Create a new AlloyDB instance, with a [Provider] and a block.
    pub fn new(provider: P, block_number: BlockId) -> Self {
        Self {
            provider,
            block_number,
            _marker: std::marker::PhantomData,
        }
    }

    /// Set the block number on which the queries will be based on.
    pub fn set_block_number(&mut self, block_number: BlockId) {
        self.block_number = block_number;
    }
}

impl<T: Transport + Clone, N: Network, P: Provider<T, N>> DatabaseAsyncRef for AlloyDB<T, N, P> {
    type Error = TransportError;

    async fn basic_async_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let nonce = self
            .provider
            .get_transaction_count(address)
            .block_id(self.block_number);
        let balance = self
            .provider
            .get_balance(address)
            .block_id(self.block_number);
        let code = self
            .provider
            .get_code_at(address)
            .block_id(self.block_number);

        let (nonce, balance, code) = tokio::join!(nonce, balance, code,);

        let balance = balance?;
        let code = Bytecode::new_raw(code?.0.into());
        let code_hash = code.hash_slow();
        let nonce = nonce?;

        Ok(Some(AccountInfo::new(balance, nonce, code_hash, code)))
    }

    async fn block_hash_async_ref(&self, number: u64) -> Result<B256, Self::Error> {
        let block = self
            .provider
            // SAFETY: We know number <= u64::MAX, so we can safely convert it to u64
            .get_block_by_number(number.into(), false)
            .await?;
        // SAFETY: If the number is given, the block is supposed to be finalized, so unwrapping is safe.
        Ok(B256::new(*block.unwrap().header().hash()))
    }

    async fn code_by_hash_async_ref(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        panic!("This should not be called, as the code is already loaded");
        // This is not needed, as the code is already loaded with basic_ref
    }

    async fn storage_async_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.provider
            .get_storage_at(address, index)
            .block_id(self.block_number)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_provider::ProviderBuilder;
    use database_interface::{DatabaseRef, WrapDatabaseAsync};

    #[test]
    #[ignore = "flaky RPC"]
    fn can_get_basic() {
        let client = ProviderBuilder::new().on_http(
            "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
                .parse()
                .unwrap(),
        );
        let alloydb = AlloyDB::new(client, BlockId::from(16148323));
        let wrapped_alloydb = WrapDatabaseAsync::new(alloydb).unwrap();

        // ETH/USDT pair on Uniswap V2
        let address: Address = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852"
            .parse()
            .unwrap();

        let acc_info = wrapped_alloydb.basic_ref(address).unwrap().unwrap();
        assert!(acc_info.exists());
    }
}
