use bytecode::Bytecode;
use core::hash::{Hash, Hasher};
use primitives::{B256, KECCAK_EMPTY, U256};

/// Account information that contains balance, nonce, code hash and code
///
/// Code is set as optional.
#[derive(Clone, Debug, Eq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountInfo {
    /// Account balance.
    pub balance: U256,
    /// Account nonce.
    pub nonce: u64,
    /// Hash of the raw bytes in `code`, or [`KECCAK_EMPTY`].
    pub code_hash: B256,
    /// [`Bytecode`] data associated with this account.
    ///
    /// If [`None`], `code_hash` will be used to fetch it from the database, if code needs to be
    /// loaded from inside `revm`.
    ///
    /// By default, this is `Some(Bytecode::default())`.
    pub code: Option<Bytecode>,
}

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            balance: U256::ZERO,
            code_hash: KECCAK_EMPTY,
            code: Some(Bytecode::default()),
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

impl Hash for AccountInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.balance.hash(state);
        self.nonce.hash(state);
        self.code_hash.hash(state);
    }
}

impl AccountInfo {
    /// Creates a new [`AccountInfo`] with the given fields.
    #[inline]
    pub fn new(balance: U256, nonce: u64, code_hash: B256, code: Bytecode) -> Self {
        Self {
            balance,
            nonce,
            code: Some(code),
            code_hash,
        }
    }

    /// Creates a new [`AccountInfo`] with the given code.
    ///
    /// # Note
    ///
    /// As code hash is calculated with [`Bytecode::hash_slow`] there will be performance penalty if used frequently.
    pub fn with_code(self, code: Bytecode) -> Self {
        Self {
            balance: self.balance,
            nonce: self.nonce,
            code_hash: code.hash_slow(),
            code: Some(code),
        }
    }

    /// Creates a new [`AccountInfo`] with the given code hash.
    ///
    /// # Note
    ///
    /// Resets code to `None`. Not guaranteed to maintain invariant `code` and `code_hash`. See
    /// also [Self::with_code_and_hash].
    pub fn with_code_hash(self, code_hash: B256) -> Self {
        Self {
            balance: self.balance,
            nonce: self.nonce,
            code_hash,
            code: None,
        }
    }

    /// Creates a new [`AccountInfo`] with the given code and code hash.
    ///
    /// # Note
    ///
    /// In debug mode panics if [`Bytecode::hash_slow`] called on `code` is not equivalent to
    /// `code_hash`. See also [`Self::with_code`].
    pub fn with_code_and_hash(self, code: Bytecode, code_hash: B256) -> Self {
        debug_assert_eq!(code.hash_slow(), code_hash);
        Self {
            balance: self.balance,
            nonce: self.nonce,
            code_hash,
            code: Some(code),
        }
    }

    /// Creates a new [`AccountInfo`] with the given balance.
    pub fn with_balance(mut self, balance: U256) -> Self {
        self.balance = balance;
        self
    }

    /// Creates a new [`AccountInfo`] with the given nonce.
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    /// Sets the [`AccountInfo`] `balance`.
    #[inline]
    pub fn set_balance(&mut self, balance: U256) -> &mut Self {
        self.balance = balance;
        self
    }

    /// Sets the [`AccountInfo`] `nonce`.
    #[inline]
    pub fn set_nonce(&mut self, nonce: u64) -> &mut Self {
        self.nonce = nonce;
        self
    }

    /// Sets the [`AccountInfo`] `code_hash` and clears any cached bytecode.
    ///
    /// # Note
    ///
    /// Calling this after `set_code(...)` will remove the bytecode you just set.
    /// If you intend to mutate the code, use only `set_code`.
    #[inline]
    pub fn set_code_hash(&mut self, code_hash: B256) -> &mut Self {
        self.code = None;
        self.code_hash = code_hash;
        self
    }

