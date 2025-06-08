use core::convert::Infallible;
use database_interface::{Database, DatabaseCommit, DatabaseRef, EmptyDB};
use primitives::{
    address, hash_map::Entry, Address, HashMap, Log, StorageKey, StorageValue, B256, KECCAK_EMPTY,
    U256,
};
use state::{Account, AccountInfo, Bytecode};
use std::vec::Vec;

/// A [Database] implementation that stores all state changes in memory.
pub type InMemoryDB = CacheDB<EmptyDB>;

/// A cache used in [CacheDB]. Its kept separate so it can be used independently.
///
/// Accounts and code are stored in two separate maps, the `accounts` map maps addresses to [DbAccount],
/// whereas contracts are identified by their code hash, and are stored in the `contracts` map.
/// The [DbAccount] holds the code hash of the contract, which is used to look up the contract in the `contracts` map.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cache {
    /// Account info where None means it is not existing. Not existing state is needed for Pre TANGERINE forks.
    /// `code` is always `None`, and bytecode can be found in `contracts`.
    pub accounts: HashMap<Address, DbAccount>,
    /// Tracks all contracts by their code hash.
    pub contracts: HashMap<B256, Bytecode>,
    /// All logs that were committed via [DatabaseCommit::commit].
    pub logs: Vec<Log>,
    /// All cached block hashes from the [DatabaseRef].
    pub block_hashes: HashMap<U256, B256>,
}

impl Default for Cache {
    fn default() -> Self {
        let mut contracts = HashMap::default();
        contracts.insert(KECCAK_EMPTY, Bytecode::default());
        contracts.insert(B256::ZERO, Bytecode::default());

        Cache {
            accounts: HashMap::default(),
            contracts,
            logs: Vec::default(),
            block_hashes: HashMap::default(),
        }
    }
}

/// A [Database] implementation that stores all state changes in memory.
///
/// This implementation wraps a [DatabaseRef] that is used to load data ([AccountInfo]).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CacheDB<ExtDB> {
    /// The cache that stores all state changes.
    pub cache: Cache,
    /// The underlying database ([DatabaseRef]) that is used to load data.
    ///
    /// Note: This is read-only, data is never written to this database.
    pub db: ExtDB,
}

impl<ExtDB: Default> Default for CacheDB<ExtDB> {
    fn default() -> Self {
        Self::new(ExtDB::default())
    }
}

impl<ExtDb> CacheDB<CacheDB<ExtDb>> {
    /// Flattens a nested cache by applying the outer cache to the inner cache.
    ///
    /// The behavior is as follows:
    /// - Accounts are overridden with outer accounts
    /// - Contracts are overridden with outer contracts
    /// - Logs are appended
    /// - Block hashes are overridden with outer block hashes
    pub fn flatten(self) -> CacheDB<ExtDb> {
        let CacheDB {
            cache:
                Cache {
                    accounts,
                    contracts,
                    logs,
                    block_hashes,
                },
            db: mut inner,
        } = self;

        inner.cache.accounts.extend(accounts);
        inner.cache.contracts.extend(contracts);
        inner.cache.logs.extend(logs);
        inner.cache.block_hashes.extend(block_hashes);
        inner
    }

    /// Discards the outer cache and return the inner cache.
    pub fn discard_outer(self) -> CacheDB<ExtDb> {
        self.db
    }
}

impl<ExtDB> CacheDB<ExtDB> {
    /// Creates a new cache with the given external database.
    pub fn new(db: ExtDB) -> Self {
        Self {
            cache: Cache::default(),
            db,
        }
    }

    /// Inserts the account's code into the cache.
    ///
    /// Accounts objects and code are stored separately in the cache, this will take the code from the account and instead map it to the code hash.
    ///
    /// Note: This will not insert into the underlying external database.
    pub fn insert_contract(&mut self, account: &mut AccountInfo) {
        if let Some(code) = &account.code {
            if !code.is_empty() {
                if account.code_hash == KECCAK_EMPTY {
                    account.code_hash = code.hash_slow();
                }
                self.cache
                    .contracts
                    .entry(account.code_hash)
                    .or_insert_with(|| code.clone());
            }
        }
        if account.code_hash.is_zero() {
            account.code_hash = KECCAK_EMPTY;
        }
    }

