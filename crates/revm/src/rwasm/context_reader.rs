use crate::primitives::{Address, Env, B256, U256};
use fluentbase_sdk::{
    BlockContext,
    BlockContextReader,
    SovereignContextReader,
    TxContext,
    TxContextReader,
};

pub struct EnvContextReader(pub Box<Env>);

impl BlockContextReader for EnvContextReader {
    fn block_chain_id(&self) -> u64 {
        self.0.cfg.chain_id
    }

    fn block_coinbase(&self) -> Address {
        self.0.block.coinbase
    }

    fn block_timestamp(&self) -> u64 {
        self.0.block.timestamp.as_limbs()[0]
    }

    fn block_number(&self) -> u64 {
        self.0.block.number.as_limbs()[0]
    }

    fn block_difficulty(&self) -> U256 {
        self.0.block.difficulty
    }

    fn block_prev_randao(&self) -> B256 {
        self.0.block.prevrandao.unwrap_or_default()
    }

    fn block_gas_limit(&self) -> u64 {
        self.0.block.gas_limit.as_limbs()[0]
    }

    fn block_base_fee(&self) -> U256 {
        self.0.block.basefee
    }
}

impl TxContextReader for EnvContextReader {
    fn tx_gas_limit(&self) -> u64 {
        self.0.tx.gas_limit
    }

    fn tx_nonce(&self) -> u64 {
        self.0.tx.nonce.unwrap_or_default()
    }

    fn tx_gas_price(&self) -> U256 {
        self.0.tx.gas_price
    }

    fn tx_gas_priority_fee(&self) -> Option<U256> {
        self.0.tx.gas_priority_fee
    }

    fn tx_origin(&self) -> Address {
        self.0.tx.caller
    }

    fn tx_value(&self) -> U256 {
        self.0.tx.value
    }
}

impl SovereignContextReader for EnvContextReader {
    fn clone_block_context(&self) -> BlockContext {
        BlockContext {
            chain_id: self.block_chain_id(),
            coinbase: self.block_coinbase(),
            timestamp: self.block_timestamp(),
            number: self.block_number(),
            difficulty: self.block_difficulty(),
            prev_randao: self.block_prev_randao(),
            gas_limit: self.block_gas_limit(),
            base_fee: self.block_base_fee(),
        }
    }

    fn clone_tx_context(&self) -> TxContext {
        TxContext {
            gas_limit: self.tx_gas_limit(),
            nonce: self.tx_nonce(),
            gas_price: self.tx_gas_price(),
            gas_priority_fee: self.tx_gas_priority_fee(),
            origin: self.tx_origin(),
            value: self.tx_value(),
        }
    }
}
