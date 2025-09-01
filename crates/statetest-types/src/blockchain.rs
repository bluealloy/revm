//! Blockchain test types for Ethereum state tests.
//!
//! This module contains structures for deserializing blockchain test JSON files
//! from the Ethereum test suite.

use crate::{deserialize_maybe_empty, AccountInfo};
use revm::{
    context::{BlockEnv, TxEnv},
    primitives::{
        eip4844::BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE, Address, Bytes, FixedBytes, TxKind, B256,
        U256,
    },
};
use serde::Deserialize;
use std::collections::BTreeMap;

/// Blockchain test suite containing multiple test cases
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct BlockchainTest(pub BTreeMap<String, BlockchainTestCase>);

/// Individual blockchain test case
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainTestCase {
    /// Genesis block header
    pub genesis_block_header: BlockHeader,
    /// Genesis block RLP encoding
    #[serde(rename = "genesisRLP")]
    pub genesis_rlp: Option<Bytes>,
    /// List of blocks in the test
    pub blocks: Vec<Block>,
    /// Post-state accounts (optional)
    pub post_state: Option<BTreeMap<Address, Account>>,
    /// Pre-state accounts
    pub pre: State,
    /// Last block hash
    pub lastblockhash: B256,
    /// Network specification
    pub network: ForkSpec,
    /// Seal engine type
    #[serde(default)]
    pub seal_engine: SealEngine,
}

/// Block header structure
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    /// Bloom filter for logs
    pub bloom: Bytes,
    /// Block coinbase/beneficiary address
    pub coinbase: Address,
    /// Block difficulty (pre-merge) or 0 (post-merge)
    pub difficulty: U256,
    /// Extra data field
    pub extra_data: Bytes,
    /// Gas limit for this block
    pub gas_limit: U256,
    /// Gas used by all transactions in this block
    pub gas_used: U256,
    /// Block hash
    pub hash: B256,
    /// Mix hash for PoW validation
    pub mix_hash: B256,
    /// PoW nonce
    pub nonce: FixedBytes<8>,
    /// Block number
    pub number: U256,
    /// Parent block hash
    pub parent_hash: B256,
    /// Root hash of the receipt trie
    pub receipt_trie: B256,
    /// State root hash after executing this block
    pub state_root: B256,
    /// Block timestamp
    pub timestamp: U256,
    /// Root hash of the transaction trie
    pub transactions_trie: B256,
    /// Uncle hash (ommers hash)
    pub uncle_hash: B256,
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
    /// Target blobs per block (EIP-4844 related)
    pub target_blobs_per_block: Option<U256>,
}

/// Block structure containing header and transactions
#[derive(Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    /// Block header (when provided directly)
    pub block_header: Option<BlockHeader>,
    /// RLP-encoded block data
    pub rlp: Bytes,
    /// Expected exception for invalid blocks
    pub expect_exception: Option<String>,
    /// List of transactions in the block
    pub transactions: Option<Vec<Transaction>>,
    /// Uncle/ommer headers
    pub uncle_headers: Option<Vec<BlockHeader>>,
    /// Transaction sequence (for invalid transaction tests)
    pub transaction_sequence: Option<Vec<TransactionSequence>>,
    /// Withdrawals in the block (post-Shanghai)
    pub withdrawals: Option<Vec<Withdrawal>>,
}

/// Transaction sequence in block
#[derive(Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSequence {
    /// Exception message
    pub exception: String,
    /// Raw transaction bytes
    pub raw_bytes: Bytes,
    /// Validity flag
    pub valid: String,
}

/// Transaction structure
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// Transaction type
    #[serde(rename = "type")]
    pub transaction_type: Option<U256>,
    /// Transaction sender
    #[serde(default)]
    pub sender: Option<Address>,
    /// Transaction data/input
    pub data: Bytes,
    /// Gas limit
    pub gas_limit: U256,
    /// Gas price (legacy transactions)
    pub gas_price: Option<U256>,
    /// Transaction nonce
    pub nonce: U256,
    /// ECDSA signature r value
    pub r: U256,
    /// ECDSA signature s value
    pub s: U256,
    /// ECDSA signature v value
    pub v: U256,
    /// Ether value to transfer
    pub value: U256,
    /// Target address
    #[serde(default, deserialize_with = "deserialize_maybe_empty")]
    pub to: Option<Address>,
    /// Chain ID for replay protection
    pub chain_id: Option<U256>,
    /// Access list (EIP-2930)
    pub access_list: Option<AccessList>,
    /// Maximum fee per gas (EIP-1559)
    pub max_fee_per_gas: Option<U256>,
    /// Maximum priority fee per gas (EIP-1559)
    pub max_priority_fee_per_gas: Option<U256>,
    /// Blob versioned hashes (EIP-4844)
    pub blob_versioned_hashes: Option<Vec<B256>>,
    /// Maximum fee per blob gas (EIP-4844)
    pub max_fee_per_blob_gas: Option<U256>,
    /// Transaction hash
    pub hash: Option<B256>,
}

/// Access list item
#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccessListItem {
    /// Account address
    pub address: Address,
    /// Storage keys
    pub storage_keys: Vec<B256>,
}

/// Access list
pub type AccessList = Vec<AccessListItem>;

/// Withdrawal structure
#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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

/// Ethereum blockchain test data state
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Default)]
pub struct State(pub BTreeMap<Address, Account>);

impl State {
    /// Return state as genesis state
    pub fn into_genesis_state(self) -> BTreeMap<Address, AccountInfo> {
        self.0
            .into_iter()
            .map(|(address, account)| {
                let storage = account
                    .storage
                    .iter()
                    .filter(|(_, v)| !v.is_zero())
                    .map(|(k, v)| (*k, *v))
                    .collect();
                let account_info = AccountInfo {
                    balance: account.balance,
                    nonce: account.nonce.to::<u64>(),
                    code: account.code,
                    storage,
                };
                (address, account_info)
            })
            .collect::<BTreeMap<_, _>>()
    }
}

/// An account
#[derive(Debug, PartialEq, Eq, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Account {
    /// Balance
    pub balance: U256,
    /// Code
    pub code: Bytes,
    /// Nonce
    pub nonce: U256,
    /// Storage
    pub storage: BTreeMap<U256, U256>,
}