    /// Inserts account info but not override storage
    pub fn insert_account_info(&mut self, address: Address, mut info: AccountInfo) {
        self.insert_contract(&mut info);
        self.cache.accounts.entry(address).or_default().info = info;
    }

    /// Wraps the cache in a [CacheDB], creating a nested cache.
    pub fn nest(self) -> CacheDB<Self> {
        CacheDB::new(self)
    }
}

impl<ExtDB: DatabaseRef> CacheDB<ExtDB> {
    /// Returns the account for the given address.
    ///
    /// If the account was not found in the cache, it will be loaded from the underlying database.
    pub fn load_account(&mut self, address: Address) -> Result<&mut DbAccount, ExtDB::Error> {
        let db = &self.db;
        match self.cache.accounts.entry(address) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => Ok(entry.insert(
                db.basic_ref(address)?
                    .map(|info| DbAccount {
                        info,
                        ..Default::default()
                    })
                    .unwrap_or_else(DbAccount::new_not_existing),
            )),
        }
    }

    /// Inserts account storage without overriding account info
    pub fn insert_account_storage(
        &mut self,
        address: Address,
        slot: StorageKey,
        value: StorageValue,
    ) -> Result<(), ExtDB::Error> {
        let account = self.load_account(address)?;
        account.storage.insert(slot, value);
        Ok(())
    }

    /// Replaces account storage without overriding account info
    pub fn replace_account_storage(
        &mut self,
        address: Address,
        storage: HashMap<StorageKey, StorageValue>,
    ) -> Result<(), ExtDB::Error> {
        let account = self.load_account(address)?;
        account.account_state = AccountState::StorageCleared;
        account.storage = storage.into_iter().collect();
        Ok(())
    }
}

impl<ExtDB> DatabaseCommit for CacheDB<ExtDB> {
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        for (address, mut account) in changes {
            if !account.is_touched() {
                continue;
            }
            if account.is_selfdestructed() {
                let db_account = self.cache.accounts.entry(address).or_default();
                db_account.storage.clear();
                db_account.account_state = AccountState::NotExisting;
                db_account.info = AccountInfo::default();
                continue;
            }
            let is_newly_created = account.is_created();
            self.insert_contract(&mut account.info);

            let db_account = self.cache.accounts.entry(address).or_default();
            db_account.info = account.info;

            db_account.account_state = if is_newly_created {
                db_account.storage.clear();
                AccountState::StorageCleared
            } else if db_account.account_state.is_storage_cleared() {
                // Preserve old account state if it already exists
                AccountState::StorageCleared
            } else {
                AccountState::Touched
            };
            db_account.storage.extend(
                account
                    .storage
                    .into_iter()
                    .map(|(key, value)| (key, value.present_value())),
            );
        }
    }
}

impl<ExtDB: DatabaseRef> Database for CacheDB<ExtDB> {
    type Error = ExtDB::Error;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let basic = match self.cache.accounts.entry(address) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(
                self.db
                    .basic_ref(address)?
                    .map(|info| DbAccount {
                        info,
                        ..Default::default()
                    })
                    .unwrap_or_else(DbAccount::new_not_existing),
            ),
        };
        Ok(basic.info())
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        match self.cache.contracts.entry(code_hash) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(entry) => {
                // If you return code bytes when basic fn is called this function is not needed.
                Ok(entry.insert(self.db.code_by_hash_ref(code_hash)?).clone())
            }
        }
    }

    /// Get the value in an account's storage slot.
    ///
    /// It is assumed that account is already loaded.
    fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        match self.cache.accounts.entry(address) {
            Entry::Occupied(mut acc_entry) => {
                let acc_entry = acc_entry.get_mut();
                match acc_entry.storage.entry(index) {
                    Entry::Occupied(entry) => Ok(*entry.get()),
                    Entry::Vacant(entry) => {
                        if matches!(
                            acc_entry.account_state,
                            AccountState::StorageCleared | AccountState::NotExisting
                        ) {
                            Ok(StorageValue::ZERO)
                        } else {
                            let slot = self.db.storage_ref(address, index)?;
                            entry.insert(slot);
                            Ok(slot)
                        }
                    }
                }
            }
            Entry::Vacant(acc_entry) => {
                // Acc needs to be loaded for us to access slots.
                let info = self.db.basic_ref(address)?;
                let (account, value) = if info.is_some() {
                    let value = self.db.storage_ref(address, index)?;
                    let mut account: DbAccount = info.into();
                    account.storage.insert(index, value);
                    (account, value)
                } else {
                    (info.into(), StorageValue::ZERO)
                };
                acc_entry.insert(account);
                Ok(value)
            }
        }
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        match self.cache.block_hashes.entry(U256::from(number)) {
            Entry::Occupied(entry) => Ok(*entry.get()),
            Entry::Vacant(entry) => {
                let hash = self.db.block_hash_ref(number)?;
                entry.insert(hash);
                Ok(hash)
            }
        }
    }
}

