//! Alloy provider database implementation.

pub use alloy_eips::BlockId;
use alloy_provider::{
    network::{primitives::HeaderResponse, BlockResponse},
    Network, Provider,
};
use alloy_transport::TransportError;
use core::error::Error;
use database_interface::{async_db::DatabaseAsyncRef, DBErrorMarker};
use primitives::{Address, StorageKey, StorageValue, B256};
use state::{AccountInfo, Bytecode};
use std::fmt::Display;

/// Error type for transport-related database operations.
#[derive(Debug)]
pub struct DBTransportError(pub TransportError);

impl DBErrorMarker for DBTransportError {}

impl Display for DBTransportError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Transport error: {}", self.0)
    }
}

impl Error for DBTransportError {}

impl From<TransportError> for DBTransportError {
    fn from(e: TransportError) -> Self {
        Self(e)
    }
}

/// An alloy-powered REVM [Database][database_interface::Database].
///
/// When accessing the database, it'll use the given provider to fetch the corresponding account's data.
#[derive(Debug)]
pub struct AlloyDB<N: Network, P: Provider<N>> {
    /// The provider to fetch the data from.
    provider: P,
    /// The block number on which the queries will be based on.
    block_number: BlockId,
    _marker: core::marker::PhantomData<fn() -> N>,
}

impl<N: Network, P: Provider<N>> AlloyDB<N, P> {
    /// Creates a new AlloyDB instance, with a [Provider] and a block.
    pub fn new(provider: P, block_number: BlockId) -> Self {
        Self {
            provider,
            block_number,
            _marker: core::marker::PhantomData,
        }
    }

    /// Sets the block number on which the queries will be based on.
    pub fn set_block_number(&mut self, block_number: BlockId) {
        self.block_number = block_number;
    }
}

impl<N: Network, P: Provider<N>> DatabaseAsyncRef for AlloyDB<N, P> {
    type Error = DBTransportError;

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
            .get_block_by_number(number.into())
            .await?;
        // SAFETY: If the number is given, the block is supposed to be finalized, so unwrapping is safe.
        Ok(B256::new(*block.unwrap().header().hash()))
    }

    async fn code_by_hash_async_ref(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        panic!("This should not be called, as the code is already loaded");
        // This is not needed, as the code is already loaded with basic_ref
    }

    async fn storage_async_ref(
        &self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        Ok(self
            .provider
            .get_storage_at(address, index)
            .block_id(self.block_number)
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_provider::ProviderBuilder;
    use database_interface::{DatabaseRef, WrapDatabaseAsync};
    use primitives::KECCAK_EMPTY;

    async fn get_real_account_info() -> AccountInfo {
        let client = ProviderBuilder::new()
            .connect("https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .await
            .unwrap()
            .erased();
        let alloydb = AlloyDB::new(client, BlockId::from(16148323));
        let wrapped_alloydb = WrapDatabaseAsync::new(alloydb).unwrap();

        // ETH/USDT pair on Uniswap V2
        let address: Address = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852"
            .parse()
            .unwrap();

        wrapped_alloydb.basic_ref(address).unwrap().unwrap()
    }

    #[tokio::test]
    #[ignore = "flaky RPC"]
    async fn test_account_info_all_scenarios() {
        // Get account once
        let acc_info = get_real_account_info().await;
        assert!(acc_info.exists());

        test_full_roundtrip(&acc_info);

        test_rpc_format(&acc_info);

        test_camel_case_format(&acc_info);

        test_unknown_fields(&acc_info);

        test_partial_data(&acc_info);
    }

    fn test_full_roundtrip(original: &AccountInfo) {
        let serialized = serde_json::to_string(original).unwrap();
        let deserialized: AccountInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original.balance, deserialized.balance);
        assert_eq!(original.nonce, deserialized.nonce);
        assert_eq!(original.code_hash, deserialized.code_hash);
        assert_eq!(original.code.is_some(), deserialized.code.is_some());
    }

    fn test_rpc_format(original: &AccountInfo) {
        let json_rpc_format = serde_json::json!({
            "balance": original.balance,
            "nonce": original.nonce,
            "code": original.code
        });

        let account: AccountInfo = serde_json::from_value(json_rpc_format).unwrap();

        assert_eq!(account.balance, original.balance);
        assert_eq!(account.nonce, original.nonce);
        assert_eq!(account.code_hash, original.code_hash);
        assert!(account.code.is_some());
    }

    fn test_camel_case_format(original: &AccountInfo) {
        let json_rpc_format = serde_json::json!({
            "balance": original.balance,
            "nonce": original.nonce,
            "codeHash": original.code_hash,  // camelCase
            "code": original.code
        });

        let account: AccountInfo = serde_json::from_value(json_rpc_format).unwrap();

        assert_eq!(account.balance, original.balance);
        assert_eq!(account.nonce, original.nonce);
        assert_eq!(account.code_hash, original.code_hash);
        assert!(account.code.is_some());
    }

    fn test_unknown_fields(original: &AccountInfo) {
        let json_rpc_format = serde_json::json!({
            "balance": original.balance,
            "nonce": original.nonce,
            "code": original.code,
            "storage_root": "0x1234567890abcdef", // unknown fields
            "unknown_field": "should_be_ignored",
            "extra_data": [1, 2, 3, 4]
        });

        let account: AccountInfo = serde_json::from_value(json_rpc_format).unwrap();

        assert_eq!(account.balance, original.balance);
        assert_eq!(account.nonce, original.nonce);
        assert_eq!(account.code_hash, original.code_hash);
        assert!(account.code.is_some());
    }

    fn test_partial_data(original: &AccountInfo) {
        let json_rpc_format = serde_json::json!({
            "balance": original.balance,
            "nonce": original.nonce,
            "code_hash": original.code_hash
        });

        let account: AccountInfo = serde_json::from_value(json_rpc_format).unwrap();

        assert_eq!(account.balance, original.balance);
        assert_eq!(account.nonce, original.nonce);
        assert_eq!(account.code_hash, original.code_hash);
        assert!(account.code.is_none()); // Code should be None when not provided
    }

    #[test]
    fn test_edge_cases() {
        // Test missing required fields
        let json_missing_balance = r#"{"nonce": 1}"#;
        let json_missing_nonce = r#"{"balance": "0x1000"}"#;

        assert!(serde_json::from_str::<AccountInfo>(json_missing_balance).is_err());
        assert!(serde_json::from_str::<AccountInfo>(json_missing_nonce).is_err());
        // Test empty account
        let empty_account = serde_json::json!({
            "balance": "0x0",
            "nonce": 0
        });

        let account: AccountInfo = serde_json::from_value(empty_account).unwrap();
        assert!(account.is_empty());
        assert_eq!(account.code_hash, KECCAK_EMPTY);
        assert!(account.code.is_none());
    }

    #[test]
    fn test_account_info_deserialize_edge_cases() {
        let json_missing_balance = r#"{"nonce": 1}"#;
        let json_missing_nonce = r#"{"balance": "0x1000"}"#;

        assert!(serde_json::from_str::<AccountInfo>(json_missing_balance).is_err());
        assert!(serde_json::from_str::<AccountInfo>(json_missing_nonce).is_err());

        let empty_account = serde_json::json!({
            "balance": "0x0",
            "nonce": 0
        });

        let account: AccountInfo = serde_json::from_value(empty_account).unwrap();
        assert!(account.is_empty());
        assert_eq!(account.code_hash, KECCAK_EMPTY);
        assert!(account.code.is_none());
    }
}
