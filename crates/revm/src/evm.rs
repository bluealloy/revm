use crate::{
    db::{Database, DatabaseCommit, DatabaseRef},
    evm_impl::{new_evm, Transact},
    primitives::{db::WrapDatabaseRef, EVMError, EVMResult, Env, ExecutionResult, ResultAndState},
    Inspector,
};

/// Struct that takes Database and enabled transact to update state directly to database.
/// additionally it allows user to set all environment parameters.
///
/// Parameters that can be set are divided between Config, Block and Transaction(tx)
///
/// For transacting on EVM you can call transact_commit that will automatically apply changes to db.
///
/// You can do a lot with rust and traits. For Database abstractions that we need you can implement,
/// Database, DatabaseRef or Database+DatabaseCommit and they enable functionality depending on what kind of
/// handling of struct you want.
/// * Database trait has mutable self in its functions. It is usefully if on get calls you want to modify
/// your cache or update some statistics. They enable `transact` and `inspect` functions
/// * DatabaseRef takes reference on object, this is useful if you only have reference on state and don't
/// want to update anything on it. It enabled `transact_ref` and `inspect_ref` functions
/// * Database+DatabaseCommit allow directly committing changes of transaction. it enabled `transact_commit`
/// and `inspect_commit`
///
/// /// # Example
///
/// ```
/// # use revm::EVM;        // Assuming this struct is in 'your_crate_name'
/// # struct SomeDatabase;  // Mocking a database type for the purpose of this example
/// # struct Env;           // Assuming the type Env is defined somewhere
///
/// let evm: EVM<SomeDatabase> = EVM::new();
/// assert!(evm.db.is_none());
/// ```
///
#[derive(Clone, Debug)]
pub struct EVM<DB> {
    pub env: Env,
    pub db: Option<DB>,
}

pub fn new<DB>() -> EVM<DB> {
    EVM::new()
}

impl<DB> Default for EVM<DB> {
    fn default() -> Self {
        Self::new()
    }
}

impl<DB: Database + DatabaseCommit> EVM<DB> {
    /// Execute transaction and apply result to database
    pub fn transact_commit(&mut self) -> Result<ExecutionResult, EVMError<DB::Error>> {
        let ResultAndState { result, state } = self.transact()?;
        self.db.as_mut().unwrap().commit(state);
        Ok(result)
    }

    /// Inspect transaction and commit changes to database.
    pub fn inspect_commit<INSP: Inspector<DB>>(
        &mut self,
        inspector: INSP,
    ) -> Result<ExecutionResult, EVMError<DB::Error>> {
        let ResultAndState { result, state } = self.inspect(inspector)?;
        self.db.as_mut().unwrap().commit(state);
        Ok(result)
    }
}

impl<DB: Database> EVM<DB> {
    /// Do checks that could make transaction fail before call/create
    pub fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        if let Some(db) = self.db.as_mut() {
            new_evm::<DB>(&mut self.env, db, None).preverify_transaction()
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Skip preverification steps and execute transaction without writing to DB, return change
    /// state.
    pub fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_mut() {
            new_evm::<DB>(&mut self.env, db, None).transact_preverified()
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Execute transaction without writing to DB, return change state.
    pub fn transact(&mut self) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_mut() {
            new_evm::<DB>(&mut self.env, db, None).transact()
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Execute transaction with given inspector, without wring to DB. Return change state.
    pub fn inspect<INSP: Inspector<DB>>(&mut self, mut inspector: INSP) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_mut() {
            new_evm::<DB>(&mut self.env, db, Some(&mut inspector)).transact()
        } else {
            panic!("Database needs to be set");
        }
    }
}

impl<'a, DB: DatabaseRef> EVM<DB> {
    /// Do checks that could make transaction fail before call/create
    pub fn preverify_transaction_ref(&self) -> Result<(), EVMError<DB::Error>> {
        if let Some(db) = self.db.as_ref() {
            new_evm::<_>(&mut self.env.clone(), &mut WrapDatabaseRef(db), None)
                .preverify_transaction()
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Skip preverification steps and execute transaction
    /// without writing to DB, return change state.
    pub fn transact_preverified_ref(&self) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_ref() {
            new_evm::<_>(&mut self.env.clone(), &mut WrapDatabaseRef(db), None)
                .transact_preverified()
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Execute transaction without writing to DB, return change state.
    pub fn transact_ref(&self) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_ref() {
            new_evm::<_>(&mut self.env.clone(), &mut WrapDatabaseRef(db), None).transact()
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Execute transaction with given inspector, without wring to DB. Return change state.
    pub fn inspect_ref<I: Inspector<WrapDatabaseRef<&'a DB>>>(
        &'a self,
        mut inspector: I,
    ) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_ref() {
            new_evm(
                &mut self.env.clone(),
                &mut WrapDatabaseRef(db),
                Some(&mut inspector),
            )
            .transact()
        } else {
            panic!("Database needs to be set");
        }
    }
}

impl<DB> EVM<DB> {
    /// Creates a new [EVM] instance with the default environment,
    pub fn new() -> Self {
        Self::with_env(Default::default())
    }

    /// Creates a new [EVM] instance with the given environment.
    pub fn with_env(env: Env) -> Self {
        Self { env, db: None }
    }

    pub fn database(&mut self, db: DB) {
        self.db = Some(db);
    }

    pub fn db(&mut self) -> Option<&mut DB> {
        self.db.as_mut()
    }

    pub fn take_db(&mut self) -> DB {
        core::mem::take(&mut self.db).unwrap()
    }
}
