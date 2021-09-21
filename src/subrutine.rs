use std::{
    collections::{HashMap, HashSet},
    fs::File,
    rc::Rc,
    thread::AccessError,
};

use bytes::Bytes;
use primitive_types::{H160, H256, U256};

use crate::{error::ExitError, Basic, Log, Transfer};

pub struct SubRutine {
    /// Applied changes to our state
    state: HashMap<H160, Account>,
    precompiles: HashSet<H160>,
    logs: Vec<Log>,
    /// It contains original values before they were changes.
    /// it is made like this so that we can revert to previous state in case of
    /// exit or revert
    changeset: Vec<HashMap<H160, (Account, Filth)>>,
    // how deep are we in call stack.
    depth: usize,
}

/// SubRutine checkpoint that will help us to go back from this
pub struct SubRutineCheckpoint {
    log_i: usize,
    changeset_i: usize,
    depth: usize,
}

impl SubRutine {
    pub fn new() -> SubRutine {
        Self {
            state: HashMap::new(),
            precompiles: HashSet::new(),
            logs: Vec::new(),
            changeset: Vec::new(),
            depth: 0,
        }
    }

    pub fn touch(&mut self, address: H160) {}

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn inc_nonce(&mut self, address: H160) {
        //TODO self.account_mut(address, backend).basic.nonce += U256::one();
    }

    pub fn transfer(&mut self, from: H160, to: H160, value: U256) -> Result<(), ExitError> {
        // TODO check funds and do transfer
        Ok(())
    }

    pub fn create_checkpoint(&mut self) -> SubRutineCheckpoint {
        self.depth += 1;
        let checkpoint = SubRutineCheckpoint {
            log_i: self.logs.len(),
            changeset_i: self.changeset.len(),
            depth: self.depth,
        };
        self.changeset.push(Default::default());
        checkpoint
    }

    pub fn exit_commit(&mut self, checkpoint: SubRutineCheckpoint) -> Result<(), ExitError> {
        self.depth -= 1;
        Ok(())
    }

    pub fn exit_revert(&mut self, checkpoint: SubRutineCheckpoint) {
        self.logs.truncate(checkpoint.log_i);
        // revert all changes from changeset
        self.changeset.truncate(checkpoint.changeset_i);
        self.depth -= 1;
    }

    pub fn exit_discard(&mut self, checkpoint: SubRutineCheckpoint) {
        self.logs.truncate(checkpoint.log_i);
        // revert all changes from changeset
        self.changeset.truncate(checkpoint.changeset_i);
        self.depth -= 1;
    }

    pub fn set_storage(
        &mut self,
        address: H160,
        index: H256,
        value: H256,
    ) -> Result<bool, ExitError> {
        // TODO see if we need to read account from DB or we need to saparate storage from accounts.
        //self.known_account(&address)
        Ok(true)
    }

    pub fn log(&mut self, log: Log) {
        self.logs.push(log);
    }

    pub fn known_account(&self, address: &H160) -> Option<&Account> {
        self.state.get(&address)
    }
    /*

    pub fn known_basic(&self, address: H160) -> Option<Basic> {
        self.known_account(address).map(|acc| acc.basic.clone())
    }

    pub fn known_code(&self, address: H160) -> Option<Vec<u8>> {
        self.known_account(address).and_then(|acc| acc.code.clone())
    }

    pub fn known_empty(&self, address: H160) -> Option<bool> {
        if let Some(account) = self.known_account(address) {
            if account.basic.balance != U256::zero() {
                return Some(false);
            }

            if account.basic.nonce != U256::zero() {
                return Some(false);
            }

            if let Some(code) = &account.code {
                return Some(
                    account.basic.balance == U256::zero()
                        && account.basic.nonce == U256::zero()
                        && code.is_empty(),
                );
            }
        }

        None
    }

    pub fn known_storage(&self, address: H160, key: H256) -> Option<H256> {
        if let Some(value) = self.storages.get(&(address, key)) {
            return Some(*value);
        }

        if let Some(account) = self.accounts.get(&address) {
            if account.reset {
                return Some(H256::default());
            }
        }

        if let Some(parent) = self.parent.as_ref() {
            return parent.known_storage(address, key);
        }

        None
    }

    pub fn known_original_storage(&self, address: H160, key: H256) -> Option<H256> {
        if let Some(account) = self.accounts.get(&address) {
            if account.reset {
                return Some(H256::default());
            }
        }

        if let Some(parent) = self.parent.as_ref() {
            return parent.known_original_storage(address, key);
        }

        None
    }*/
}

#[derive(Debug)]
pub struct Account {
    /// Balance of the account.
    pub basic: Basic,
    /// Unmodified account balance.
    pub old_balance: Option<U256>,
    /// Nonce of the account.
    pub code: Option<Bytes>,
    /// code hash
    pub code_hash: H256,
    /// storage cache
    pub storage: HashMap<H256, H256>,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum Filth {
    /// account loaded but not modified
    Cached,
    /// Account or any of its part is modified.
    Dirty,
    /// Account is selfdestructed
    Selfdestruct,
}
