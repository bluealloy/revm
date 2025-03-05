use context_interface::{
    context::{ContextTr, SStoreResult, SelfDestructResult, StateLoad},
    journaled_state::AccountLoad,
    Block, Cfg, Database, Journal, Transaction,
};
use primitives::{Address, Bytes, Log, B256, U256};

/// Host trait with all methods that can be called by the Interpreter.
///
/// This trait is implemented for all types that have `ContextTr` trait.
pub trait Host {
    /* Block */
    fn basefee(&self) -> U256;
    fn blob_basefee(&self) -> U256;
    fn gas_limit(&self) -> U256;
    fn difficulty(&self) -> U256;
    fn block_number(&self) -> U256;
    fn timestamp(&self) -> U256;
    fn beneficiary(&self) -> Address;
    fn chain_id(&self) -> U256;

    /* Transaction */
    fn gas_price(&self) -> U256;
    fn caller(&self) -> Address;
    fn blob_hash(&self, number: usize) -> Option<U256>;

    /* State */
    fn block_hash(&mut self, number: u64) -> Option<B256>;
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Option<StateLoad<SelfDestructResult>>;

    fn log(&mut self, log: Log);
    fn sstore(
        &mut self,
        address: Address,
        key: U256,
        value: U256,
    ) -> Option<StateLoad<SStoreResult>>;
    fn sload(&mut self, address: Address, key: U256) -> Option<StateLoad<U256>>;
    fn tstore(&mut self, address: Address, key: U256, value: U256);
    fn tload(&mut self, address: Address, key: U256) -> U256;
    fn load_account_delegated(&mut self, address: Address) -> Option<AccountLoad>;
    fn load_account_code(&mut self, address: Address) -> Option<Bytes>;
    fn load_account_code_hash(&mut self, address: Address) -> Option<B256>;
}

impl<CTX: ContextTr> Host for CTX {
    fn basefee(&self) -> U256 {
        U256::from(self.block().basefee())
    }

    fn blob_basefee(&self) -> U256 {
        U256::from(self.block().blob_gasprice().unwrap_or(0))
    }

    fn gas_limit(&self) -> U256 {
        U256::from(self.block().gas_limit())
    }

    fn difficulty(&self) -> U256 {
        self.block().difficulty()
    }

    fn block_number(&self) -> U256 {
        U256::from(self.block().number())
    }

    fn timestamp(&self) -> U256 {
        U256::from(self.block().timestamp())
    }

    fn beneficiary(&self) -> Address {
        self.block().beneficiary()
    }

    fn chain_id(&self) -> U256 {
        U256::from(self.cfg().chain_id())
    }

    fn gas_price(&self) -> U256 {
        U256::from(self.tx().gas_price())
    }

    fn caller(&self) -> Address {
        self.tx().caller()
    }

    fn blob_hash(&self, number: usize) -> Option<U256> {
        self.tx()
            .blob_versioned_hashes()
            .get(number)
            .map(|b| U256::from_be_bytes(b.0))
    }

    fn block_hash(&mut self, number: u64) -> Option<B256> {
        self.journal()
            .db()
            .block_hash(number)
            .map_err(|e| {
                *self.error() = Err(e);
                ()
            })
            .ok()
    }

    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Option<StateLoad<SelfDestructResult>> {
        self.journal().selfdestruct(address, target)
    }

    fn selfdestruct_result(&mut self) -> Option<StateLoad<SelfDestructResult>> {
        self.journal().selfdestruct_result()
    }

    fn log(&mut self, log: Log) {
        self.journal().log(log)
    }

    fn sstore(
        &mut self,
        address: Address,
        key: U256,
        value: U256,
    ) -> Option<StateLoad<SStoreResult>> {
        self.journal().sstore(address, key, value)
    }
}
