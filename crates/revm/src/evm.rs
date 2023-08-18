use crate::primitives::{specification, EVMError, EVMResult, Env, ExecutionResult, SpecId};
use crate::{
    db::{Database, DatabaseCommit, DatabaseRef, RefDBWrapper},
    evm_impl::{EVMImpl, Transact},
    inspectors::NoOpInspector,
    Inspector,
};
use alloc::boxed::Box;
use revm_interpreter::primitives::ResultAndState;
use revm_precompile::Precompiles;

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
#[derive(Clone)]
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
    /// Execute transaction without writing to DB, return change state.
    pub fn transact(&mut self) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_mut() {
            let mut noop = NoOpInspector {};
            let out = evm_inner::<DB, false>(&mut self.env, db, &mut noop).transact();
            out
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Execute transaction with given inspector, without wring to DB. Return change state.
    pub fn inspect<INSP: Inspector<DB>>(&mut self, mut inspector: INSP) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_mut() {
            evm_inner::<DB, true>(&mut self.env, db, &mut inspector).transact()
        } else {
            panic!("Database needs to be set");
        }
    }
}

impl<'a, DB: DatabaseRef> EVM<DB> {
    /// Execute transaction without writing to DB, return change state.
    pub fn transact_ref(&self) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_ref() {
            let mut noop = NoOpInspector {};
            let mut db = RefDBWrapper::new(db);
            let db = &mut db;
            let out =
                evm_inner::<RefDBWrapper<DB::Error>, false>(&mut self.env.clone(), db, &mut noop)
                    .transact();
            out
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Execute transaction with given inspector, without wring to DB. Return change state.
    pub fn inspect_ref<INSP: Inspector<RefDBWrapper<'a, DB::Error>>>(
        &'a self,
        mut inspector: INSP,
    ) -> EVMResult<DB::Error> {
        if let Some(db) = self.db.as_ref() {
            let mut db = RefDBWrapper::new(db);
            let db = &mut db;
            let out = evm_inner::<RefDBWrapper<DB::Error>, true>(
                &mut self.env.clone(),
                db,
                &mut inspector,
            )
            .transact();
            out
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

macro_rules! create_evm {
    ($spec:ident, $db:ident, $env:ident, $inspector:ident) => {
        Box::new(EVMImpl::<'a, $spec, DB, INSPECT>::new(
            $db,
            $env,
            $inspector,
            Precompiles::new(to_precompile_id($spec::SPEC_ID)).clone(),
        )) as Box<dyn Transact<DB::Error> + 'a>
    };
}

pub fn to_precompile_id(spec_id: SpecId) -> revm_precompile::SpecId {
    match spec_id {
        SpecId::FRONTIER
        | SpecId::FRONTIER_THAWING
        | SpecId::HOMESTEAD
        | SpecId::DAO_FORK
        | SpecId::TANGERINE
        | SpecId::SPURIOUS_DRAGON => revm_precompile::SpecId::HOMESTEAD,
        SpecId::BYZANTIUM | SpecId::CONSTANTINOPLE | SpecId::PETERSBURG => {
            revm_precompile::SpecId::BYZANTIUM
        }
        SpecId::ISTANBUL | SpecId::MUIR_GLACIER => revm_precompile::SpecId::ISTANBUL,
        SpecId::BERLIN
        | SpecId::LONDON
        | SpecId::ARROW_GLACIER
        | SpecId::GRAY_GLACIER
        | SpecId::MERGE
        | SpecId::SHANGHAI
        | SpecId::CANCUN
        | SpecId::LATEST => revm_precompile::SpecId::BERLIN,
    }
}

pub fn evm_inner<'a, DB: Database, const INSPECT: bool>(
    env: &'a mut Env,
    db: &'a mut DB,
    insp: &'a mut dyn Inspector<DB>,
) -> Box<dyn Transact<DB::Error> + 'a> {
    use specification::*;
    match env.cfg.spec_id {
        SpecId::FRONTIER | SpecId::FRONTIER_THAWING => create_evm!(FrontierSpec, db, env, insp),
        SpecId::HOMESTEAD | SpecId::DAO_FORK => create_evm!(HomesteadSpec, db, env, insp),
        SpecId::TANGERINE => create_evm!(TangerineSpec, db, env, insp),
        SpecId::SPURIOUS_DRAGON => create_evm!(SpuriousDragonSpec, db, env, insp),
        SpecId::BYZANTIUM => create_evm!(ByzantiumSpec, db, env, insp),
        SpecId::PETERSBURG | SpecId::CONSTANTINOPLE => create_evm!(PetersburgSpec, db, env, insp),
        SpecId::ISTANBUL | SpecId::MUIR_GLACIER => create_evm!(IstanbulSpec, db, env, insp),
        SpecId::BERLIN => create_evm!(BerlinSpec, db, env, insp),
        SpecId::LONDON | SpecId::ARROW_GLACIER | SpecId::GRAY_GLACIER => {
            create_evm!(LondonSpec, db, env, insp)
        }
        SpecId::MERGE => create_evm!(MergeSpec, db, env, insp),
        SpecId::SHANGHAI => create_evm!(ShanghaiSpec, db, env, insp),
        SpecId::CANCUN => create_evm!(CancunSpec, db, env, insp),
        SpecId::LATEST => create_evm!(LatestSpec, db, env, insp),
    }
}