    /// Replaces the [`AccountInfo`] bytecode and recalculates `code_hash`.
    ///
    /// # Note
    ///
    /// As code hash is calculated with [`Bytecode::hash_slow`] there will be performance penalty if used frequently.
    #[inline]
    pub fn set_code(&mut self, code: Bytecode) -> &mut Self {
        self.code_hash = code.hash_slow();
        self.code = Some(code);
        self
    }
    /// Sets the bytecode and its hash.
    ///
    /// # Note
    ///
    /// It is on the caller's responsibility to ensure that the bytecode hash is correct.
    pub fn set_code_and_hash(&mut self, code: Bytecode, code_hash: B256) {
        self.code_hash = code_hash;
        self.code = Some(code);
    }
    /// Returns a copy of this account with the [`Bytecode`] removed.
    ///
    /// This is useful when creating journals or snapshots of the state, where it is
    /// desirable to store the code blobs elsewhere.
    ///
    /// ## Note
    ///
    /// This is distinct from [`without_code`][Self::without_code] in that it returns
    /// a new [`AccountInfo`] instance with the code removed.
    ///
    /// [`without_code`][Self::without_code] will modify and return the same instance.
    #[inline]
    pub fn copy_without_code(&self) -> Self {
        Self {
            balance: self.balance,
            nonce: self.nonce,
            code_hash: self.code_hash,
            code: None,
        }
    }

    /// Strips the [`Bytecode`] from this account and drop it.
    ///
    /// This is useful when creating journals or snapshots of the state, where it is
    /// desirable to store the code blobs elsewhere.
    ///
    /// ## Note
    ///
    /// This is distinct from [`copy_without_code`][Self::copy_without_code] in that it
    /// modifies the account in place.
    ///
    /// [`copy_without_code`][Self::copy_without_code]
    /// will copy the non-code fields and return a new [`AccountInfo`] instance.
    pub fn without_code(mut self) -> Self {
        self.take_bytecode();
        self
    }

    /// Returns if an account is empty.
    ///
    /// An account is empty if the following conditions are met.
    /// - code hash is zero or set to the Keccak256 hash of the empty string `""`
    /// - balance is zero
    /// - nonce is zero
    #[inline]
    pub fn is_empty(&self) -> bool {
        let code_empty = self.is_empty_code_hash() || self.code_hash.is_zero();
        code_empty && self.balance.is_zero() && self.nonce == 0
    }

    /// Returns `true` if the account is not empty.
    #[inline]
    pub fn exists(&self) -> bool {
        !self.is_empty()
    }

    /// Returns `true` if account has no nonce and code.
    #[inline]
    pub fn has_no_code_and_nonce(&self) -> bool {
        self.is_empty_code_hash() && self.nonce == 0
    }

    /// Returns bytecode hash associated with this account.
    ///
    /// If account does not have code, it returns `KECCAK_EMPTY` hash.
    #[inline]
    pub fn code_hash(&self) -> B256 {
        self.code_hash
    }

    /// Returns true if the code hash is the Keccak256 hash of the empty string `""`.
    #[inline]
    pub fn is_empty_code_hash(&self) -> bool {
        self.code_hash == KECCAK_EMPTY
    }

    /// Takes bytecode from account.
    ///
    /// Code will be set to [None].
    #[inline]
    pub fn take_bytecode(&mut self) -> Option<Bytecode> {
        self.code.take()
    }

    /// Initializes an [`AccountInfo`] with the given balance, setting all other fields to their
    /// default values.
    #[inline]
    pub fn from_balance(balance: U256) -> Self {
        AccountInfo {
            balance,
            ..Default::default()
        }
    }

    /// Initializes an [`AccountInfo`] with the given bytecode, setting its balance to zero, its
    /// nonce to `1`, and calculating the code hash from the given bytecode.
    #[inline]
    pub fn from_bytecode(bytecode: Bytecode) -> Self {
        let hash = bytecode.hash_slow();

        AccountInfo {
            balance: U256::ZERO,
            nonce: 1,
            code: Some(bytecode),
            code_hash: hash,
        }
    }
}
