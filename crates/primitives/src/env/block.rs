use alloy_primitives::{Address, B256, U256};

use crate::{BlobExcessGasAndPrice, BlockEnv};

pub trait Block: Default {
    fn env(&self) -> BlockEnv;
    fn mut_env(&mut self, block: BlockEnv);
    // fn number(&self) -> U256;
    // fn beneficiary(&self) -> Address;
    // fn timestamp(&self) -> U256;
    // fn gas_limit(&self) -> U256;
    // fn basefee(&self) -> U256;
    // fn difficulty(&self) -> U256;
    // fn prevrandao(&self) -> Option<B256>;
    // fn blob_gasprice(&self) -> Option<u128>;
    // fn blob_excess_gas_and_price(&self) -> Option<BlobExcessGasAndPrice>;
}

impl Block for BlockEnv {
    fn env(&self) -> BlockEnv {
        self.clone()
    }
    fn mut_env(&mut self, block: BlockEnv) {
        *self = block;
    }
    // fn number(&self) -> U256 {
    //     self.number
    // }

    // fn beneficiary(&self) -> Address {
    //     self.coinbase
    // }

    // fn timestamp(&self) -> U256 {
    //     self.timestamp
    // }

    // fn gas_limit(&self) -> U256 {
    //     self.gas_limit
    // }

    // fn basefee(&self) -> U256 {
    //     self.basefee
    // }

    // fn difficulty(&self) -> U256 {
    //     self.difficulty
    // }

    // fn prevrandao(&self) -> Option<B256> {
    //     self.prevrandao
    // }

    // fn blob_gasprice(&self) -> Option<u128> {
    //     self.get_blob_gasprice()
    // }

    // fn blob_excess_gas_and_price(&self) -> Option<BlobExcessGasAndPrice> {
    //     self.blob_excess_gas_and_price.clone()
    // }
}
