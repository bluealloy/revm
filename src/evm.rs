use crate::{BerlinSpec, CreateScheme, FrontierSpec, GlobalEnv, Inspector, IstanbulSpec, LatestSpec, SpecId, TransactOut, TransactTo, collection::vec::Vec, db::Database, error::ExitReason, evm_impl::EVMImpl, subroutine::State};

use primitive_types::{H160, H256, U256};
use sha3::Digest;

use super::precompiles::Precompiles;
use bytes::Bytes;

fn new_inner<'a, DB: Database, const INSPECT: bool>(
    specid: SpecId,
    global_env: GlobalEnv,
    db: &'a mut DB,
    inspector: Option<Box<dyn Inspector + 'a>>,
) -> Box<dyn EVM + 'a> {
    match specid {
        SpecId::LATEST => Box::new(EVMImpl::<'a, LatestSpec, DB, INSPECT>::new(
            db,
            global_env,
            inspector,
            Precompiles::new_berlin(),
        )) as Box<dyn EVM + 'a>,
        SpecId::BERLIN => Box::new(EVMImpl::<'a, BerlinSpec, DB, INSPECT>::new(
            db,
            global_env,
            inspector,
            Precompiles::new_berlin(),
        )) as Box<dyn EVM + 'a>,
        SpecId::ISTANBUL => Box::new(EVMImpl::<'a, IstanbulSpec, DB, INSPECT>::new(
            db,
            global_env,
            inspector,
            Precompiles::new_istanbul(),
        )) as Box<dyn EVM + 'a>,
        SpecId::BYZANTINE => Box::new(EVMImpl::<'a, FrontierSpec, DB, INSPECT>::new(
            db,
            global_env,
            inspector,
            Precompiles::new_berlin(),
        )) as Box<dyn EVM + 'a>,
        _ => panic!("Spec Not supported"),
    }
}
pub fn new<'a, DB: Database>(
    specid: SpecId,
    global_env: GlobalEnv,
    db: &'a mut DB,
) -> Box<dyn EVM + 'a> {
    new_inner::<DB, false>(specid, global_env, db, None)
}

pub fn new_inspect<'a, DB: Database>(
    specid: SpecId,
    global_env: GlobalEnv,
    db: &'a mut DB,
    inspector: Box<dyn Inspector + 'a>,
) -> Box<dyn EVM + 'a> {
    new_inner::<DB, true>(specid, global_env, db, Some(inspector))
}


pub trait EVM {
    /// Do transaction.
    /// Return ExitReason, Output for call or Address if we are creating contract, gas spend, State that needs to be applied.
    fn transact(
        &mut self,
        caller: H160,
        transact_to: TransactTo,
        value: U256,
        data: Bytes,
        gas_limit: u64,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> (ExitReason, TransactOut, u64, State);
}
