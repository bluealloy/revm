//! Blockchain test types for Ethereum state tests.
//!
//! This module contains structures for deserializing blockchain test JSON files
//! from the Ethereum test suite.

use revm::{
    context::{BlockEnv, TxEnv},
    primitives::{Address, Bytes, TxKind, B256, U256},
};
use serde::{de::IntoDeserializer, Deserialize, Deserializer};
use std::collections::HashMap;

use crate::AccountInfo;

/// Deserialize a hex string to u8
fn deserialize_hex_u8<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");
    u8::from_str_radix(s, 16).map_err(serde::de::Error::custom)
}

/// Deserialize an empty string as None for Option<Address>
fn deserialize_option_address<'de, D>(deserializer: D) -> Result<Option<Address>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Address::deserialize(s.into_deserializer()).map(Some)
    }
}

/// Deserialize B256 with proper padding for shorter values
fn deserialize_b256_pad<'de, D>(deserializer: D) -> Result<B256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");
    
    // Pad with zeros if the string is shorter than 64 hex chars
    let padded = if s.len() < 64 {
        format!("{:0>64}", s)
    } else {
        s.to_string()
    };
    
    B256::deserialize(format!("0x{}", padded).into_deserializer())
}

/// Deserialize Vec<B256> with proper padding for shorter values
fn deserialize_vec_b256_pad<'de, D>(deserializer: D) -> Result<Vec<B256>, D::Error>
where
    D: Deserializer<'de>,
{
    let strings: Vec<String> = Vec::deserialize(deserializer)?;
    strings
        .into_iter()
        .map(|s| {
            let s = s.trim_start_matches("0x");
            let padded = if s.len() < 64 {
                format!("{:0>64}", s)
            } else {
                s.to_string()
            };
            B256::deserialize(format!("0x{}", padded).into_deserializer())
        })
        .collect::<Result<Vec<_>, _>>()
}

/// Deserialize Option<Vec<B256>> with proper padding for shorter values
fn deserialize_option_vec_b256_pad<'de, D>(deserializer: D) -> Result<Option<Vec<B256>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<Vec<String>> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(strings) => {
            let vec = strings
                .into_iter()
                .map(|s| {
                    let s = s.trim_start_matches("0x");
                    let padded = if s.len() < 64 {
                        format!("{:0>64}", s)
                    } else {
                        s.to_string()
                    };
                    B256::deserialize(format!("0x{}", padded).into_deserializer())
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(vec))
        }
    }
}

mod test;

/// Blockchain test suite containing multiple test cases
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct BlockchainTest(pub HashMap<String, BlockchainTestCase>);

/// Individual blockchain test case
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainTestCase {
    /// Network name/fork identifier
    pub network: String,
    /// Genesis block header
    pub genesis_block_header: BlockHeader,
    /// Pre-state accounts
    pub pre: HashMap<Address, AccountInfo>,
    /// Post-state accounts
    pub post_state: HashMap<Address, AccountInfo>,
    /// Last block hash
    pub lastblockhash: B256,
    /// Network configuration
    pub config: Config,
    /// Genesis block RLP encoding
    #[serde(rename = "genesisRLP")]
    pub genesis_rlp: Bytes,
    /// List of blocks in the test
    pub blocks: Vec<Block>,
    /// Seal engine type
    pub seal_engine: String,
    /// Test metadata
    #[serde(rename = "_info")]
    pub info: TestInfo,
}

/// Block header structure
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    /// Parent block hash
    pub parent_hash: B256,
    /// Uncle hash (ommers hash)
    pub uncle_hash: B256,
    /// Block coinbase/beneficiary address
    pub coinbase: Address,
    /// State root hash after executing this block
    pub state_root: B256,
    /// Root hash of the transaction trie
    pub transactions_trie: B256,
    /// Root hash of the receipt trie
    pub receipt_trie: B256,
    /// Bloom filter for logs
    pub bloom: Bytes,
    /// Block difficulty (pre-merge) or 0 (post-merge)
    pub difficulty: U256,
    /// Block number
    pub number: U256,
    /// Gas limit for this block
    pub gas_limit: U256,
    /// Gas used by all transactions in this block
    pub gas_used: U256,
    /// Block timestamp
    pub timestamp: U256,
    /// Extra data field
    pub extra_data: Bytes,
    /// Mix hash for PoW validation
    pub mix_hash: B256,
    /// PoW nonce
    pub nonce: U256,
    /// Base fee per gas (EIP-1559)
    pub base_fee_per_gas: Option<U256>,
    /// Withdrawals root hash (post-Shanghai)
    pub withdrawals_root: Option<B256>,
    /// Blob gas used (EIP-4844)
    pub blob_gas_used: Option<U256>,
    /// Excess blob gas (EIP-4844)
    pub excess_blob_gas: Option<U256>,
    /// Parent beacon block root (EIP-4788)
    pub parent_beacon_block_root: Option<B256>,
    /// Requests hash (for future EIPs)
    pub requests_hash: Option<B256>,
    /// Block hash
    pub hash: B256,
}

