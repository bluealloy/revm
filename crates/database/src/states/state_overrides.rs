use bytecode::Bytecode;
use primitives::{
    Address, AddressMap, B256Map, StorageKey, StorageKeyMap, StorageValue, KECCAK_EMPTY,
};
use state::{Account, AccountInfo};

/// Controls how an account's storage overrides are applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StorageOverrideMode {
    /// Override only the supplied slots and fall through for all other slots.
    #[default]
    Diff,
    /// Replace the account's storage, making every unspecified slot zero.
    Replace,
}

/// State values that take precedence over an attached block access list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateOverrides {
    accounts: AddressMap<Option<AccountInfo>>,
    storage: AddressMap<StorageOverrides>,
    contracts: B256Map<Bytecode>,
}

impl StateOverrides {
    /// Extends the overrides with account and nested per-account storage values.
    pub fn extend(
        &mut self,
        accounts: impl IntoIterator<Item = (Address, Option<AccountInfo>)>,
        storage: impl IntoIterator<
            Item = (
                Address,
                StorageOverrideMode,
                impl IntoIterator<Item = (StorageKey, StorageValue)>,
            ),
        >,
    ) {
        for (address, account) in accounts {
            self.insert_account(address, account);
        }
        for (address, mode, slots) in storage {
            let storage = self.storage.entry(address).or_default();
            if mode == StorageOverrideMode::Replace {
                storage.slots.clear();
                storage.wiped = true;
            }
            storage.slots.extend(slots);
        }
    }

    /// Returns an account override, including an explicit absent account.
    #[inline]
    pub fn account(&self, address: &Address) -> Option<&Option<AccountInfo>> {
        self.accounts.get(address)
    }

    /// Returns an overridden bytecode by hash.
    #[inline]
    pub fn code_by_hash(&self, code_hash: &primitives::B256) -> Option<&Bytecode> {
        self.contracts.get(code_hash)
    }

    /// Returns a storage override, including zero for an unspecified slot after replacement.
    #[inline]
    pub fn storage(&self, address: &Address, key: StorageKey) -> Option<StorageValue> {
        let storage = self.storage.get(address)?;
        storage
            .slots
            .get(&key)
            .copied()
            .or(storage.wiped.then_some(StorageValue::ZERO))
    }

    /// Applies committed execution state above the current overrides.
    pub(crate) fn commit(&mut self, changes: &AddressMap<Account>) {
        for (address, account) in changes {
            if !account.is_touched() {
                continue;
            }

            if account.is_selfdestructed() || (!account.is_created() && account.is_empty()) {
                self.insert_account(*address, None);
                self.storage.entry(*address).or_default().wipe();
                continue;
            }

            self.insert_account(*address, Some(account.info.clone()));
            let storage = self.storage.entry(*address).or_default();
            if account.is_created() {
                storage.wipe();
            }
            storage.slots.extend(
                account
                    .changed_storage_slots()
                    .map(|(key, slot)| (*key, slot.present_value)),
            );
        }
    }

    fn insert_account(&mut self, address: Address, mut account: Option<AccountInfo>) {
        if let Some(info) = account.as_mut() {
            if let Some(code) = info.code.as_ref() {
                if !code.is_empty() {
                    if info.code_hash == KECCAK_EMPTY {
                        info.code_hash = code.hash_slow();
                    }
                    self.contracts
                        .entry(info.code_hash)
                        .or_insert_with(|| code.clone());
                }
            }
            if info.code_hash.is_zero() {
                info.code_hash = KECCAK_EMPTY;
            }
        }
        if account.is_none() {
            self.storage.entry(address).or_default().wipe();
        }
        self.accounts.insert(address, account);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct StorageOverrides {
    slots: StorageKeyMap<StorageValue>,
    wiped: bool,
}

impl StorageOverrides {
    fn wipe(&mut self) {
        self.slots.clear();
        self.wiped = true;
    }
}
