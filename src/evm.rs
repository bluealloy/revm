use crate::{
    collection::{vec::Vec, Map},
    evm_impl::EVMImpl,
    inspector::NoOpInspector,
    opcode::Control,
    precompiles, AccountInfo, BerlinSpec, ExitRevert, FrontierSpec, Inspector, IstanbulSpec,
    LatestSpec, SpecId,
};
use core::cmp::min;
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};
use std::marker::PhantomData;

use super::precompiles::{
    Precompile, PrecompileFn, PrecompileOutput, PrecompileResult, Precompiles,
};
use crate::{
    db::Database,
    error::{ExitError, ExitReason, ExitSucceed},
    machine::{Contract, Gas, Machine, Memory, Stack},
    opcode::OpCode,
    spec::{NotStaticSpec, Spec},
    subroutine::{Account, State, SubRoutine},
    util, CallContext, CreateScheme, GlobalEnv, Log, Transfer,
};
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
    fn call(
        &mut self,
        caller: H160,
        address: H160,
        value: U256,
        data: Bytes,
        gas_limit: u64,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> (ExitReason, Bytes, u64, State);
    fn create(
        &mut self,
        caller: H160,
        value: U256,
        init_code: Bytes,
        create_scheme: CreateScheme,
        gas_limit: u64,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> (ExitReason, Option<H160>, u64, State);
}
