use crate::db::Database;
use crate::journaled_state::JournaledState;
use crate::primitives::{Address, Bytecode, Env, B256, U256};
use revm_precompile::Precompiles;

/// EVM Data contains all the data that EVM needs to execute.
#[derive(Debug)]
pub struct EVMData<'a, DB: Database> {
    /// EVM Environment contains all the information about config, block and transaction that
    /// evm needs.
    pub env: &'a mut Env,
    /// EVM State with journaling support.
    pub journaled_state: JournaledState,
    /// Database to load data from.
    pub db: &'a mut DB,
    /// Error that happened during execution.
    pub error: Option<DB::Error>,
    /// Precompiles that are available for evm.
    pub precompiles: Precompiles,
    /// Used as temporary value holder to store L1 block info.
    #[cfg(feature = "optimism")]
    pub l1_block_info: Option<crate::optimism::L1BlockInfo>,
}

impl<'a, DB: Database> EVMData<'a, DB> {
    /// Return environment.
    pub fn env(&mut self) -> &mut Env {
        self.env
    }

    /// Fetch block hash from database.
    pub fn block_hash(&mut self, number: U256) -> Option<B256> {
        self.db
            .block_hash(number)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Load account and return flags (is_cold, exists)
    pub fn load_account(&mut self, address: Address) -> Option<(bool, bool)> {
        self.journaled_state
            .load_account_exist(address, self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Return account balance and is_cold flag.
    pub fn balance(&mut self, address: Address) -> Option<(U256, bool)> {
        self.journaled_state
            .load_account(address, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
            .map(|(acc, is_cold)| (acc.info.balance, is_cold))
    }

    /// Return account code and if address is cold loaded.
    pub fn code(&mut self, address: Address) -> Option<(Bytecode, bool)> {
        let (acc, is_cold) = self
            .journaled_state
            .load_code(address, self.db)
            .map_err(|e| self.error = Some(e))
            .ok()?;
        Some((acc.info.code.clone().unwrap(), is_cold))
    }

    /// Get code hash of address.
    pub fn code_hash(&mut self, address: Address) -> Option<(B256, bool)> {
        let (acc, is_cold) = self
            .journaled_state
            .load_code(address, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .ok()?;
        if acc.is_empty() {
            return Some((B256::ZERO, is_cold));
        }

        Some((acc.info.code_hash, is_cold))
    }

    /// Load storage slot, if storage is not present inside the account then it will be loaded from database.
    pub fn sload(&mut self, address: Address, index: U256) -> Option<(U256, bool)> {
        // account is always warm. reference on that statement https://eips.ethereum.org/EIPS/eip-2929 see `Note 2:`
        self.journaled_state
            .sload(address, index, self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Storage change of storage slot, before storing `sload`` will be called for that slot.
    pub fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        self.journaled_state
            .sstore(address, index, value, self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Returns transient storage value.
    pub fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.journaled_state.tload(address, index)
    }

    /// Stores transient storage value.
    pub fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.journaled_state.tstore(address, index, value)
    }
}
