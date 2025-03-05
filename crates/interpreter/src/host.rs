use context_interface::{
    context::{ContextTr, SStoreResult, SelfDestructResult, StateLoad},
    journaled_state::AccountLoad,
    Block, Cfg, Database, Journal, Transaction, TransactionType,
};
use primitives::{Address, Bytes, Log, B256, BLOCK_HASH_HISTORY, U256};

use crate::instructions::utility::IntoU256;

/// Host trait with all methods are needed by the Interpreter.
///
/// This trait is implemented for all types that have `ContextTr` trait.
pub trait Host {
    /* Block */
    fn basefee(&self) -> U256;
    fn blob_gasprice(&self) -> U256;
    fn gas_limit(&self) -> U256;
    fn difficulty(&self) -> U256;
    fn prevrandao(&self) -> Option<U256>;
    fn block_number(&self) -> u64;
    fn timestamp(&self) -> U256;
    fn beneficiary(&self) -> Address;
    fn chain_id(&self) -> U256;

    /* Transaction */
    fn effective_gas_price(&self) -> U256;
    fn caller(&self) -> Address;
    fn blob_hash(&self, number: usize) -> Option<U256>;

    /* Config */
    fn max_initcode_size(&self) -> usize;

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
    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>>;
    fn sload(&mut self, address: Address, key: U256) -> Option<StateLoad<U256>>;
    fn tstore(&mut self, address: Address, key: U256, value: U256);
    fn tload(&mut self, address: Address, key: U256) -> U256;
    fn load_account_delegated(&mut self, address: Address) -> Option<StateLoad<AccountLoad>>;
    fn load_account_code(&mut self, address: Address) -> Option<StateLoad<Bytes>>;
    fn load_account_code_hash(&mut self, address: Address) -> Option<StateLoad<B256>>;
}

impl<CTX: ContextTr> Host for CTX {
    /* Block */

    fn basefee(&self) -> U256 {
        U256::from(self.block().basefee())
    }

    fn blob_gasprice(&self) -> U256 {
        U256::from(self.block().blob_gasprice().unwrap_or(0))
    }

    fn gas_limit(&self) -> U256 {
        U256::from(self.block().gas_limit())
    }

    fn difficulty(&self) -> U256 {
        self.block().difficulty()
    }

    fn prevrandao(&self) -> Option<U256> {
        self.block().prevrandao().map(|r| r.into_u256())
    }

    fn block_number(&self) -> u64 {
        self.block().number()
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

    fn effective_gas_price(&self) -> U256 {
        let basefee = self.block().basefee();
        U256::from(self.tx().effective_gas_price(basefee as u128))
    }

    fn caller(&self) -> Address {
        self.tx().caller()
    }

    /* Transaction */

    fn blob_hash(&self, number: usize) -> Option<U256> {
        let tx = &self.tx();
        if tx.tx_type() != TransactionType::Eip4844 {
            return None;
        }
        tx.blob_versioned_hashes()
            .get(number)
            .map(|t| U256::from_be_bytes(t.0))
    }

    /* Config */

    fn max_initcode_size(&self) -> usize {
        self.cfg().max_code_size().saturating_mul(2)
    }

    /* State */

    fn block_hash(&mut self, requested_number: u64) -> Option<B256> {
        let block_number = self.block().number();

        let Some(diff) = block_number.checked_sub(requested_number) else {
            return Some(B256::ZERO);
        };

        // blockhash should push zero if number is same as current block number.
        if diff == 0 {
            return Some(B256::ZERO);
        }

        if diff <= BLOCK_HASH_HISTORY {
            return self
                .journal()
                .db()
                .block_hash(requested_number)
                .map_err(|e| {
                    *self.error() = Err(e);
                })
                .ok();
        }

        Some(B256::ZERO)
    }

    fn load_account_delegated(&mut self, address: Address) -> Option<StateLoad<AccountLoad>> {
        self.journal()
            .load_account_delegated(address)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Gets balance of `address` and if the account is cold.
    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>> {
        self.journal()
            .load_account(address)
            .map(|acc| acc.map(|a| a.info.balance))
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Gets code of `address` and if the account is cold.
    fn load_account_code(&mut self, address: Address) -> Option<StateLoad<Bytes>> {
        self.journal()
            .code(address)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Gets code hash of `address` and if the account is cold.
    fn load_account_code_hash(&mut self, address: Address) -> Option<StateLoad<B256>> {
        self.journal()
            .code_hash(address)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Gets storage value of `address` at `index` and if the account is cold.
    fn sload(&mut self, address: Address, index: U256) -> Option<StateLoad<U256>> {
        self.journal()
            .sload(address, index)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Sets storage value of account address at index.
    ///
    /// Returns [`StateLoad`] with [`SStoreResult`] that contains original/new/old storage value.
    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<StateLoad<SStoreResult>> {
        self.journal()
            .sstore(address, index, value)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }

    /// Gets the transient storage value of `address` at `index`.
    fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.journal().tload(address, index)
    }

    /// Sets the transient storage value of `address` at `index`.
    fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.journal().tstore(address, index, value)
    }

    /// Emits a log owned by `address` with given `LogData`.
    fn log(&mut self, log: Log) {
        self.journal().log(log);
    }

    /// Marks `address` to be deleted, with funds transferred to `target`.
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Option<StateLoad<SelfDestructResult>> {
        self.journal()
            .selfdestruct(address, target)
            .map_err(|e| {
                *self.error() = Err(e);
            })
            .ok()
    }
}
