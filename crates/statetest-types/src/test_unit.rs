use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

use crate::{AccountInfo, Env, SpecName, Test, TransactionParts};
use revm::{
    context::{block::BlockEnv, cfg::CfgEnv},
    context_interface::block::calc_excess_blob_gas,
    database::CacheState,
    primitives::{
        eip4844::TARGET_BLOB_GAS_PER_BLOCK_CANCUN, hardfork::SpecId, keccak256, Address, Bytes,
        B256,
    },
    state::Bytecode,
};

/// Single test unit struct
#[derive(Debug, PartialEq, Eq, Deserialize)]
//#[serde(deny_unknown_fields)]
// field config
pub struct TestUnit {
    /// Test info is optional.
    #[serde(default, rename = "_info")]
    pub info: Option<serde_json::Value>,

    /// Test environment configuration.
    ///
    /// Contains the environmental information for executing the test, including
    /// block information, coinbase address, difficulty, gas limit, and other
    /// blockchain state parameters required for proper test execution.
    pub env: Env,

    /// Pre-execution state.
    ///
    /// A mapping of addresses to their account information before the transaction
    /// is executed. This represents the initial state of all accounts involved
    /// in the test, including their balances, nonces, code, and storage.
    pub pre: HashMap<Address, AccountInfo>,

    /// Post-execution expectations per specification.
    ///
    /// Maps each Ethereum specification name (hardfork) to a vector of expected
    /// test results. This allows a single test to define different expected outcomes
    /// for different protocol versions, enabling comprehensive testing across
    /// multiple Ethereum upgrades.
    pub post: BTreeMap<SpecName, Vec<Test>>,

    /// Transaction details to be executed.
    ///
    /// Contains the transaction parameters that will be executed against the
    /// pre-state. This includes sender, recipient, value, data, gas limits,
    /// and other transaction fields that may vary based on indices.
    pub transaction: TransactionParts,

    /// Expected output data from the transaction execution.
    ///
    /// Optional field containing the expected return data from the transaction.
    /// This is typically used for testing contract calls that return specific
    /// values or for CREATE operations that return deployed contract addresses.
    #[serde(default)]
    pub out: Option<Bytes>,
    //pub config
}

impl TestUnit {
    /// Prepare the state from the test unit.
    ///
    /// This function uses [`TestUnit::pre`] to prepare the pre-state from the test unit.
    /// It creates a new cache state and inserts the accounts from the test unit.
    ///
    /// # Returns
    ///
    /// A [`CacheState`] object containing the pre-state accounts and storages.
    pub fn state(&self) -> CacheState {
        let mut cache_state = CacheState::new(false);
        for (address, info) in &self.pre {
            let code_hash = keccak256(&info.code);
            let bytecode = Bytecode::new_raw_checked(info.code.clone())
                .unwrap_or(Bytecode::new_legacy(info.code.clone()));
            let acc_info = revm::state::AccountInfo {
                balance: info.balance,
                code_hash,
                code: Some(bytecode),
                nonce: info.nonce,
            };
            cache_state.insert_account_with_storage(*address, acc_info, info.storage.clone());
        }
        cache_state
    }

    /// Create a block environment from the test unit.
    ///
    /// This function sets up the block environment using the current test unit's
    /// environment settings and the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `cfg` - The configuration environment containing spec and blob settings
    ///
    /// # Returns
    ///
    /// A configured [`BlockEnv`] ready for execution
    pub fn block_env(&self, cfg: &CfgEnv) -> BlockEnv {
        let mut block = BlockEnv {
            number: self.env.current_number,
            beneficiary: self.env.current_coinbase,
            timestamp: self.env.current_timestamp,
            gas_limit: self.env.current_gas_limit.try_into().unwrap_or(u64::MAX),
            basefee: self
                .env
                .current_base_fee
                .unwrap_or_default()
                .try_into()
                .unwrap_or(u64::MAX),
            difficulty: self.env.current_difficulty,
            prevrandao: self.env.current_random,
            ..BlockEnv::default()
        };

        // Handle EIP-4844 blob gas
        if let Some(current_excess_blob_gas) = self.env.current_excess_blob_gas {
            block.set_blob_excess_gas_and_price(
                current_excess_blob_gas.to(),
                revm::primitives::eip4844::BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN,
            );
        } else if let (Some(parent_blob_gas_used), Some(parent_excess_blob_gas)) = (
            self.env.parent_blob_gas_used,
            self.env.parent_excess_blob_gas,
        ) {
            block.set_blob_excess_gas_and_price(
                calc_excess_blob_gas(
                    parent_blob_gas_used.to(),
                    parent_excess_blob_gas.to(),
                    self.env
                        .parent_target_blobs_per_block
                        .map(|i| i.to())
                        .unwrap_or(TARGET_BLOB_GAS_PER_BLOCK_CANCUN),
                ),
                revm::primitives::eip4844::BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN,
            );
        }

        // Set default prevrandao for merge
        if cfg.spec.is_enabled_in(SpecId::MERGE) && block.prevrandao.is_none() {
            block.prevrandao = Some(B256::default());
        }

        block
    }
}