/// Block structure containing header and transactions
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    /// Block header (when provided directly)
    pub block_header: Option<BlockHeader>,
    /// List of transactions in the block
    pub transactions: Option<Vec<Transaction>>,
    /// Uncle/ommer headers
    pub uncle_headers: Option<Vec<BlockHeader>>,
    /// Withdrawals in the block (post-Shanghai)
    pub withdrawals: Option<Vec<Withdrawal>>,
    /// RLP-encoded block data
    pub rlp: String,
    /// Block number as string
    pub blocknumber: Option<U256>,
    /// Expected exception for invalid blocks
    #[serde(rename = "expectException")]
    pub expect_exception: Option<String>,
    /// Decoded RLP data (for invalid block tests)
    pub rlp_decoded: Option<BlockRlpDecoded>,
}

/// Decoded RLP block data
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockRlpDecoded {
    /// Decoded block header
    pub block_header: BlockHeader,
    /// Decoded transactions
    pub transactions: Vec<Transaction>,
    /// Decoded uncle headers
    pub uncle_headers: Vec<BlockHeader>,
    /// Decoded withdrawals
    pub withdrawals: Vec<Withdrawal>,
    /// Block number
    pub blocknumber: U256,
}

/// Transaction structure
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// Transaction type (0=Legacy, 1=EIP-2930, 2=EIP-1559, 3=EIP-4844)
    #[serde(rename = "type", deserialize_with = "deserialize_hex_u8")]
    pub tx_type: u8,
    /// Chain ID for replay protection
    pub chain_id: Option<U256>,
    /// Transaction nonce
    pub nonce: U256,
    /// Gas price (legacy transactions)
    pub gas_price: Option<U256>,
    /// Gas limit
    pub gas_limit: U256,
    /// Recipient address (None for contract creation)
    #[serde(deserialize_with = "deserialize_option_address")]
    pub to: Option<Address>,
    /// Ether value to transfer
    pub value: U256,
    /// Transaction data/input
    pub data: Bytes,
    /// ECDSA signature v value
    pub v: U256,
    /// ECDSA signature r value
    #[serde(deserialize_with = "deserialize_b256_pad")]
    pub r: B256,
    /// ECDSA signature s value
    #[serde(deserialize_with = "deserialize_b256_pad")]
    pub s: B256,
    /// Transaction sender address
    pub sender: Address,
    /// Maximum fee per gas (EIP-1559)
    // EIP-1559 fields
    pub max_fee_per_gas: Option<U256>,
    /// Maximum priority fee per gas (EIP-1559)
    pub max_priority_fee_per_gas: Option<U256>,
    /// Blob versioned hashes (EIP-4844)
    // EIP-4844 fields
    #[serde(default, deserialize_with = "deserialize_option_vec_b256_pad")]
    pub blob_versioned_hashes: Option<Vec<B256>>,
    /// Maximum fee per blob gas (EIP-4844)
    pub max_fee_per_blob_gas: Option<U256>,
}

/// Withdrawal structure
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Withdrawal {
    /// Withdrawal index
    pub index: U256,
    /// Validator index
    pub validator_index: U256,
    /// Withdrawal recipient address
    pub address: Address,
    /// Withdrawal amount in gwei
    pub amount: U256,
}

/// Network configuration
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Network name/fork
    pub network: String,
    /// Chain ID
    pub chainid: U256,
    /// Blob gas schedule configuration
    pub blob_schedule: Option<BlobSchedule>,
}

/// Blob gas schedule configuration
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobSchedule(pub HashMap<String, BlobConfig>);

/// Blob configuration for a specific fork
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobConfig {
    /// Target blob count per block
    pub target: U256,
    /// Maximum blob count per block
    pub max: U256,
    /// Base fee update fraction
    pub base_fee_update_fraction: U256,
}

/// Test metadata information
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TestInfo {
    /// Test hash
    pub hash: B256,
    /// Test comment
    pub comment: String,
    /// Tool used to fill the test
    pub filling_transition_tool: String,
    /// Test description
    pub description: String,
    /// Test source URL
    pub url: String,
    /// Fixture format version
    pub fixture_format: String,
    /// Reference specification URL
    #[serde(rename = "reference-spec")]
    pub reference_spec: Option<String>,
    /// Reference specification version
    #[serde(rename = "reference-spec-version")]
    pub reference_spec_version: Option<String>,
    /// EELS resolution information
    #[serde(rename = "eels-resolution")]
    pub eels_resolution: Option<EelsResolution>,
}

