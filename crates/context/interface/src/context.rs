pub use crate::journaled_state::StateLoad;
use crate::{Block, Cfg, Database, JournalTr, Transaction};
use auto_impl::auto_impl;
use database_interface::DBErrorMarker;
use primitives::U256;

#[auto_impl(&mut, Box)]
pub trait ContextTr {
    type Block: Block;
    type Tx: Transaction;
    type Cfg: Cfg;
    type Db: Database;
    type Journal: JournalTr<Database = Self::Db>;
    type Chain;

    fn tx(&self) -> &Self::Tx;
    fn block(&self) -> &Self::Block;
    fn cfg(&self) -> &Self::Cfg;
    fn journal(&mut self) -> &mut Self::Journal;
    fn journal_ref(&self) -> &Self::Journal;
    fn db(&mut self) -> &mut Self::Db;
    fn db_ref(&self) -> &Self::Db;
    fn chain(&mut self) -> &mut Self::Chain;
    fn error(&mut self) -> &mut Result<(), ContextError<<Self::Db as Database>::Error>>;
    fn tx_journal(&mut self) -> (&mut Self::Tx, &mut Self::Journal);
}

/// Inner Context error used for Interpreter to set error without returning it frm instruction
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContextError<DbError: DBErrorMarker> {
    /// Database error.
    Db(DbError),
    /// Custom string error.
    Custom(String),
}

impl<DbError: DBErrorMarker> From<DbError> for ContextError<DbError> {
    fn from(value: DbError) -> Self {
        Self::Db(value)
    }
}

/// Represents the result of an `sstore` operation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SStoreResult {
    /// Value of the storage when it is first read
    pub original_value: U256,
    /// Current value of the storage
    pub present_value: U256,
    /// New value that is set
    pub new_value: U256,
}

impl SStoreResult {
    /// Returns `true` if the new value is equal to the present value.
    #[inline]
    pub fn is_new_eq_present(&self) -> bool {
        self.new_value == self.present_value
    }

    /// Returns `true` if the original value is equal to the present value.
    #[inline]
    pub fn is_original_eq_present(&self) -> bool {
        self.original_value == self.present_value
    }

    /// Returns `true` if the original value is equal to the new value.
    #[inline]
    pub fn is_original_eq_new(&self) -> bool {
        self.original_value == self.new_value
    }

    /// Returns `true` if the original value is zero.
    #[inline]
    pub fn is_original_zero(&self) -> bool {
        self.original_value.is_zero()
    }

    /// Returns `true` if the present value is zero.
    #[inline]
    pub fn is_present_zero(&self) -> bool {
        self.present_value.is_zero()
    }

    /// Returns `true` if the new value is zero.
    #[inline]
    pub fn is_new_zero(&self) -> bool {
        self.new_value.is_zero()
    }
}

/// Result of a selfdestruct action
///
/// Value returned are needed to calculate the gas spent.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfDestructResult {
    pub had_value: bool,
    pub target_exists: bool,
    pub previously_destroyed: bool,
}

pub trait ContextSetters: ContextTr {
    fn set_tx(&mut self, tx: Self::Tx);
    fn set_block(&mut self, block: Self::Block);
}
