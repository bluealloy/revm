use crate::{
    db::{Database, DatabaseCommit, DatabaseRef},
    evm::{new_evm, Transact},
    handler::{InspectorHandle, MainnetHandle, RegisterHandler},
    primitives::{db::WrapDatabaseRef, EVMError, EVMResult, Env, ExecutionResult, ResultAndState},
    Context, Evm, EvmContext, Inspector,
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
pub struct EvmFactory<DB> {
    pub env: Box<Env>,
    pub db: Option<DB>,
}

pub fn new<DB>() -> EvmFactory<DB> {
    EvmFactory::new()
}

impl<DB> Default for EvmFactory<DB> {
    fn default() -> Self {
        Self::new()
    }
}

impl<DB: Database + DatabaseCommit> EvmFactory<DB> {
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

impl<DB: Database> EvmFactory<DB> {
    pub fn execute_evm<
        'a,
        OUT,
        EXT: RegisterHandler<'a, DB, EXT> + 'a,
        FN: Fn(&mut Evm<'a, EXT, DB>) -> OUT,
    >(
        &mut self,
        external: EXT,
        exec: FN,
    ) -> OUT
    where
        DB: 'a,
    {
        let Some(db) = self.db.take() else {
            panic!("Database needs to be set");
        };
        let env = core::mem::take(&mut self.env);
        let mut evm = new_evm::<EXT, DB>(env, db, external);
        let res = exec(&mut evm);

        let Context {
            evm: EvmContext { db, env, .. },
            ..
        } = evm.into_context();
        self.env = env;
        self.db = Some(db);

        res
    }
    /// Do checks that could make transaction fail before call/create
    pub fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        self.execute_evm(MainnetHandle::default(), |evm| evm.preverify_transaction())
    }

    /// Skip preverification steps and execute transaction without writing to DB, return change
    /// state.
    pub fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        self.execute_evm(MainnetHandle::default(), |evm| evm.transact_preverified())
    }

    /// Execute transaction without writing to DB, return change state.
    pub fn transact(&mut self) -> EVMResult<DB::Error> {
        self.execute_evm(MainnetHandle::default(), |evm| evm.transact())
    }

    /// Execute transaction with given inspector, without wring to DB. Return change state.
    pub fn inspect<INSP: Inspector<DB>>(&mut self, inspector: INSP) -> EVMResult<DB::Error> {
        let insp = InspectorHandle::new(inspector);
        self.execute_evm(insp, |evm| evm.transact())
    }
}

impl<DB: DatabaseRef> EvmFactory<DB> {
    pub fn execute_evm_ref<
        'a,
        OUT: 'a,
        EXT: RegisterHandler<'a, WrapDatabaseRef<&'a DB>, EXT> + 'a,
        FN: Fn(&mut Evm<'a, EXT, WrapDatabaseRef<&'a DB>>) -> OUT,
    >(
        &'a self,
        external: EXT,
        exec: FN,
    ) -> OUT
    where
        DB: 'a,
    {
        //unimplemented!();
        let Some(db) = self.db.as_ref() else {
            panic!("Database needs to be set");
        };
        let env = self.env.clone();
        let mut evm = new_evm::<EXT, WrapDatabaseRef<&DB>>(env, WrapDatabaseRef(db), external);
        let res = exec(&mut evm);

        res
    }
    /// Do checks that could make transaction fail before call/create
    pub fn preverify_transaction_ref(&self) -> Result<(), EVMError<DB::Error>> {
        self.execute_evm_ref(MainnetHandle::default(), |evm| evm.preverify_transaction())
    }

    /// Skip preverification steps and execute transaction
    /// without writing to DB, return change state.
    pub fn transact_preverified_ref(&self) -> EVMResult<DB::Error> {
        self.execute_evm_ref(MainnetHandle::default(), |evm| evm.transact_preverified())
    }

    /// Execute transaction without writing to DB, return change state.
    pub fn transact_ref(&self) -> EVMResult<DB::Error> {
        self.execute_evm_ref(MainnetHandle::default(), |evm| evm.transact())
    }

    /// Execute transaction with given inspector, without wring to DB. Return change state.
    pub fn inspect_ref<'a, I: Inspector<WrapDatabaseRef<&'a DB>> + 'a>(
        &'a self,
        inspector: I,
    ) -> EVMResult<DB::Error> {
        let insp = InspectorHandle::new(inspector);
        self.execute_evm_ref(insp, |evm| evm.transact())
    }
}

impl<DB> EvmFactory<DB> {
    /// Creates a new [EVM] instance with the default environment,
    pub fn new() -> Self {
        Self::with_env(Default::default())
    }

    /// Creates a new [EVM] instance with the given environment.
    pub fn with_env(env: Box<Env>) -> Self {
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
