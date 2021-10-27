use crate::{
    db::{Database, WriteDatabase},
    error::ExitReason,
    evm_impl::EVMImpl,
    subroutine::State,
    BerlinSpec, ByzantineSpec, Env, Inspector, IstanbulSpec, LatestSpec, LondonSpec, NoOpInspector,
    Spec, SpecId, TransactOut,
};
use revm_precompiles::Precompiles;
/// Struct that takes Database and enabled transact to update state dirrectly to database.
/// additionaly it allows user to set all environment parameters.
/// Parameters that can be set are devided between Config, Block and Transaction(tx)
pub struct EVM<'a, DB: Database + WriteDatabase> {
    pub env: Env,
    pub inspector: Option<&'a mut dyn Inspector>,
    db: Option<DB>,
}

pub fn new<'a, DB: Database + WriteDatabase>() -> EVM<'a, DB> {
    EVM::new()
}

impl<'a, DB: Database + WriteDatabase> EVM<'a, DB> {
    pub fn new() -> Self {
        Self {
            env: Env::default(),
            db: None,
            inspector: None,
        }
    }

    pub fn db(&mut self) -> Option<&mut DB> {
        self.db.as_mut()
    }

    pub fn take_db(&mut self) -> DB {
        core::mem::take(&mut self.db).unwrap()
    }

    /// do dummy transaction and return change state. Does not touch the DB.
    pub fn transact_only(&mut self) -> (ExitReason, TransactOut, u64, State) {
        if let Some(db) = &mut self.db {
            let (exit, out, gas, state) = if let Some(inspector) = self.inspector.as_mut() {
                inner_inspect(&self.env, db, *inspector)
            } else {
                inner(&self.env, db)
            }
            .transact();
            return (exit, out, gas, state);
        } else {
            panic!("Database handler needs to be set");
        }
    }

    /// Do transaction and apply result to database
    pub fn transact(&mut self) -> (ExitReason, TransactOut, u64) {
        let (exit, out, gas, state) = self.transact_only();
        self.db.as_mut().unwrap().apply(state);
        (exit, out, gas)
    }
}

/// All functions inside this impl are various setters for evn.
/// all setters are prefixed with cfg_, block_, tx_ for better readability.
impl<'a, DB: Database + WriteDatabase> EVM<'a, DB> {
    pub fn inspector(&mut self, inspector: &'a mut dyn Inspector) {
        self.inspector = Some(inspector);
    }

    pub fn database(&mut self, db: DB) {
        self.db = Some(db);
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

fn inner_wrapper<'a, DB: Database, const INSPECT: bool>(
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

pub fn inner<'a, DB: Database>(env: &'a Env, db: &'a mut DB) -> Box<dyn Transact + 'a> {
    /**** SAFETY ********
     * NOOP_INSP is not used inside EVM because INSPECTOR flag is set to false and
     * no fn is going to be called from inspector so it is just dummy placeholder.
     * And NoOpInspector is empty struct.
     */
    static mut NOOP_INSP: NoOpInspector = NoOpInspector {};
    inner_wrapper::<DB, false>(env.cfg.spec_id, env, db, unsafe { &mut NOOP_INSP }
        as &'a mut dyn Inspector)
}

pub fn inner_inspect<'a, DB: Database>(
    env: &'a Env,
    db: &'a mut DB,
    inspector: &'a mut dyn Inspector,
) -> Box<dyn Transact + 'a> {
    inner_wrapper::<DB, true>(env.cfg.spec_id, env, db, inspector)
}

pub trait Transact {
    /// Do transaction.
    /// Return ExitReason, Output for call or Address if we are creating contract, gas spend, State that needs to be applied.
    fn transact(&mut self) -> (ExitReason, TransactOut, u64, State);
}
