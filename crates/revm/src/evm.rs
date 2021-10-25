use crate::{
    collection::vec::Vec, db::Database, error::ExitReason, evm_impl::EVMImpl, subroutine::State,
    BerlinSpec, ByzantineSpec, GlobalEnv, Inspector, IstanbulSpec, LatestSpec, LondonSpec,
    NoOpInspector, Spec, SpecId, TransactOut, TransactTo,
};

use primitive_types::{H160, H256, U256};

use bytes::Bytes;
use revm_precompiles::Precompiles;

macro_rules! create_evm {
    ($spec:tt,$db:ident,$global_env:ident,$inspector:ident) => {
        Box::new(EVMImpl::<'a, $spec, DB, INSPECT>::new(
            $db,
            $global_env,
            $inspector,
            Precompiles::new::<{ SpecId::to_precompile_id($spec::SPEC_ID) }>(),
        )) as Box<dyn EVM + 'a>
    };
}

fn new_inner<'a, DB: Database, const INSPECT: bool>(
    specid: SpecId,
    env: GlobalEnv,
    db: &'a mut DB,
    insp: &'a mut dyn Inspector,
) -> Box<dyn EVM + 'a> {
    match specid {
        SpecId::LATEST => create_evm!(LatestSpec, db, env, insp),
        SpecId::LONDON => create_evm!(LondonSpec, db, env, insp),
        SpecId::BERLIN => create_evm!(BerlinSpec, db, env, insp),
        SpecId::ISTANBUL => create_evm!(IstanbulSpec, db, env, insp),
        SpecId::BYZANTINE => create_evm!(ByzantineSpec, db, env, insp),
        _ => panic!("Spec Not supported"),
    }
}

pub fn new<'a, DB: Database>(
    specid: SpecId,
    global_env: GlobalEnv,
    db: &'a mut DB,
) -> Box<dyn EVM + 'a> {
    /**** SAFETY ********
     * NOOP_INSP is not used inside EVM because INSPECTOR flag is set to false and
     * no fn is going to be called from inspector so it is just dummy placeholder.
     * And NoOpInspector is empty struct.
     */
    static mut NOOP_INSP: NoOpInspector = NoOpInspector {};
    new_inner::<DB, false>(specid, global_env, db, unsafe { &mut NOOP_INSP }
        as &'a mut dyn Inspector)
}

pub fn new_inspect<'a, DB: Database>(
    specid: SpecId,
    global_env: GlobalEnv,
    db: &'a mut DB,
    inspector: &'a mut dyn Inspector,
) -> Box<dyn EVM + 'a> {
    new_inner::<DB, true>(specid, global_env, db, inspector)
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
