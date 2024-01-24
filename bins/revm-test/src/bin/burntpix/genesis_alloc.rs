use revm::primitives::{Address, Bytes, U256};
use std::{collections::HashMap, str::FromStr};

use crate::static_data::{
    BURNTPIX_ADDRESS_ONE, BURNTPIX_ADDRESS_THREE, BURNTPIX_ADDRESS_TWO, BURNTPIX_BYTECODE_FOUR,
    BURNTPIX_BYTECODE_ONE, BURNTPIX_BYTECODE_THREE, BURNTPIX_BYTECODE_TWO, BURNTPIX_MAIN_ADDRESS,
    STORAGE_ONE, STORAGE_TWO, STORAGE_ZERO,
};

/// Generating genesis allocs as from <https://github.com/karalabe/burntpix-benchmark/blob/main/genesis.json>
pub struct GenesisAlloc {
    pub contracts: HashMap<Address, Bytes>,
    pub storage: HashMap<U256, U256>,
}

impl GenesisAlloc {
    pub fn new() -> GenesisAlloc {
        let mut instance = GenesisAlloc {
            contracts: HashMap::new(),
            storage: HashMap::new(),
        };

        instance.populate_contracts();
        instance.populate_storage();

        instance
    }
    fn populate_contracts(&mut self) {
        self.contracts
            .insert(BURNTPIX_ADDRESS_ONE, BURNTPIX_BYTECODE_ONE);
        self.contracts
            .insert(BURNTPIX_MAIN_ADDRESS, BURNTPIX_BYTECODE_TWO);
        self.contracts
            .insert(BURNTPIX_ADDRESS_TWO, BURNTPIX_BYTECODE_THREE);
        self.contracts
            .insert(BURNTPIX_ADDRESS_THREE, BURNTPIX_BYTECODE_FOUR);
    }

    fn populate_storage(&mut self) {
        self.storage
            .insert(U256::ZERO, U256::from_str(STORAGE_ZERO).unwrap());
        self.storage.insert(
            U256::from_str("0x1").unwrap(),
            U256::from_str(STORAGE_ONE).unwrap(),
        );
        self.storage.insert(
            U256::from_str("0x2").unwrap(),
            U256::from_str(STORAGE_TWO).unwrap(),
        );
    }
}

impl Default for GenesisAlloc {
    fn default() -> Self {
        Self::new()
    }
}
