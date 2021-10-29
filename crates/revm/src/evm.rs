use crate::{
    db::{Database, WriteDatabase},
    error::ExitReason,
    evm_impl::{EVMImpl, Transact},
    subroutine::State,
    BerlinSpec, ByzantineSpec, Env, Inspector, IstanbulSpec, LatestSpec, LondonSpec, NoOpInspector,
    Spec, SpecId, TransactOut,
};
use revm_precompiles::Precompiles;
/// Struct that takes Database and enabled transact to update state dirrectly to database.
/// additionaly it allows user to set all environment parameters.
/// 
/// Parameters that can be set are devided between Config, Block and Transaction(tx)
/// 
/// For transacting on EVM you can call transact_commit that will automatically apply changes to db.
pub struct EVM<DB: Database + WriteDatabase> {
    pub env: Env,
    pub db: Option<DB>,
}

pub fn new<DB: Database + WriteDatabase>() -> EVM<DB> {
    EVM::new()
}

impl<DB: Database + WriteDatabase> EVM<DB> {
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

    /// Execute transaction without writing to DB, return change state.
    pub fn transact(&mut self) -> (ExitReason, TransactOut, u64, State) {
        if let Some(db) = self.db.as_mut() {
            let mut noop = NoOpInspector {};
            let out =
                evm_inner::<DB, false>(self.env.cfg.spec_id, &mut self.env, db, &mut noop).transact();
            out
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Execute transaction and apply result to database
    pub fn transact_commit(&mut self) -> (ExitReason, TransactOut, u64) {
        let (exit, out, gas, state) = self.transact();
        self.db.as_mut().unwrap().apply(state);
        (exit, out, gas)
    }

    /// Execute transaction with given inspector, without wring to DB. Return change state.
    pub fn inspect<INSP: Inspector>(
        &mut self,
        mut inspector: INSP,
    ) -> (ExitReason, TransactOut, u64, State) {
        if let Some(db) = self.db.as_mut() {
            let out = evm_inner::<DB, true>(self.env.cfg.spec_id, &mut self.env, db, &mut inspector)
                .transact();
            out
        } else {
            panic!("Database needs to be set");
        }
    }

    /// Inspect transaction and commit changes to database.
    pub fn inspect_commit<INSP: Inspector>(
        &mut self,
        inspector: INSP,
    ) -> (ExitReason, TransactOut, u64) {
        let (exit, out, gas, state) = self.inspect(inspector);
        self.db.as_mut().unwrap().apply(state);
        (exit, out, gas)
    }
}

macro_rules! create_evm {
    ($spec:tt,$db:ident,$env:ident,$inspector:ident) => {
        Box::new(EVMImpl::<'a, $spec, DB, INSPECT>::new(
            $db,
            $env,
            $inspector,
            Precompiles::new::<{ SpecId::to_precompile_id($spec::SPEC_ID) }>(),
        )) as Box<dyn Transact + 'a>
    };
}

fn evm_inner<'a, DB: Database, const INSPECT: bool>(
    specid: SpecId,
    env: &'a Env,
    db: &'a mut DB,
    insp: &'a mut dyn Inspector,
) -> Box<dyn Transact + 'a> {
    match specid {
        SpecId::LATEST => create_evm!(LatestSpec, db, env, insp),
        SpecId::LONDON => create_evm!(LondonSpec, db, env, insp),
        SpecId::BERLIN => create_evm!(BerlinSpec, db, env, insp),
        SpecId::ISTANBUL => create_evm!(IstanbulSpec, db, env, insp),
        SpecId::BYZANTINE => create_evm!(ByzantineSpec, db, env, insp),
        _ => panic!("Spec Not supported"),
    }
}
