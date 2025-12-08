use crate::{AccountInfo, Env, SpecName, Test, TransactionParts};
use revm::{
    context::{block::BlockEnv, cfg::CfgEnv},
    database::CacheState,
    primitives::{hardfork::SpecId, keccak256, Address, Bytes, HashMap, B256},
    state::Bytecode,
};
use serde::Deserialize;
use std::collections::BTreeMap;

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
    pub fn block_env(&self, cfg: &mut CfgEnv) -> BlockEnv {
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
        // Use spec-aware blob fee fraction: Cancun uses 3338477, Prague/Osaka use 5007716
        if let Some(current_excess_blob_gas) = self.env.current_excess_blob_gas {
            block.set_blob_excess_gas_and_price(
                current_excess_blob_gas.to(),
                cfg.blob_base_fee_update_fraction(),
            );
        }

        // Set default prevrandao for merge
        if cfg.spec.is_enabled_in(SpecId::MERGE) && block.prevrandao.is_none() {
            block.prevrandao = Some(B256::default());
        }

        block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm::{
        context_interface::block::calc_blob_gasprice,
        primitives::{
            eip4844::{BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN, BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE},
            U256,
        },
    };

    /// Creates a minimal TestUnit with excess blob gas set for testing blob fee calculation
    fn create_test_unit_with_excess_blob_gas(excess_blob_gas: u64) -> TestUnit {
        TestUnit {
            info: None,
            env: Env {
                current_chain_id: None,
                current_coinbase: Address::ZERO,
                current_difficulty: U256::ZERO,
                current_gas_limit: U256::from(1_000_000u64),
                current_number: U256::from(1u64),
                current_timestamp: U256::from(1u64),
                current_base_fee: Some(U256::from(1u64)),
                previous_hash: None,
                current_random: None,
                current_beacon_root: None,
                current_withdrawals_root: None,
                current_excess_blob_gas: Some(U256::from(excess_blob_gas)),
            },
            pre: HashMap::default(),
            post: BTreeMap::default(),
            transaction: TransactionParts {
                tx_type: None,
                data: vec![],
                gas_limit: vec![],
                gas_price: None,
                nonce: U256::ZERO,
                secret_key: B256::ZERO,
                sender: None,
                to: None,
                value: vec![],
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                initcodes: None,
                access_lists: vec![],
                authorization_list: None,
                blob_versioned_hashes: vec![],
                max_fee_per_blob_gas: None,
            },
            out: None,
        }
    }

    /// Test that block_env uses the correct blob base fee update fraction for Cancun
    #[test]
    fn test_block_env_blob_fee_fraction_cancun() {
        let unit = create_test_unit_with_excess_blob_gas(0x240000); // 2,359,296

        let mut cfg = CfgEnv::default();
        cfg.spec = SpecId::CANCUN;

        let block = unit.block_env(&mut cfg);

        // Verify blob gas price is calculated with Cancun fraction
        let blob_info = block
            .blob_excess_gas_and_price
            .expect("blob info should be set");
        assert_eq!(blob_info.excess_blob_gas, 0x240000);

        // Calculate expected price with Cancun fraction (3338477)
        // blob_gasprice = fake_exponential(1, excess_blob_gas, BLOB_BASE_FEE_UPDATE_FRACTION)
        // With excess_blob_gas=0x240000 and CANCUN fraction=3338477, price should be 2
        let expected_price = calc_blob_gasprice(0x240000, BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN);
        assert_eq!(blob_info.blob_gasprice, expected_price);
        assert_eq!(blob_info.blob_gasprice, 2); // With Cancun fraction, price is 2
    }

    /// Test that block_env uses the correct blob base fee update fraction for Prague
    #[test]
    fn test_block_env_blob_fee_fraction_prague() {
        let unit = create_test_unit_with_excess_blob_gas(0x240000); // 2,359,296

        let mut cfg = CfgEnv::default();
        cfg.spec = SpecId::PRAGUE;

        let block = unit.block_env(&mut cfg);

        // Verify blob gas price is calculated with Prague fraction
        let blob_info = block
            .blob_excess_gas_and_price
            .expect("blob info should be set");
        assert_eq!(blob_info.excess_blob_gas, 0x240000);

        // Calculate expected price with Prague fraction (5007716)
        // With excess_blob_gas=0x240000 and PRAGUE fraction=5007716, price should be 1
        let expected_price = calc_blob_gasprice(0x240000, BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE);
        assert_eq!(blob_info.blob_gasprice, expected_price);
        assert_eq!(blob_info.blob_gasprice, 1); // With Prague fraction, price is 1
    }

    /// Test that block_env uses the correct blob base fee update fraction for Osaka
    #[test]
    fn test_block_env_blob_fee_fraction_osaka() {
        let unit = create_test_unit_with_excess_blob_gas(0x240000); // 2,359,296

        let mut cfg = CfgEnv::default();
        cfg.spec = SpecId::OSAKA;

        let block = unit.block_env(&mut cfg);

        // Osaka should use Prague fraction (same as Prague)
        let blob_info = block
            .blob_excess_gas_and_price
            .expect("blob info should be set");
        let expected_price = calc_blob_gasprice(0x240000, BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE);
        assert_eq!(blob_info.blob_gasprice, expected_price);
        assert_eq!(blob_info.blob_gasprice, 1); // With Prague fraction, price is 1
    }

    /// Test that demonstrates the bug scenario from IMPLEMENTATION_PROMPT.md
    /// With excess_blob_gas=0x240000 and maxFeePerBlobGas=0x01:
    /// - Cancun fraction (3338477): blob_price = 2, tx FAILS (insufficient fee)
    /// - Prague fraction (5007716): blob_price = 1, tx SUCCEEDS
    #[test]
    fn test_blob_fee_difference_affects_tx_validity() {
        let excess_blob_gas = 0x240000u64;

        // Calculate prices with both fractions
        let cancun_price =
            calc_blob_gasprice(excess_blob_gas, BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN);
        let prague_price =
            calc_blob_gasprice(excess_blob_gas, BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE);

        // Verify the prices are different
        assert_eq!(cancun_price, 2, "Cancun blob price should be 2");
        assert_eq!(prague_price, 1, "Prague blob price should be 1");

        // With maxFeePerBlobGas=1:
        // - Cancun: 1 < 2, tx would fail with insufficient fee
        // - Prague: 1 >= 1, tx would succeed
        let max_fee_per_blob_gas = 1u128;
        assert!(
            max_fee_per_blob_gas < cancun_price,
            "Tx should fail with Cancun fraction"
        );
        assert!(
            max_fee_per_blob_gas >= prague_price,
            "Tx should succeed with Prague fraction"
        );
    }
}
