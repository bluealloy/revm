use crate::{
    db::{Database, DatabaseCommit, DatabaseRef},
    primitives::{specification, EVMError, EVMResult, Env, ExecutionResult},
    r#impl::{EVMImpl, Transact},
};
use alloc::boxed::Box;
use revm_primitives::{db::WrapDatabaseRef, ResultAndState};

/// Struct that takes Database and enabled transact to update state directly to database.
/// additionally it allows user to set all environment parameters.
///
/// Parameters that can be set are divided between Config, Block and Transaction(tx)
///
/// For transacting on EVM you can call transact_commit that will automatically apply changes to db.
///
/// You can do a lot with rust and traits. For Database abstractions that we need you can implement,
/// Database, DatabaseRef or Database+DatabaseCommit and they enable functionality depending on what
/// kind of handling of struct you want.
/// * Database trait has mutable self in its functions. It is usefully if on get calls you want to
///   modify
/// your cache or update some statistics. They enable `transact` and `inspect` functions
/// * DatabaseRef takes reference on object, this is useful if you only have reference on state and
///   don't
/// want to update anything on it. It enabled `transact_ref` and `inspect_ref` functions
/// * Database+DatabaseCommit allow directly committing changes of transaction. it enabled
///   `transact_commit`
/// and `inspect_commit`
///
/// /// # Example
///
/// ```
/// # use revm_rwasm::RWASM;        // Assuming this struct is in 'your_crate_name'
/// # struct SomeDatabase;  // Mocking a database type for the purpose of this example
/// # struct Env;           // Assuming the type Env is defined somewhere
///
/// let evm: RWASM<SomeDatabase> = RWASM::new();
/// assert!(evm.db.is_none());
/// ```
#[derive(Clone, Debug)]
pub struct RWASM<DB> {
    pub env: Env,
    pub db: Option<DB>,
}

pub fn new<DB>() -> RWASM<DB> {
    RWASM::new()
}

impl<DB> Default for RWASM<DB> {
    fn default() -> Self {
        Self::new()
    }
}

impl<DB: Database + DatabaseCommit> RWASM<DB> {
    /// Execute transaction and apply result to database
    pub fn transact_commit(&mut self) -> Result<ExecutionResult, EVMError<DB::Error>> {
        let ResultAndState { result, state } = self.transact()?;
        self.db.as_mut().unwrap().commit(state);
        Ok(result)
    }
}

impl<DB: Database> RWASM<DB> {
    /// Do checks that could make transaction fail before call/create
    pub fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        if let Some(db) = self.db.as_mut() {
            evm_inner::<DB>(&mut self.env, db).preverify_transaction()
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Skip preverification steps and execute transaction without writing to DB, return change
    /// state.
    pub fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_mut() {
            evm_inner::<DB>(&mut self.env, db).transact_preverified()
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Execute transaction without writing to DB, return change state.
    pub fn transact(&mut self) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_mut() {
            evm_inner::<DB>(&mut self.env, db).transact()
        } else {
            panic!("Database needs to be set");
        }
    }
}

impl<'a, DB: DatabaseRef> RWASM<DB> {
    /// Do checks that could make transaction fail before call/create
    pub fn preverify_transaction_ref(&self) -> Result<(), EVMError<DB::Error>> {
        if let Some(db) = self.db.as_ref() {
            evm_inner::<_>(&mut self.env.clone(), &mut WrapDatabaseRef(db)).preverify_transaction()
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Skip preverification steps and execute transaction
    /// without writing to DB, return change state.
    pub fn transact_preverified_ref(&self) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_ref() {
            evm_inner::<_>(&mut self.env.clone(), &mut WrapDatabaseRef(db)).transact_preverified()
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Execute transaction without writing to DB, return change state.
    pub fn transact_ref(&self) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_ref() {
            evm_inner::<_>(&mut self.env.clone(), &mut WrapDatabaseRef(db)).transact()
        } else {
            panic!("Database needs to be set");
        }
    }
}

impl<DB> RWASM<DB> {
    /// Creates a new [RWASM] instance with the default environment,
    pub fn new() -> Self {
        Self::with_env(Default::default())
    }

    /// Creates a new [RWASM] instance with the given environment.
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

pub fn evm_inner<'a, DB: Database>(
    env: &'a mut Env,
    db: &'a mut DB,
) -> Box<dyn Transact<DB::Error> + 'a> {
    macro_rules! create_evm {
        ($spec:ident) => {
            Box::new(EVMImpl::<'a, $spec, DB>::new(db, env)) as Box<dyn Transact<DB::Error> + 'a>
        };
    }

    use specification::*;
    match env.cfg.spec_id {
        FRONTIER | FRONTIER_THAWING => create_evm!(FrontierSpec),
        HOMESTEAD | DAO_FORK => create_evm!(HomesteadSpec),
        TANGERINE => create_evm!(TangerineSpec),
        SPURIOUS_DRAGON => create_evm!(SpuriousDragonSpec),
        BYZANTIUM => create_evm!(ByzantiumSpec),
        PETERSBURG | CONSTANTINOPLE => create_evm!(PetersburgSpec),
        ISTANBUL | MUIR_GLACIER => create_evm!(IstanbulSpec),
        BERLIN => create_evm!(BerlinSpec),
        LONDON | ARROW_GLACIER | GRAY_GLACIER => {
            create_evm!(LondonSpec)
        }
        MERGE => create_evm!(MergeSpec),
        SHANGHAI => create_evm!(ShanghaiSpec),
        CANCUN => create_evm!(CancunSpec),
        LATEST => create_evm!(LatestSpec),
        _ => unreachable!("not supported spec id"),
    }
}
