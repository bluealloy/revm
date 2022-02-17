use crate::{
    db::{Database, DatabaseCommit, DatabaseRef, RefDBWrapper},
    evm_impl::{EVMImpl, Transact},
    subroutine::State,
    BerlinSpec, ByzantineSpec, Env, Inspector, IstanbulSpec, LatestSpec, Log, LondonSpec,
    NoOpInspector, Return, Spec, SpecId, TransactOut,
};
use alloc::{boxed::Box, vec::Vec};
use revm_precompiles::Precompiles;
/// Struct that takes Database and enabled transact to update state dirrectly to database.
/// additionaly it allows user to set all environment parameters.
///
/// Parameters that can be set are devided between Config, Block and Transaction(tx)
///
/// For transacting on EVM you can call transact_commit that will automatically apply changes to db.
///
/// You can do a lot with rust and traits. For Database abstractions that we need you can implement,
/// Database, DatabaseRef or Database+DatabaseCommit and they enable functionality depending on what kind of
/// handling of struct you want.
/// * Database trait has mutable self in its functions. It is usefull if on get calls you want to modify
/// your cache or update some statistics. They enable `transact` and `inspect` functions
/// * DatabaseRef takes reference on object, this is useful if you only have reference on state and dont
/// want to update anything on it. It enabled `transact_ref` and `inspect_ref` functions
/// * Database+DatabaseCommit allow's dirrectly commiting changes of transaction. it enabled `transact_commit`
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
    pub fn transact_commit(&mut self) -> (Return, TransactOut, u64, Vec<Log>) {
        let (exit, out, gas, state, logs) = self.transact();
        self.db.as_mut().unwrap().commit(state);
        (exit, out, gas, logs)
    }
    /// Inspect transaction and commit changes to database.
    pub fn inspect_commit<INSP: Inspector<DB>>(
        &mut self,
        inspector: INSP,
    ) -> (Return, TransactOut, u64, Vec<Log>) {
        let (exit, out, gas, state, logs) = self.inspect(inspector);
        self.db.as_mut().unwrap().commit(state);
        (exit, out, gas, logs)
    }
}

impl<DB: Database> EVM<DB> {
    /// Execute transaction without writing to DB, return change state.
    pub fn transact(&mut self) -> (Return, TransactOut, u64, State, Vec<Log>) {
        if let Some(db) = self.db.as_mut() {
            let mut noop = NoOpInspector {};
            let out = evm_inner::<DB, false>(&mut self.env, db, &mut noop).transact();
            out
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Execute transaction with given inspector, without wring to DB. Return change state.
    pub fn inspect<INSP: Inspector<DB>>(
        &mut self,
        mut inspector: INSP,
    ) -> (Return, TransactOut, u64, State, Vec<Log>) {
        if let Some(db) = self.db.as_mut() {
            evm_inner::<DB, true>(&mut self.env, db, &mut inspector).transact()
        } else {
            panic!("Database needs to be set");
        }
    }
}

impl<'a, DB: DatabaseRef> EVM<DB> {
    /// Execute transaction without writing to DB, return change state.
    pub fn transact_ref(&mut self) -> (Return, TransactOut, u64, State, Vec<Log>) {
        if let Some(db) = self.db.as_ref() {
            let mut noop = NoOpInspector {};
            let mut db = RefDBWrapper::new(db);
            let db = &mut db;
            let out = evm_inner::<RefDBWrapper, false>(&mut self.env, db, &mut noop).transact();
            out
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Execute transaction with given inspector, without wring to DB. Return change state.
    pub fn inspect_ref<INSP: Inspector<RefDBWrapper<'a>>>(
        &'a mut self,
        mut inspector: INSP,
    ) -> (Return, TransactOut, u64, State, Vec<Log>) {
        if let Some(db) = self.db.as_ref() {
            let mut db = RefDBWrapper::new(db);
            let db = &mut db;
            let out = evm_inner::<RefDBWrapper, true>(&mut self.env, db, &mut inspector).transact();
            out
        } else {
            panic!("Database needs to be set");
        }
    }
}

impl<DB> EVM<DB> {
    pub fn new() -> Self {
        Self {
            env: Env::default(),
            db: None,
        }
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
    ($spec:ident, $db:ident,$env:ident,$inspector:ident) => {
        Box::new(EVMImpl::<'a, $spec, DB, INSPECT>::new(
            $db,
            $env,
            $inspector,
            Precompiles::new::<{ SpecId::to_precompile_id($spec::SPEC_ID) }>(),
        )) as Box<dyn Transact + 'a>
    };
}

pub fn evm_inner<'a, DB: Database, const INSPECT: bool>(
    env: &'a mut Env,
    db: &'a mut DB,
    insp: &'a mut dyn Inspector<DB>,
) -> Box<dyn Transact + 'a> {
    match env.cfg.spec_id {
        SpecId::LATEST => create_evm!(LatestSpec, db, env, insp),
        SpecId::LONDON => create_evm!(LondonSpec, db, env, insp),
        SpecId::BERLIN => create_evm!(BerlinSpec, db, env, insp),
        SpecId::ISTANBUL => create_evm!(IstanbulSpec, db, env, insp),
        SpecId::BYZANTINE => create_evm!(ByzantineSpec, db, env, insp),
        _ => panic!("Spec Not supported"),
    }
}
