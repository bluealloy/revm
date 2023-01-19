use crate::{Bytecode, B160, B256, KECCAK_EMPTY, U256};
use hashbrown::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Account {
    /// Balance of the account.
    pub info: AccountInfo,
    /// storage cache
    pub storage: HashMap<U256, StorageSlot>,
    /// If account is newly created, we will not ask database for storage values
    pub storage_cleared: bool,
    /// if account is destroyed it will be scheduled for removal.
    pub is_destroyed: bool,
    /// if account is touched
    pub is_touched: bool,
    /// used only for pre spurious dragon hardforks where exisnting and empty was two saparate states.
    /// it became same state after EIP-161: State trie clearing
    pub is_not_existing: bool,
}

pub type State = HashMap<B160, Account>;
pub type Storage = HashMap<U256, StorageSlot>;

impl Account {
    pub fn is_empty(&self) -> bool {
        self.info.is_empty()
    }
    pub fn new_not_existing() -> Self {
        Self {
            info: AccountInfo::default(),
            storage: HashMap::new(),
            storage_cleared: false,
            is_destroyed: false,
            is_touched: false,
            is_not_existing: true,
        }
    }
}

impl From<AccountInfo> for Account {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            storage: HashMap::new(),
            storage_cleared: false,
            is_destroyed: false,
            is_touched: false,
            is_not_existing: false,
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageSlot {
    pub original_value: U256,
    /// When loaded with sload present value is set to original value
    pub present_value: U256,
}

impl StorageSlot {
    pub fn new(original: U256) -> Self {
        Self {
            original_value: original,
            present_value: original,
        }
    }

    /// Returns true if the present value differs from the original value
    pub fn is_changed(&self) -> bool {
        self.original_value != self.present_value
    }

    pub fn original_value(&self) -> U256 {
        self.original_value
    }

    pub fn present_value(&self) -> U256 {
        self.present_value
    }
}

/// AccountInfo account information.
#[derive(Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountInfo {
    /// Account balance.
    pub balance: U256,
    /// Account nonce.
    pub nonce: u64,
    /// code hash,
    pub code_hash: B256,
    /// code: if None, `code_by_hash` will be used to fetch it if code needs to be loaded from
    /// inside of revm.
    pub code: Option<Bytecode>,
}

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            balance: U256::ZERO,
            code_hash: KECCAK_EMPTY,
            code: Some(Bytecode::new()),
            nonce: 0,
        }
    }
}

impl PartialEq for AccountInfo {
    fn eq(&self, other: &Self) -> bool {
        self.balance == other.balance
            && self.nonce == other.nonce
            && self.code_hash == other.code_hash
    }
}

impl AccountInfo {
    pub fn new(balance: U256, nonce: u64, code: Bytecode) -> Self {
        let code_hash = code.hash();
        Self {
            balance,
            nonce,
            code: Some(code),
            code_hash,
        }
    }

    pub fn is_empty(&self) -> bool {
        let code_empty = self.code_hash == KECCAK_EMPTY || self.code_hash == B256::zero();
        self.balance == U256::ZERO && self.nonce == 0 && code_empty
    }

    pub fn exists(&self) -> bool {
        !self.is_empty()
    }

    pub fn from_balance(balance: U256) -> Self {
        AccountInfo {
            balance,
            ..Default::default()
        }
    }
}