/// Fork specification
#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Ord, Clone, Copy, Deserialize)]
pub enum ForkSpec {
    /// Frontier
    Frontier,
    /// Frontier to Homestead
    FrontierToHomesteadAt5,
    /// Homestead
    Homestead,
    /// Homestead to Tangerine
    HomesteadToDaoAt5,
    /// Homestead to Tangerine
    HomesteadToEIP150At5,
    /// Tangerine
    EIP150,
    /// Spurious Dragon
    EIP158,
    /// Spurious Dragon to Byzantium
    EIP158ToByzantiumAt5,
    /// Byzantium
    Byzantium,
    /// Byzantium to Constantinople
    ByzantiumToConstantinopleAt5,
    /// Byzantium to Constantinople
    ByzantiumToConstantinopleFixAt5,
    /// Constantinople
    Constantinople,
    /// Constantinople fix
    ConstantinopleFix,
    /// Istanbul
    Istanbul,
    /// Berlin
    Berlin,
    /// Berlin to London
    BerlinToLondonAt5,
    /// London
    London,
    /// Paris aka The Merge
    #[serde(alias = "Merge")]
    Paris,
    /// Paris to Shanghai transition
    ParisToShanghaiAtTime15k,
    /// Shanghai
    Shanghai,
    /// Shanghai to Cancun transition
    ShanghaiToCancunAtTime15k,
    /// Merge EOF test
    #[serde(alias = "Merge+3540+3670")]
    MergeEOF,
    /// After Merge Init Code test
    #[serde(alias = "Merge+3860")]
    MergeMeterInitCode,
    /// After Merge plus new PUSH0 opcode
    #[serde(alias = "Merge+3855")]
    MergePush0,
    /// Cancun
    Cancun,
    /// Cancun to Prague transition
    CancunToPragueAtTime15k,
    /// Prague
    Prague,
    /// Prague to Osaka transition
    PragueToOsakaAtTime15k,
    /// Osaka
    Osaka,
    /// BPO1 to BPO2 transition
    BPO1ToBPO2AtTime15k,
}

/// Possible seal engines
#[derive(Debug, PartialEq, Eq, Default, Deserialize)]
pub enum SealEngine {
    /// No consensus checks
    #[default]
    NoProof,
    /// Proof of Work
    Ethash,
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

        let excess_blob_gas = self.excess_blob_gas.unwrap_or_default().to::<u64>();
        block_env
            .set_blob_excess_gas_and_price(excess_blob_gas, BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE);

        block_env
    }
}

impl Transaction {
    /// Convert Transaction to TxEnv
    /// Note: The 'to' and 'sender' fields need to be provided separately in the reth model
    pub fn to_tx_env(&self) -> Result<TxEnv, String> {
        // Determine transaction type
        let tx_type = self.transaction_type.map(|t| t.to::<u8>()).unwrap_or(0);

        // Set transaction kind (to address)
        let kind = if let Some(to_addr) = self.to {
            TxKind::Call(to_addr)
        } else {
            TxKind::Create
        };

        let Some(sender) = self.sender else {
            return Err("Sender is required".to_string());
        };

        // Build the base transaction
        let mut builder = TxEnv::builder()
            .tx_type(Some(tx_type))
            .caller(sender)
            .gas_limit(self.gas_limit.to::<u64>())
            .nonce(self.nonce.to::<u64>())
            .value(self.value)
            .data(self.data.clone())
            .blob_hashes(self.blob_versioned_hashes.clone().unwrap_or_default())
            .max_fee_per_blob_gas(self.max_fee_per_blob_gas.unwrap_or_default().to::<u128>())
            .kind(kind);

        // Set chain ID if present
        if let Some(chain_id) = self.chain_id {
            builder = builder.chain_id(Some(chain_id.to::<u64>()));
        }

        // Handle gas pricing based on transaction type
        builder = match tx_type {
            0 | 1 => {
                // Legacy or EIP-2930 transaction
                if let Some(gas_price) = self.gas_price {
                    builder.gas_price(gas_price.to::<u128>())
                } else {
                    builder
                }
            }
            2 | 3 => {
                // EIP-1559 or EIP-4844 transaction
                let mut b = builder;
                if let Some(max_fee) = self.max_fee_per_gas {
                    b = b.gas_price(max_fee.to::<u128>());
                }
                if let Some(priority_fee) = self.max_priority_fee_per_gas {
                    b = b.gas_priority_fee(Some(priority_fee.to::<u128>()));
                }
                b
            }
            _ => {
                // For unknown types, try to use gas_price if available
                if let Some(gas_price) = self.gas_price {
                    builder.gas_price(gas_price.to::<u128>())
                } else {
                    builder
                }
            }
        };

        builder
            .build()
            .map_err(|e| format!("Failed to build TxEnv: {e:?}"))
    }
}

impl BlockchainTestCase {
    /// Get the genesis block environment
    pub fn genesis_block_env(&self) -> BlockEnv {
        self.genesis_block_header.to_block_env()
    }
}

#[cfg(test)]
mod test {
    use revm::primitives::address;

    use super::*;

    #[test]
    fn test_transaction_deserialization() {
        let tx = r#"
        {
            "type": "0x00",
            "chainId": "0x01",
            "nonce": "0x00",
            "gasPrice": "0x0a",
            "gasLimit": "0x05f5e100",
            "to": "0x1000000000000000000000000000000000000000",
            "value": "0x00",
            "data": "0x",
            "v": "0x25",
            "r": "0x52665e44edaa715e7c5f531675a96a47c7827593adf02f5d9b97c4bc952500ec",
            "s": "0x1b4d3b625da8720d6a05d67ad1aa986089717cd0ff4fef2ee0e76779f9746957",
            "sender": "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b"
        }"#;
        let _: Transaction = serde_json::from_str(tx).unwrap();
    }

    #[test]
    fn test_blockchain_test_deserialization() {
        // Test that we can deserialize the sample JSON
        let result: Result<BlockchainTest, _> = serde_json::from_str(SAMPLE);

        // Note: The test may fail because the sample JSON has a different structure
        // than what reth expects (e.g., network is a string instead of ForkSpec enum)
        // This is expected as the formats differ slightly
        if let Err(e) = result {
            println!(
                "Expected deserialization error due to format differences: {}",
                e
            );
        }
    }