/// EELS resolution information
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EelsResolution {
    /// Git repository URL
    pub git_url: String,
    /// Git branch name
    pub branch: String,
    /// Git commit hash
    pub commit: String,
}

impl BlockHeader {
    /// Convert BlockHeader to BlockEnv
    pub fn to_block_env(&self) -> BlockEnv {
        let mut block_env = BlockEnv {
            number: self.number,
            beneficiary: self.coinbase,
            timestamp: self.timestamp,
            gas_limit: self.gas_limit.to::<u64>(),
            basefee: self.base_fee_per_gas.unwrap_or_default().to::<u64>(),
            difficulty: self.difficulty,
            prevrandao: if self.difficulty.is_zero() {
                Some(self.mix_hash)
            } else {
                None
            },
            blob_excess_gas_and_price: None,
        };

        // Set blob gas info if available and non-zero
        if let (Some(blob_gas_used), Some(excess_blob_gas)) =
            (self.blob_gas_used, self.excess_blob_gas)
        {
            let blob_gas_used_u64 = blob_gas_used.to::<u64>();
            let excess_blob_gas_u64 = excess_blob_gas.to::<u64>();

            // Only set if there's actual blob gas activity
            if blob_gas_used_u64 > 0 || excess_blob_gas_u64 > 0 {
                block_env.set_blob_excess_gas_and_price(blob_gas_used_u64, excess_blob_gas_u64);
            }
        }

        block_env
    }
}

impl Transaction {
    /// Convert Transaction to TxEnv using the builder pattern
    pub fn to_tx_env(&self) -> Result<TxEnv, String> {
        let mut builder = TxEnv::builder();

        // Set transaction type
        builder = builder.tx_type(Some(self.tx_type));

        // Set caller
        builder = builder.caller(self.sender);

        // Set gas limit
        let gas_limit = self.gas_limit.to::<u64>();
        builder = builder.gas_limit(gas_limit);

        // Set nonce
        let nonce = self.nonce.to::<u64>();
        builder = builder.nonce(nonce);

        // Set value
        builder = builder.value(self.value);

        // Set data
        builder = builder.data(self.data.clone());

        // Set transaction kind (to address)
        let kind = if let Some(to) = self.to {
            TxKind::Call(to)
        } else {
            TxKind::Create
        };
        builder = builder.kind(kind);

        // Set chain ID if present
        if let Some(chain_id) = self.chain_id {
            let chain_id = chain_id.to::<u64>();
            builder = builder.chain_id(Some(chain_id));
        }

        // Handle gas pricing based on transaction type
        match self.tx_type {
            0 | 1 => {
                // Legacy or EIP-2930 transaction
                if let Some(gas_price) = self.gas_price {
                    let gas_price = gas_price.to::<u128>();
                    builder = builder.gas_price(gas_price);
                }
            }
            2..=4 => {
                // EIP-1559, EIP-4844, or EIP-7702 transaction
                if let Some(max_fee) = self.max_fee_per_gas {
                    let max_fee = max_fee.to::<u128>();
                    builder = builder.gas_price(max_fee);
                }

                if let Some(priority_fee) = self.max_priority_fee_per_gas {
                    let priority_fee = priority_fee.to::<u128>();
                    builder = builder.gas_priority_fee(Some(priority_fee));
                }
            }
            _ => return Err(format!("Unsupported transaction type: {}", self.tx_type)),
        }

        // Handle blob gas for EIP-4844 transactions
        if self.tx_type == 3 {
            if let Some(blob_hashes) = &self.blob_versioned_hashes {
                builder = builder.blob_hashes(blob_hashes.clone());
            }

            if let Some(max_blob_fee) = self.max_fee_per_blob_gas {
                let max_blob_fee = max_blob_fee.to::<u128>();
                builder = builder.max_fee_per_blob_gas(max_blob_fee);
            }
        }

        // Note: Authorization list for EIP-7702 would need additional handling
        // but the test format doesn't include the full authorization data

        builder
            .build()
            .map_err(|e| format!("Failed to build TxEnv: {:?}", e))
    }
}

impl BlockchainTestCase {
    /// Get the genesis block environment
    pub fn genesis_block_env(&self) -> BlockEnv {
        self.genesis_block_header.to_block_env()
    }

    /// Get the chain ID as u64
    pub fn chain_id(&self) -> u64 {
        self.config.chainid.to::<u64>()
    }
}
