use crate::db::Database;
use crate::journaled_state::JournaledState;
use crate::primitives::{Address, Bytecode, Env, B256, U256};
use revm_precompile::Precompiles;

#[derive(Debug)]
pub struct EVMData<'a, DB: Database> {
    pub env: &'a mut Env,
    pub journaled_state: JournaledState,
    pub db: &'a mut DB,
    pub error: Option<DB::Error>,
    pub precompiles: Precompiles,
    /// Used as temporary value holder to store L1 block info.
    #[cfg(feature = "optimism")]
    pub l1_block_info: Option<crate::optimism::L1BlockInfo>,
}

impl<'a, DB: Database> EVMData<'a, DB> {
    pub fn env(&mut self) -> &mut Env {
        self.env
    }

    pub fn block_hash(&mut self, number: U256) -> Option<B256> {
        self.db
            .block_hash(number)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    pub fn load_account(&mut self, address: Address) -> Option<(bool, bool)> {
        self.journaled_state
            .load_account_exist(address, self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    pub fn balance(&mut self, address: Address) -> Option<(U256, bool)> {
        self.journaled_state
            .load_account(address, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
            .map(|(acc, is_cold)| (acc.info.balance, is_cold))
    }

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

    pub fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.journaled_state.tload(address, index)
    }

    pub fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.journaled_state.tstore(address, index, value)
    }
}