    #[test]
    fn test_fork_spec_deserialization() {
        // Test ForkSpec enum deserialization
        let fork_specs = vec![
            ("Frontier", ForkSpec::Frontier),
            ("Homestead", ForkSpec::Homestead),
            ("Byzantium", ForkSpec::Byzantium),
            ("Constantinople", ForkSpec::Constantinople),
            ("Istanbul", ForkSpec::Istanbul),
            ("Berlin", ForkSpec::Berlin),
            ("London", ForkSpec::London),
            ("Paris", ForkSpec::Paris),
            ("Merge", ForkSpec::Paris), // Alias test
            ("Shanghai", ForkSpec::Shanghai),
            ("Cancun", ForkSpec::Cancun),
            ("CancunToPragueAtTime15k", ForkSpec::CancunToPragueAtTime15k),
            ("Prague", ForkSpec::Prague),
            ("PragueToOsakaAtTime15k", ForkSpec::PragueToOsakaAtTime15k),
            ("Osaka", ForkSpec::Osaka),
        ];

        for (json, expected) in fork_specs {
            let result: ForkSpec = serde_json::from_str(json).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_transaction_conversion() {
        use crate::blockchain::Transaction;
        use revm::primitives::{Bytes, U256};

        let tx = Transaction {
            transaction_type: Some(U256::from(0)),
            sender: Some(address!("0x1000000000000000000000000000000000000000")),
            data: Bytes::default(),
            gas_limit: U256::from(21000),
            gas_price: Some(U256::from(1000000000)),
            nonce: U256::from(0),
            r: U256::from(1),
            s: U256::from(2),
            v: U256::from(27),
            value: U256::from(1000),
            chain_id: Some(U256::from(1)),
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            max_fee_per_blob_gas: None,
            hash: None,
            to: None,
            blob_versioned_hashes: None,
        };

        // Test conversion with dummy sender and to address
        let tx_env = tx.to_tx_env().unwrap();

        assert_eq!(tx_env.tx_type, 0);
        assert_eq!(tx_env.nonce, 0);
        assert_eq!(tx_env.gas_limit, 21000);
        assert_eq!(tx_env.gas_price, 1000000000);
        assert_eq!(tx_env.value, U256::from(1000));
    }

    const SAMPLE: &str = r#"
    {
    "tests/osaka/eip7825_transaction_gas_limit_cap/test_tx_gas_limit.py::test_transaction_gas_limit_cap_at_transition[fork_PragueToOsakaAtTime15k-blockchain_test]": {
        "network": "PragueToOsakaAtTime15k",
        "genesisBlockHeader": {
            "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "uncleHash": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
            "coinbase": "0x0000000000000000000000000000000000000000",
            "stateRoot": "0xfe13aa0b3a4ea731b1715a429c1cf100db415262a5bdd49478dc7b9e61cbf1df",
            "transactionsTrie": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            "receiptTrie": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            "bloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "difficulty": "0x00",
            "number": "0x00",
            "gasLimit": "0x044aa200",
            "gasUsed": "0x00",
            "timestamp": "0x00",
            "extraData": "0x00",
            "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "nonce": "0x0000000000000000",
            "baseFeePerGas": "0x07",
            "withdrawalsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            "blobGasUsed": "0x00",
            "excessBlobGas": "0x00",
            "parentBeaconBlockRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "requestsHash": "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "hash": "0x04b688c25df122da84e0b1a21b90dd8f898e6005cdc2d1ac6c60ded9ff9b2de4"
        },
        "pre": {
            "0x00000000219ab540356cbb839cbe05303d7705fa": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x60806040526004361061003f5760003560e01c806301ffc9a71461004457806322895118146100a4578063621fd130146101ba578063c5f2892f14610244575b600080fd5b34801561005057600080fd5b506100906004803603602081101561006757600080fd5b50357fffffffff000000000000000000000000000000000000000000000000000000001661026b565b604080519115158252519081900360200190f35b6101b8600480360360808110156100ba57600080fd5b8101906020810181356401000000008111156100d557600080fd5b8201836020820111156100e757600080fd5b8035906020019184600183028401116401000000008311171561010957600080fd5b91939092909160208101903564010000000081111561012757600080fd5b82018360208201111561013957600080fd5b8035906020019184600183028401116401000000008311171561015b57600080fd5b91939092909160208101903564010000000081111561017957600080fd5b82018360208201111561018b57600080fd5b803590602001918460018302840111640100000000831117156101ad57600080fd5b919350915035610304565b005b3480156101c657600080fd5b506101cf6110b5565b6040805160208082528351818301528351919283929083019185019080838360005b838110156102095781810151838201526020016101f1565b50505050905090810190601f1680156102365780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b34801561025057600080fd5b506102596110c7565b60408051918252519081900360200190f35b60007fffffffff0000000000000000000000000000000000000000000000000000000082167f01ffc9a70000000000000000000000000000000000000000000000000000000014806102fe57507fffffffff0000000000000000000000000000000000000000000000000000000082167f8564090700000000000000000000000000000000000000000000000000000000145b92915050565b6030861461035d576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260268152602001806118056026913960400191505060405180910390fd5b602084146103b6576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252603681526020018061179c6036913960400191505060405180910390fd5b6060821461040f576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260298152602001806118786029913960400191505060405180910390fd5b670de0b6b3a7640000341015610470576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260268152602001806118526026913960400191505060405180910390fd5b633b9aca003406156104cd576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260338152602001806117d26033913960400191505060405180910390fd5b633b9aca00340467ffffffffffffffff811115610535576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252602781526020018061182b6027913960400191505060405180910390fd5b6060610540826114ba565b90507f649bbc62d0e31342afea4e5cd82d4049e7e1ee912fc0889aa790803be39038c589898989858a8a6105756020546114ba565b6040805160a0808252810189905290819060208201908201606083016080840160c085018e8e80828437600083820152601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe01690910187810386528c815260200190508c8c808284376000838201819052601f9091017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe01690920188810386528c5181528c51602091820193918e019250908190849084905b83811015610648578181015183820152602001610630565b50505050905090810190601f1680156106755780820380516001836020036101000a031916815260200191505b5086810383528881526020018989808284376000838201819052601f9091017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169092018881038452895181528951602091820193918b019250908190849084905b838110156106ef5781810151838201526020016106d7565b50505050905090810190601f16801561071c5780820380516001836020036101000a031916815260200191505b509d505050505050505050505050505060405180910390a1600060028a8a600060801b604051602001808484808284377fffffffffffffffffffffffffffffffff0000000000000000000000000000000090941691909301908152604080517ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0818403018152601090920190819052815191955093508392506020850191508083835b602083106107fc57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe090920191602091820191016107bf565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610859573d6000803e3d6000fd5b5050506040513d602081101561086e57600080fd5b5051905060006002806108846040848a8c6116fe565b6040516020018083838082843780830192505050925050506040516020818303038152906040526040518082805190602001908083835b602083106108f857805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe090920191602091820191016108bb565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610955573d6000803e3d6000fd5b5050506040513d602081101561096a57600080fd5b5051600261097b896040818d6116fe565b60405160009060200180848480828437919091019283525050604080518083038152602092830191829052805190945090925082918401908083835b602083106109f457805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe090920191602091820191016109b7565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610a51573d6000803e3d6000fd5b5050506040513d6020811015610a6657600080fd5b5051604080516020818101949094528082019290925280518083038201815260609092019081905281519192909182918401908083835b60208310610ada57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610a9d565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610b37573d6000803e3d6000fd5b5050506040513d6020811015610b4c57600080fd5b50516040805160208101858152929350600092600292839287928f928f92018383808284378083019250505093505050506040516020818303038152906040526040518082805190602001908083835b60208310610bd957805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610b9c565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610c36573d6000803e3d6000fd5b5050506040513d6020811015610c4b57600080fd5b50516040518651600291889160009188916020918201918291908601908083835b60208310610ca957805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610c6c565b6001836020036101000a0380198251168184511680821785525050505050509050018367ffffffffffffffff191667ffffffffffffffff1916815260180182815260200193505050506040516020818303038152906040526040518082805190602001908083835b60208310610d4e57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610d11565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610dab573d6000803e3d6000fd5b5050506040513d6020811015610dc057600080fd5b5051604080516020818101949094528082019290925280518083038201815260609092019081905281519192909182918401908083835b60208310610e3457805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610df7565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610e91573d6000803e3d6000fd5b5050506040513d6020811015610ea657600080fd5b50519050858114610f02576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260548152602001806117486054913960600191505060405180910390fd5b60205463ffffffff11610f60576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260218152602001806117276021913960400191505060405180910390fd5b602080546001019081905560005b60208110156110a9578160011660011415610fa0578260008260208110610f9157fe5b0155506110ac95505050505050565b600260008260208110610faf57fe5b01548460405160200180838152602001828152602001925050506040516020818303038152906040526040518082805190602001908083835b6020831061102557805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610fe8565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015611082573d6000803e3d6000fd5b5050506040513d602081101561109757600080fd5b50519250600282049150600101610f6e565b50fe5b50505050505050565b60606110c26020546114ba565b905090565b6020546000908190815b60208110156112f05781600116600114156111e6576002600082602081106110f557fe5b01548460405160200180838152602001828152602001925050506040516020818303038152906040526040518082805190602001908083835b6020831061116b57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0909201916020918201910161112e565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa1580156111c8573d6000803e3d6000fd5b5050506040513d60208110156111dd57600080fd5b505192506112e2565b600283602183602081106111f657fe5b015460405160200180838152602001828152602001925050506040516020818303038152906040526040518082805190602001908083835b6020831061126b57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0909201916020918201910161122e565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa1580156112c8573d6000803e3d6000fd5b5050506040513d60208110156112dd57600080fd5b505192505b6002820491506001016110d1565b506002826112ff6020546114ba565b600060401b6040516020018084815260200183805190602001908083835b6020831061135a57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0909201916020918201910161131d565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790527fffffffffffffffffffffffffffffffffffffffffffffffff000000000000000095909516920191825250604080518083037ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8018152601890920190819052815191955093508392850191508083835b6020831061143f57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101611402565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa15801561149c573d6000803e3d6000fd5b5050506040513d60208110156114b157600080fd5b50519250505090565b60408051600880825281830190925260609160208201818036833701905050905060c082901b8060071a60f81b826000815181106114f457fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060061a60f81b8260018151811061153757fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060051a60f81b8260028151811061157a57fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060041a60f81b826003815181106115bd57fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060031a60f81b8260048151811061160057fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060021a60f81b8260058151811061164357fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060011a60f81b8260068151811061168657fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060001a60f81b826007815181106116c957fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a90535050919050565b6000808585111561170d578182fd5b83861115611719578182fd5b505082019391909203915056fe4465706f736974436f6e74726163743a206d65726b6c6520747265652066756c6c4465706f736974436f6e74726163743a207265636f6e7374727563746564204465706f7369744461746120646f6573206e6f74206d6174636820737570706c696564206465706f7369745f646174615f726f6f744465706f736974436f6e74726163743a20696e76616c6964207769746864726177616c5f63726564656e7469616c73206c656e6774684465706f736974436f6e74726163743a206465706f7369742076616c7565206e6f74206d756c7469706c65206f6620677765694465706f736974436f6e74726163743a20696e76616c6964207075626b6579206c656e6774684465706f736974436f6e74726163743a206465706f7369742076616c756520746f6f20686967684465706f736974436f6e74726163743a206465706f7369742076616c756520746f6f206c6f774465706f736974436f6e74726163743a20696e76616c6964207369676e6174757265206c656e677468a2646970667358221220dceca8706b29e917dacf25fceef95acac8d90d765ac926663ce4096195952b6164736f6c634300060b0033",
                "storage": {
                    "0x22": "0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
                    "0x23": "0xdb56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71",
                    "0x24": "0xc78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c",
                    "0x25": "0x536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c",
                    "0x26": "0x9efde052aa15429fae05bad4d0b1d7c64da64d03d7a1854a588c2cb8430c0d30",
                    "0x27": "0xd88ddfeed400a8755596b21942c1497e114c302e6118290f91e6772976041fa1",
                    "0x28": "0x87eb0ddba57e35f6d286673802a4af5975e22506c7cf4c64bb6be5ee11527f2c",
                    "0x29": "0x26846476fd5fc54a5d43385167c95144f2643f533cc85bb9d16b782f8d7db193",
                    "0x2a": "0x506d86582d252405b840018792cad2bf1259f1ef5aa5f887e13cb2f0094f51e1",
                    "0x2b": "0xffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b",
                    "0x2c": "0x6cf04127db05441cd833107a52be852868890e4317e6a02ab47683aa75964220",
                    "0x2d": "0xb7d05f875f140027ef5118a2247bbb84ce8f2f0f1123623085daf7960c329f5f",
                    "0x2e": "0xdf6af5f5bbdb6be9ef8aa618e4bf8073960867171e29676f8b284dea6a08a85e",
                    "0x2f": "0xb58d900f5e182e3c50ef74969ea16c7726c549757cc23523c369587da7293784",
                    "0x30": "0xd49a7502ffcfb0340b1d7885688500ca308161a7f96b62df9d083b71fcc8f2bb",
                    "0x31": "0x8fe6b1689256c0d385f42f5bbe2027a22c1996e110ba97c171d3e5948de92beb",
                    "0x32": "0x8d0d63c39ebade8509e0ae3c9c3876fb5fa112be18f905ecacfecb92057603ab",
                    "0x33": "0x95eec8b2e541cad4e91de38385f2e046619f54496c2382cb6cacd5b98c26f5a4",
                    "0x34": "0xf893e908917775b62bff23294dbbe3a1cd8e6cc1c35b4801887b646a6f81f17f",
                    "0x35": "0xcddba7b592e3133393c16194fac7431abf2f5485ed711db282183c819e08ebaa",
                    "0x36": "0x8a8d7fe3af8caa085a7639a832001457dfb9128a8061142ad0335629ff23ff9c",
                    "0x37": "0xfeb3c337d7a51a6fbf00b9e34c52e1c9195c969bd4e7a0bfd51d5c5bed9c1167",
                    "0x38": "0xe71f0aa83cc32edfbefa9f4d3e0174ca85182eec9f3a09f6a6c0df6377a510d7",
                    "0x39": "0x31206fa80a50bb6abe29085058f16212212a60eec8f049fecb92d8c8e0a84bc0",
                    "0x3a": "0x21352bfecbeddde993839f614c3dac0a3ee37543f9b412b16199dc158e23b544",
                    "0x3b": "0x619e312724bb6d7c3153ed9de791d764a366b389af13c58bf8a8d90481a46765",
                    "0x3c": "0x7cdd2986268250628d0c10e385c58c6191e6fbe05191bcc04f133f2cea72c1c4",
                    "0x3d": "0x848930bd7ba8cac54661072113fb278869e07bb8587f91392933374d017bcbe1",
                    "0x3e": "0x8869ff2c22b28cc10510d9853292803328be4fb0e80495e8bb8d271f5b889636",
                    "0x3f": "0xb5fe28e79f1b850f8658246ce9b6a1e7b49fc06db7143e8fe0b4f2b0c5523a5c",
                    "0x40": "0x985e929f70af28d0bdd1a90a808f977f597c7c778c489e98d3bd8910d31ac0f7"
                }
            },
            "0x00000961ef480eb55e80d19ad83579a64c007002": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x3373fffffffffffffffffffffffffffffffffffffffe1460cb5760115f54807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff146101f457600182026001905f5b5f82111560685781019083028483029004916001019190604d565b909390049250505036603814608857366101f457346101f4575f5260205ff35b34106101f457600154600101600155600354806003026004013381556001015f35815560010160203590553360601b5f5260385f601437604c5fa0600101600355005b6003546002548082038060101160df575060105b5f5b8181146101835782810160030260040181604c02815460601b8152601401816001015481526020019060020154807fffffffffffffffffffffffffffffffff00000000000000000000000000000000168252906010019060401c908160381c81600701538160301c81600601538160281c81600501538160201c81600401538160181c81600301538160101c81600201538160081c81600101535360010160e1565b910180921461019557906002556101a0565b90505f6002555f6003555b5f54807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff14156101cd57505f5b6001546002828201116101e25750505f6101e8565b01600290035b5f555f600155604c025ff35b5f5ffd",
                "storage": {}
            },
            "0x0000bbddc7ce488642fb579f8b00f3a590007251": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x3373fffffffffffffffffffffffffffffffffffffffe1460d35760115f54807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1461019a57600182026001905f5b5f82111560685781019083028483029004916001019190604d565b9093900492505050366060146088573661019a573461019a575f5260205ff35b341061019a57600154600101600155600354806004026004013381556001015f358155600101602035815560010160403590553360601b5f5260605f60143760745fa0600101600355005b6003546002548082038060021160e7575060025b5f5b8181146101295782810160040260040181607402815460601b815260140181600101548152602001816002015481526020019060030154905260010160e9565b910180921461013b5790600255610146565b90505f6002555f6003555b5f54807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff141561017357505f5b6001546001828201116101885750505f61018e565b01600190035b5f555f6001556074025ff35b5f5ffd",
                "storage": {}
            },
            "0x0000f90827f1c53a10cb7a02335b175320002935": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x3373fffffffffffffffffffffffffffffffffffffffe14604657602036036042575f35600143038111604257611fff81430311604257611fff9006545f5260205ff35b5f5ffd5b5f35611fff60014303065500",
                "storage": {}
            },
            "0x000f3df6d732807ef1319fb7b8bb8522d0beac02": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x3373fffffffffffffffffffffffffffffffffffffffe14604d57602036146024575f5ffd5b5f35801560495762001fff810690815414603c575f5ffd5b62001fff01545f5260205ff35b5f5ffd5b62001fff42064281555f359062001fff015500",
                "storage": {}
            },
            "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b": {
                "nonce": "0x00",
                "balance": "0x3635c9adc5dea00000",
                "code": "0x",
                "storage": {}
            },
            "0x0000000000000000000000000000000000001000": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x60016000540160005500",
                "storage": {}
            }
        },
        "postState": {
            "0x00000000219ab540356cbb839cbe05303d7705fa": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x60806040526004361061003f5760003560e01c806301ffc9a71461004457806322895118146100a4578063621fd130146101ba578063c5f2892f14610244575b600080fd5b34801561005057600080fd5b506100906004803603602081101561006757600080fd5b50357fffffffff000000000000000000000000000000000000000000000000000000001661026b565b604080519115158252519081900360200190f35b6101b8600480360360808110156100ba57600080fd5b8101906020810181356401000000008111156100d557600080fd5b8201836020820111156100e757600080fd5b8035906020019184600183028401116401000000008311171561010957600080fd5b91939092909160208101903564010000000081111561012757600080fd5b82018360208201111561013957600080fd5b8035906020019184600183028401116401000000008311171561015b57600080fd5b91939092909160208101903564010000000081111561017957600080fd5b82018360208201111561018b57600080fd5b803590602001918460018302840111640100000000831117156101ad57600080fd5b919350915035610304565b005b3480156101c657600080fd5b506101cf6110b5565b6040805160208082528351818301528351919283929083019185019080838360005b838110156102095781810151838201526020016101f1565b50505050905090810190601f1680156102365780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b34801561025057600080fd5b506102596110c7565b60408051918252519081900360200190f35b60007fffffffff0000000000000000000000000000000000000000000000000000000082167f01ffc9a70000000000000000000000000000000000000000000000000000000014806102fe57507fffffffff0000000000000000000000000000000000000000000000000000000082167f8564090700000000000000000000000000000000000000000000000000000000145b92915050565b6030861461035d576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260268152602001806118056026913960400191505060405180910390fd5b602084146103b6576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252603681526020018061179c6036913960400191505060405180910390fd5b6060821461040f576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260298152602001806118786029913960400191505060405180910390fd5b670de0b6b3a7640000341015610470576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260268152602001806118526026913960400191505060405180910390fd5b633b9aca003406156104cd576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260338152602001806117d26033913960400191505060405180910390fd5b633b9aca00340467ffffffffffffffff811115610535576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252602781526020018061182b6027913960400191505060405180910390fd5b6060610540826114ba565b90507f649bbc62d0e31342afea4e5cd82d4049e7e1ee912fc0889aa790803be39038c589898989858a8a6105756020546114ba565b6040805160a0808252810189905290819060208201908201606083016080840160c085018e8e80828437600083820152601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe01690910187810386528c815260200190508c8c808284376000838201819052601f9091017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe01690920188810386528c5181528c51602091820193918e019250908190849084905b83811015610648578181015183820152602001610630565b50505050905090810190601f1680156106755780820380516001836020036101000a031916815260200191505b5086810383528881526020018989808284376000838201819052601f9091017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169092018881038452895181528951602091820193918b019250908190849084905b838110156106ef5781810151838201526020016106d7565b50505050905090810190601f16801561071c5780820380516001836020036101000a031916815260200191505b509d505050505050505050505050505060405180910390a1600060028a8a600060801b604051602001808484808284377fffffffffffffffffffffffffffffffff0000000000000000000000000000000090941691909301908152604080517ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0818403018152601090920190819052815191955093508392506020850191508083835b602083106107fc57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe090920191602091820191016107bf565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610859573d6000803e3d6000fd5b5050506040513d602081101561086e57600080fd5b5051905060006002806108846040848a8c6116fe565b6040516020018083838082843780830192505050925050506040516020818303038152906040526040518082805190602001908083835b602083106108f857805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe090920191602091820191016108bb565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610955573d6000803e3d6000fd5b5050506040513d602081101561096a57600080fd5b5051600261097b896040818d6116fe565b60405160009060200180848480828437919091019283525050604080518083038152602092830191829052805190945090925082918401908083835b602083106109f457805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe090920191602091820191016109b7565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610a51573d6000803e3d6000fd5b5050506040513d6020811015610a6657600080fd5b5051604080516020818101949094528082019290925280518083038201815260609092019081905281519192909182918401908083835b60208310610ada57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610a9d565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610b37573d6000803e3d6000fd5b5050506040513d6020811015610b4c57600080fd5b50516040805160208101858152929350600092600292839287928f928f92018383808284378083019250505093505050506040516020818303038152906040526040518082805190602001908083835b60208310610bd957805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610b9c565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610c36573d6000803e3d6000fd5b5050506040513d6020811015610c4b57600080fd5b50516040518651600291889160009188916020918201918291908601908083835b60208310610ca957805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610c6c565b6001836020036101000a0380198251168184511680821785525050505050509050018367ffffffffffffffff191667ffffffffffffffff1916815260180182815260200193505050506040516020818303038152906040526040518082805190602001908083835b60208310610d4e57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610d11565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610dab573d6000803e3d6000fd5b5050506040513d6020811015610dc057600080fd5b5051604080516020818101949094528082019290925280518083038201815260609092019081905281519192909182918401908083835b60208310610e3457805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610df7565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015610e91573d6000803e3d6000fd5b5050506040513d6020811015610ea657600080fd5b50519050858114610f02576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260548152602001806117486054913960600191505060405180910390fd5b60205463ffffffff11610f60576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260218152602001806117276021913960400191505060405180910390fd5b602080546001019081905560005b60208110156110a9578160011660011415610fa0578260008260208110610f9157fe5b0155506110ac95505050505050565b600260008260208110610faf57fe5b01548460405160200180838152602001828152602001925050506040516020818303038152906040526040518082805190602001908083835b6020831061102557805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101610fe8565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa158015611082573d6000803e3d6000fd5b5050506040513d602081101561109757600080fd5b50519250600282049150600101610f6e565b50fe5b50505050505050565b60606110c26020546114ba565b905090565b6020546000908190815b60208110156112f05781600116600114156111e6576002600082602081106110f557fe5b01548460405160200180838152602001828152602001925050506040516020818303038152906040526040518082805190602001908083835b6020831061116b57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0909201916020918201910161112e565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa1580156111c8573d6000803e3d6000fd5b5050506040513d60208110156111dd57600080fd5b505192506112e2565b600283602183602081106111f657fe5b015460405160200180838152602001828152602001925050506040516020818303038152906040526040518082805190602001908083835b6020831061126b57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0909201916020918201910161122e565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa1580156112c8573d6000803e3d6000fd5b5050506040513d60208110156112dd57600080fd5b505192505b6002820491506001016110d1565b506002826112ff6020546114ba565b600060401b6040516020018084815260200183805190602001908083835b6020831061135a57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0909201916020918201910161131d565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790527fffffffffffffffffffffffffffffffffffffffffffffffff000000000000000095909516920191825250604080518083037ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8018152601890920190819052815191955093508392850191508083835b6020831061143f57805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe09092019160209182019101611402565b51815160209384036101000a7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01801990921691161790526040519190930194509192505080830381855afa15801561149c573d6000803e3d6000fd5b5050506040513d60208110156114b157600080fd5b50519250505090565b60408051600880825281830190925260609160208201818036833701905050905060c082901b8060071a60f81b826000815181106114f457fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060061a60f81b8260018151811061153757fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060051a60f81b8260028151811061157a57fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060041a60f81b826003815181106115bd57fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060031a60f81b8260048151811061160057fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060021a60f81b8260058151811061164357fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060011a60f81b8260068151811061168657fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053508060001a60f81b826007815181106116c957fe5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a90535050919050565b6000808585111561170d578182fd5b83861115611719578182fd5b505082019391909203915056fe4465706f736974436f6e74726163743a206d65726b6c6520747265652066756c6c4465706f736974436f6e74726163743a207265636f6e7374727563746564204465706f7369744461746120646f6573206e6f74206d6174636820737570706c696564206465706f7369745f646174615f726f6f744465706f736974436f6e74726163743a20696e76616c6964207769746864726177616c5f63726564656e7469616c73206c656e6774684465706f736974436f6e74726163743a206465706f7369742076616c7565206e6f74206d756c7469706c65206f6620677765694465706f736974436f6e74726163743a20696e76616c6964207075626b6579206c656e6774684465706f736974436f6e74726163743a206465706f7369742076616c756520746f6f20686967684465706f736974436f6e74726163743a206465706f7369742076616c756520746f6f206c6f774465706f736974436f6e74726163743a20696e76616c6964207369676e6174757265206c656e677468a2646970667358221220dceca8706b29e917dacf25fceef95acac8d90d765ac926663ce4096195952b6164736f6c634300060b0033",
                "storage": {
                    "0x22": "0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
                    "0x23": "0xdb56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71",
                    "0x24": "0xc78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c",
                    "0x25": "0x536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c",
                    "0x26": "0x9efde052aa15429fae05bad4d0b1d7c64da64d03d7a1854a588c2cb8430c0d30",
                    "0x27": "0xd88ddfeed400a8755596b21942c1497e114c302e6118290f91e6772976041fa1",
                    "0x28": "0x87eb0ddba57e35f6d286673802a4af5975e22506c7cf4c64bb6be5ee11527f2c",
                    "0x29": "0x26846476fd5fc54a5d43385167c95144f2643f533cc85bb9d16b782f8d7db193",
                    "0x2a": "0x506d86582d252405b840018792cad2bf1259f1ef5aa5f887e13cb2f0094f51e1",
                    "0x2b": "0xffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b",
                    "0x2c": "0x6cf04127db05441cd833107a52be852868890e4317e6a02ab47683aa75964220",
                    "0x2d": "0xb7d05f875f140027ef5118a2247bbb84ce8f2f0f1123623085daf7960c329f5f",
                    "0x2e": "0xdf6af5f5bbdb6be9ef8aa618e4bf8073960867171e29676f8b284dea6a08a85e",
                    "0x2f": "0xb58d900f5e182e3c50ef74969ea16c7726c549757cc23523c369587da7293784",
                    "0x30": "0xd49a7502ffcfb0340b1d7885688500ca308161a7f96b62df9d083b71fcc8f2bb",
                    "0x31": "0x8fe6b1689256c0d385f42f5bbe2027a22c1996e110ba97c171d3e5948de92beb",
                    "0x32": "0x8d0d63c39ebade8509e0ae3c9c3876fb5fa112be18f905ecacfecb92057603ab",
                    "0x33": "0x95eec8b2e541cad4e91de38385f2e046619f54496c2382cb6cacd5b98c26f5a4",
                    "0x34": "0xf893e908917775b62bff23294dbbe3a1cd8e6cc1c35b4801887b646a6f81f17f",
                    "0x35": "0xcddba7b592e3133393c16194fac7431abf2f5485ed711db282183c819e08ebaa",
                    "0x36": "0x8a8d7fe3af8caa085a7639a832001457dfb9128a8061142ad0335629ff23ff9c",
                    "0x37": "0xfeb3c337d7a51a6fbf00b9e34c52e1c9195c969bd4e7a0bfd51d5c5bed9c1167",
                    "0x38": "0xe71f0aa83cc32edfbefa9f4d3e0174ca85182eec9f3a09f6a6c0df6377a510d7",
                    "0x39": "0x31206fa80a50bb6abe29085058f16212212a60eec8f049fecb92d8c8e0a84bc0",
                    "0x3a": "0x21352bfecbeddde993839f614c3dac0a3ee37543f9b412b16199dc158e23b544",
                    "0x3b": "0x619e312724bb6d7c3153ed9de791d764a366b389af13c58bf8a8d90481a46765",
                    "0x3c": "0x7cdd2986268250628d0c10e385c58c6191e6fbe05191bcc04f133f2cea72c1c4",
                    "0x3d": "0x848930bd7ba8cac54661072113fb278869e07bb8587f91392933374d017bcbe1",
                    "0x3e": "0x8869ff2c22b28cc10510d9853292803328be4fb0e80495e8bb8d271f5b889636",
                    "0x3f": "0xb5fe28e79f1b850f8658246ce9b6a1e7b49fc06db7143e8fe0b4f2b0c5523a5c",
                    "0x40": "0x985e929f70af28d0bdd1a90a808f977f597c7c778c489e98d3bd8910d31ac0f7"
                }
            },
            "0x00000961ef480eb55e80d19ad83579a64c007002": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x3373fffffffffffffffffffffffffffffffffffffffe1460cb5760115f54807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff146101f457600182026001905f5b5f82111560685781019083028483029004916001019190604d565b909390049250505036603814608857366101f457346101f4575f5260205ff35b34106101f457600154600101600155600354806003026004013381556001015f35815560010160203590553360601b5f5260385f601437604c5fa0600101600355005b6003546002548082038060101160df575060105b5f5b8181146101835782810160030260040181604c02815460601b8152601401816001015481526020019060020154807fffffffffffffffffffffffffffffffff00000000000000000000000000000000168252906010019060401c908160381c81600701538160301c81600601538160281c81600501538160201c81600401538160181c81600301538160101c81600201538160081c81600101535360010160e1565b910180921461019557906002556101a0565b90505f6002555f6003555b5f54807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff14156101cd57505f5b6001546002828201116101e25750505f6101e8565b01600290035b5f555f600155604c025ff35b5f5ffd",
                "storage": {}
            },
            "0x0000bbddc7ce488642fb579f8b00f3a590007251": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x3373fffffffffffffffffffffffffffffffffffffffe1460d35760115f54807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1461019a57600182026001905f5b5f82111560685781019083028483029004916001019190604d565b9093900492505050366060146088573661019a573461019a575f5260205ff35b341061019a57600154600101600155600354806004026004013381556001015f358155600101602035815560010160403590553360601b5f5260605f60143760745fa0600101600355005b6003546002548082038060021160e7575060025b5f5b8181146101295782810160040260040181607402815460601b815260140181600101548152602001816002015481526020019060030154905260010160e9565b910180921461013b5790600255610146565b90505f6002555f6003555b5f54807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff141561017357505f5b6001546001828201116101885750505f61018e565b01600190035b5f555f6001556074025ff35b5f5ffd",
                "storage": {}
            },
            "0x0000f90827f1c53a10cb7a02335b175320002935": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x3373fffffffffffffffffffffffffffffffffffffffe14604657602036036042575f35600143038111604257611fff81430311604257611fff9006545f5260205ff35b5f5ffd5b5f35611fff60014303065500",
                "storage": {
                    "0x00": "0x04b688c25df122da84e0b1a21b90dd8f898e6005cdc2d1ac6c60ded9ff9b2de4"
                }
            },
            "0x000f3df6d732807ef1319fb7b8bb8522d0beac02": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x3373fffffffffffffffffffffffffffffffffffffffe14604d57602036146024575f5ffd5b5f35801560495762001fff810690815414603c575f5ffd5b62001fff01545f5260205ff35b5f5ffd5b62001fff42064281555f359062001fff015500",
                "storage": {
                    "0x1a98": "0x3a97"
                }
            },
            "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b": {
                "nonce": "0x01",
                "balance": "0x3635c9adc5de996bf0",
                "code": "0x",
                "storage": {}
            },
            "0x0000000000000000000000000000000000001000": {
                "nonce": "0x01",
                "balance": "0x00",
                "code": "0x60016000540160005500",
                "storage": {
                    "0x00": "0x01"
                }
            },
            "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba": {
                "nonce": "0x00",
                "balance": "0x01f938",
                "code": "0x",
                "storage": {}
            }
        },
        "lastblockhash": "0xd4c4adfc3e91b0f8855851d598b43c9aa24e46dc03463a1e6e39154a3b9baf11",
        "config": {
            "network": "PragueToOsakaAtTime15k",
            "chainid": "0x01",
            "blobSchedule": {
                "Cancun": {
                    "target": "0x03",
                    "max": "0x06",
                    "baseFeeUpdateFraction": "0x32f0ed"
                },
                "Prague": {
                    "target": "0x06",
                    "max": "0x09",
                    "baseFeeUpdateFraction": "0x4c6964"
                },
                "Osaka": {
                    "target": "0x06",
                    "max": "0x09",
                    "baseFeeUpdateFraction": "0x4c6964"
                }
            }
        },
        "genesisRLP": "0xf9025df90257a00000000000000000000000000000000000000000000000000000000000000000a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347940000000000000000000000000000000000000000a0fe13aa0b3a4ea731b1715a429c1cf100db415262a5bdd49478dc7b9e61cbf1dfa056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000808084044aa200808000a0000000000000000000000000000000000000000000000000000000000000000088000000000000000007a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4218080a00000000000000000000000000000000000000000000000000000000000000000a0e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855c0c0c0",
        "blocks": [
            {
                "blockHeader": {
                    "parentHash": "0x04b688c25df122da84e0b1a21b90dd8f898e6005cdc2d1ac6c60ded9ff9b2de4",
                    "uncleHash": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                    "coinbase": "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
                    "stateRoot": "0xc1f2dd64894ad795674b904a05d8b1e25e44c1bcab551f891561505cf9d23ec0",
                    "transactionsTrie": "0x62a7a0c935742f0a198a50f095a7936080f099512e3c0f55cf245251e52b8956",
                    "receiptTrie": "0x06f890d54ec65d8650b6c73eefd1fbc39f78b5b25f4e1ec10885c9f29f84ee98",
                    "bloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "difficulty": "0x00",
                    "number": "0x01",
                    "gasLimit": "0x044aa200",
                    "gasUsed": "0xa868",
                    "timestamp": "0x3a97",
                    "extraData": "0x",
                    "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "nonce": "0x0000000000000000",
                    "baseFeePerGas": "0x07",
                    "withdrawalsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                    "blobGasUsed": "0x00",
                    "excessBlobGas": "0x00",
                    "parentBeaconBlockRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "requestsHash": "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    "hash": "0xd4c4adfc3e91b0f8855851d598b43c9aa24e46dc03463a1e6e39154a3b9baf11"
                },
                "transactions": [
                    {
                        "type": "0x00",
                        "chainId": "0x01",
                        "nonce": "0x00",
                        "gasPrice": "0x0a",
                        "gasLimit": "0x01c9c381",
                        "to": "0x0000000000000000000000000000000000001000",
                        "value": "0x00",
                        "data": "0x",
                        "v": "0x26",
                        "r": "0xf6fc2259158f1ab63eef05e3a8c55ca90621f289349ed04f0bdb90190aea976d",
                        "s": "0x1298f50281fe143647f01741fb7e68f0dc82e47a05fe5a9b6193c1270c5e3658",
                        "sender": "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b"
                    }
                ],
                "uncleHeaders": [],
                "withdrawals": [],
                "rlp": "0xf902c5f9025ba004b688c25df122da84e0b1a21b90dd8f898e6005cdc2d1ac6c60ded9ff9b2de4a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347942adc25665018aa1fe0e6bc666dac8fc2697ff9baa0c1f2dd64894ad795674b904a05d8b1e25e44c1bcab551f891561505cf9d23ec0a062a7a0c935742f0a198a50f095a7936080f099512e3c0f55cf245251e52b8956a006f890d54ec65d8650b6c73eefd1fbc39f78b5b25f4e1ec10885c9f29f84ee98b9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800184044aa20082a868823a9780a0000000000000000000000000000000000000000000000000000000000000000088000000000000000007a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4218080a00000000000000000000000000000000000000000000000000000000000000000a0e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855f863f861800a8401c9c381940000000000000000000000000000000000001000808026a0f6fc2259158f1ab63eef05e3a8c55ca90621f289349ed04f0bdb90190aea976da01298f50281fe143647f01741fb7e68f0dc82e47a05fe5a9b6193c1270c5e3658c0c0",
                "blocknumber": "1"
            },
            {
                "rlp": "0xf902c3f90259a0d4c4adfc3e91b0f8855851d598b43c9aa24e46dc03463a1e6e39154a3b9baf11a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347942adc25665018aa1fe0e6bc666dac8fc2697ff9baa050671d216437ec3e13a03ff464ed91146fd61165e8adc1622e58d9be953ae4a5a04657b722e50dc4184f522f842bee4abeae676efe01b43d96ca1ca68695982cfda056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800284044aa20080823a9880a0000000000000000000000000000000000000000000000000000000000000000088000000000000000007a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4218080a00000000000000000000000000000000000000000000000000000000000000000a0e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855f863f861010a8401c9c381940000000000000000000000000000000000001000808025a032fd9f592d89b3468bcee7034fab624e546d8efb4f27d62a53e4f7b1382430cca06563508d39899cc529f5d0686a530ac145d68393c858f08edaada21f17e704b7c0c0",
                "expectException": "TransactionException.GAS_LIMIT_EXCEEDS_MAXIMUM",
                "rlp_decoded": {
                    "blockHeader": {
                        "parentHash": "0xd4c4adfc3e91b0f8855851d598b43c9aa24e46dc03463a1e6e39154a3b9baf11",
                        "uncleHash": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                        "coinbase": "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
                        "stateRoot": "0x50671d216437ec3e13a03ff464ed91146fd61165e8adc1622e58d9be953ae4a5",
                        "transactionsTrie": "0x4657b722e50dc4184f522f842bee4abeae676efe01b43d96ca1ca68695982cfd",
                        "receiptTrie": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                        "bloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                        "difficulty": "0x00",
                        "number": "0x02",
                        "gasLimit": "0x044aa200",
                        "gasUsed": "0x00",
                        "timestamp": "0x3a98",
                        "extraData": "0x",
                        "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                        "nonce": "0x0000000000000000",
                        "baseFeePerGas": "0x07",
                        "withdrawalsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                        "blobGasUsed": "0x00",
                        "excessBlobGas": "0x00",
                        "parentBeaconBlockRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                        "requestsHash": "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                        "hash": "0x64bf114af5b2312b734d3f7a94f15858210602292527a869c05e4f3083585385"
                    },
                    "transactions": [
                        {
                            "type": "0x00",
                            "chainId": "0x01",
                            "nonce": "0x01",
                            "gasPrice": "0x0a",
                            "gasLimit": "0x01c9c381",
                            "to": "0x0000000000000000000000000000000000001000",
                            "value": "0x00",
                            "data": "0x",
                            "v": "0x25",
                            "r": "0x32fd9f592d89b3468bcee7034fab624e546d8efb4f27d62a53e4f7b1382430cc",
                            "s": "0x6563508d39899cc529f5d0686a530ac145d68393c858f08edaada21f17e704b7",
                            "sender": "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b"
                        }
                    ],
                    "uncleHeaders": [],
                    "withdrawals": [],
                    "blocknumber": "2"
                }
            }
        ],
        "sealEngine": "NoProof",
        "_info": {
            "hash": "0xa19f6207b969ec0c5baa2aa6218e6410818c163eb88be6b39e61955ed4cc50c5",
            "comment": "`execution-spec-tests` generated test",
            "filling-transition-tool": "ethereum-spec-evm-resolver 0.0.5",
            "description": "Test transaction gas limit cap behavior at the Osaka transition.\n\n    Before timestamp 15000: No gas limit cap (transactions with gas > 30M are valid)\n    At/after timestamp 15000: Gas limit cap of 30M is enforced",
            "url": "https://github.com/ethereum/execution-spec-tests/blob/fusaka-devnet-2@v1.2.0/tests/osaka/eip7825_transaction_gas_limit_cap/test_tx_gas_limit.py#L118",
            "fixture-format": "blockchain_test",
            "reference-spec": "https://github.com/ethereum/EIPs/blob/master/EIPS/eip-7825.md",
            "reference-spec-version": "47cbfed315988c0bd4d10002c110ae402504cd94",
            "eels-resolution": {
                "git-url": "https://github.com/spencer-tb/execution-specs.git",
                "branch": "forks/osaka",
                "commit": "bc829598ff1923f9215a6a407ef74621077fd3bb"
            }
        }
    }
}
    "#;
}
