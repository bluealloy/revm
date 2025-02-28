use crate::DatabaseCommit;
use core::{convert::Infallible, error::Error, fmt};
use primitives::{Address, HashMap};
use state::Account;
use std::sync::Arc;

/// EVM database commit interface that can fail.
///
/// This is intended for use with types that may fail to commit changes, e.g.
/// because they are directly interacting with the filesystem, or must arrange
/// access to a shared resource.
pub trait TryDatabaseCommit {
    /// Error type for when [`TryDatabaseCommit::try_commit`] fails.
    type Error: Error;

    /// Attempt to commit changes to the database.
    fn try_commit(&mut self, changes: HashMap<Address, Account>) -> Result<(), Self::Error>;
}

impl<Db> TryDatabaseCommit for Db
where
    Db: DatabaseCommit,
{
    type Error = Infallible;

    #[inline]
    fn try_commit(&mut self, changes: HashMap<Address, Account>) -> Result<(), Self::Error> {
        self.commit(changes);
        Ok(())
    }
}

/// Error type for implementation of [`TryDatabaseCommit`] on
/// [`Arc`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArcUpgradeError;

impl fmt::Display for ArcUpgradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Arc reference is not unique, cannot mutate")
    }
}

impl Error for ArcUpgradeError {}

impl<Db> TryDatabaseCommit for Arc<Db>
where
    Db: DatabaseCommit + Send + Sync,
{
    type Error = ArcUpgradeError;

    #[inline]
    fn try_commit(&mut self, changes: HashMap<Address, Account>) -> Result<(), Self::Error> {
        Arc::get_mut(self)
            .map(|db| db.commit(changes))
            .ok_or(ArcUpgradeError)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::DatabaseCommit;
    use std::sync::Arc;

    struct MockDb;

    impl DatabaseCommit for MockDb {
        fn commit(&mut self, _changes: HashMap<Address, Account>) {}
    }

    #[test]
    fn arc_try_commit() {
        let mut db = Arc::new(MockDb);
        let db_2 = Arc::clone(&db);

        assert_eq!(
            db.try_commit(Default::default()).unwrap_err(),
            ArcUpgradeError
        );
        drop(db_2);
        db.try_commit(Default::default()).unwrap();
    }
}