impl<ExtDB: DatabaseRef> DatabaseRef for CacheDB<ExtDB> {
    type Error = ExtDB::Error;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        match self.cache.accounts.get(&address) {
            Some(acc) => Ok(acc.info()),
            None => self.db.basic_ref(address),
        }
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        match self.cache.contracts.get(&code_hash) {
            Some(entry) => Ok(entry.clone()),
            None => self.db.code_by_hash_ref(code_hash),
        }
    }

    fn storage_ref(
        &self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        match self.cache.accounts.get(&address) {
            Some(acc_entry) => match acc_entry.storage.get(&index) {
                Some(entry) => Ok(*entry),
                None => {
                    if matches!(
                        acc_entry.account_state,
                        AccountState::StorageCleared | AccountState::NotExisting
                    ) {
                        Ok(StorageValue::ZERO)
                    } else {
                        self.db.storage_ref(address, index)
                    }
                }
            },
            None => self.db.storage_ref(address, index),
        }
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        match self.cache.block_hashes.get(&U256::from(number)) {
            Some(entry) => Ok(*entry),
            None => self.db.block_hash_ref(number),
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DbAccount {
    pub info: AccountInfo,
    /// If account is selfdestructed or newly created, storage will be cleared.
    pub account_state: AccountState,
    /// Storage slots
    pub storage: HashMap<StorageKey, StorageValue>,
}

impl DbAccount {
    pub fn new_not_existing() -> Self {
        Self {
            account_state: AccountState::NotExisting,
            ..Default::default()
        }
    }

    pub fn info(&self) -> Option<AccountInfo> {
        if matches!(self.account_state, AccountState::NotExisting) {
            None
        } else {
            Some(self.info.clone())
        }
    }
}

impl From<Option<AccountInfo>> for DbAccount {
    fn from(from: Option<AccountInfo>) -> Self {
        from.map(Self::from).unwrap_or_else(Self::new_not_existing)
    }
}

impl From<AccountInfo> for DbAccount {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            account_state: AccountState::None,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AccountState {
    /// Before Spurious Dragon hardfork there was a difference between empty and not existing.
    /// And we are flagging it here.
    NotExisting,
    /// EVM touched this account. For newer hardfork this means it can be cleared/removed from state.
    Touched,
    /// EVM cleared storage of this account, mostly by selfdestruct, we don't ask database for storage slots
    /// and assume they are StorageValue::ZERO
    StorageCleared,
    /// EVM didn't interacted with this account
    #[default]
    None,
}

impl AccountState {
    /// Returns `true` if EVM cleared storage of this account
    pub fn is_storage_cleared(&self) -> bool {
        matches!(self, AccountState::StorageCleared)
    }
}

/// Custom benchmarking DB that only has account info for the zero address.
///
/// Any other address will return an empty account.
#[derive(Debug, Default, Clone)]
pub struct BenchmarkDB(pub Bytecode, B256);

impl BenchmarkDB {
    pub fn new_bytecode(bytecode: Bytecode) -> Self {
        let hash = bytecode.hash_slow();
        Self(bytecode, hash)
    }
}

/// BYTECODE address
pub const FFADDRESS: Address = address!("0xffffffffffffffffffffffffffffffffffffffff");
pub const BENCH_TARGET: Address = FFADDRESS;
pub const BENCH_TARGET_BALANCE: U256 = U256::from_limbs([10_000_000, 0, 0, 0]);
/// CALLER address
pub const EEADDRESS: Address = address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
pub const BENCH_CALLER: Address = EEADDRESS;
pub const BENCH_CALLER_BALANCE: U256 = U256::from_limbs([10_000_000, 0, 0, 0]);

impl Database for BenchmarkDB {
    type Error = Infallible;
    /// Get basic account information.
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        if address == BENCH_TARGET {
            return Ok(Some(AccountInfo {
                nonce: 1,
                balance: BENCH_TARGET_BALANCE,
                code: Some(self.0.clone()),
                code_hash: self.1,
            }));
        }
        if address == BENCH_CALLER {
            return Ok(Some(AccountInfo {
                nonce: 0,
                balance: BENCH_CALLER_BALANCE,
                code: None,
                code_hash: KECCAK_EMPTY,
            }));
        }
        Ok(None)
    }

    /// Get account code by its hash
    fn code_by_hash(&mut self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(Bytecode::default())
    }

    /// Get storage value of address at index.
    fn storage(
        &mut self,
        _address: Address,
        _index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        Ok(StorageValue::default())
    }

    // History related
    fn block_hash(&mut self, _number: u64) -> Result<B256, Self::Error> {
        Ok(B256::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{CacheDB, EmptyDB};
    use database_interface::Database;
    use primitives::{Address, HashMap, StorageKey, StorageValue};
    use state::AccountInfo;

    #[test]
    fn test_insert_account_storage() {
        let account = Address::with_last_byte(42);
        let nonce = 42;
        let mut init_state = CacheDB::new(EmptyDB::default());
        init_state.insert_account_info(
            account,
            AccountInfo {
                nonce,
                ..Default::default()
            },
        );

        let (key, value) = (StorageKey::from(123), StorageValue::from(456));
        let mut new_state = CacheDB::new(init_state);
        new_state
            .insert_account_storage(account, key, value)
            .unwrap();

        assert_eq!(new_state.basic(account).unwrap().unwrap().nonce, nonce);
        assert_eq!(new_state.storage(account, key), Ok(value));
    }

    #[test]
    fn test_replace_account_storage() {
        let account = Address::with_last_byte(42);
        let nonce = 42;
        let mut init_state = CacheDB::new(EmptyDB::default());
        init_state.insert_account_info(
            account,
            AccountInfo {
                nonce,
                ..Default::default()
            },
        );

        let (key0, value0) = (StorageKey::from(123), StorageValue::from(456));
        let (key1, value1) = (StorageKey::from(789), StorageValue::from(999));
        init_state
            .insert_account_storage(account, key0, value0)
            .unwrap();

        let mut new_state = CacheDB::new(init_state);
        new_state
            .replace_account_storage(account, HashMap::from_iter([(key1, value1)]))
            .unwrap();

        assert_eq!(new_state.basic(account).unwrap().unwrap().nonce, nonce);
        assert_eq!(new_state.storage(account, key0), Ok(StorageValue::ZERO));
        assert_eq!(new_state.storage(account, key1), Ok(value1));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize_deserialize_cachedb() {
        let account = Address::with_last_byte(69);
        let nonce = 420;
        let mut init_state = CacheDB::new(EmptyDB::default());
        init_state.insert_account_info(
            account,
            AccountInfo {
                nonce,
                ..Default::default()
            },
        );

        let serialized = serde_json::to_string(&init_state).unwrap();
        let deserialized: CacheDB<EmptyDB> = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.cache.accounts.contains_key(&account));
        assert_eq!(
            deserialized
                .cache
                .accounts
                .get(&account)
                .unwrap()
                .info
                .nonce,
            nonce
        );
    }
}
