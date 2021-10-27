use crate::{
    collection::vec::Vec,
    db::{Database, WriteDatabase},
    error::ExitReason,
    evm_impl::EVMImpl,
    subroutine::State,
    BerlinSpec, BlockEnv, ByzantineSpec, CfgEnv, Env, Inspector, IstanbulSpec, LatestSpec,
    LondonSpec, NoOpInspector, Spec, SpecId, TransactOut, TransactTo, TxEnv,
};

use bytes::Bytes;
use primitive_types::{H160, H256, U256};
use revm_precompiles::Precompiles;

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

fn inner_inner<'a, DB: Database, const INSPECT: bool>(
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
    inner_inner::<DB, false>(env.cfg.spec_id, env, db, unsafe { &mut NOOP_INSP }
        as &'a mut dyn Inspector)
}

pub fn inner_inspect<'a, DB: Database>(
    env: &'a Env,
    db: &'a mut DB,
    inspector: &'a mut dyn Inspector,
) -> Box<dyn Transact + 'a> {
    inner_inner::<DB, true>(env.cfg.spec_id, env, db, inspector)
}

pub trait Transact {
    /// Do transaction.
    /// Return ExitReason, Output for call or Address if we are creating contract, gas spend, State that needs to be applied.
    fn transact(&mut self) -> (ExitReason, TransactOut, u64, State);
}

/// Struct that takes Database and enabled transact to update state dirrectly to database.
/// additionaly it allows user to partialy set config parameters.
/// Parameters that can be set are devided between Config, Block and Transaction  
pub struct EVM<'a, DB: Database + WriteDatabase> {
    env: Env,
    db: &'a mut DB,
    inspector: Option<&'a mut dyn Inspector>,
}

pub fn new<'a, DB: Database + WriteDatabase>(db: &'a mut DB) -> EVM<'a, DB> {
    EVM::new(db)
}

impl<'a, DB: Database + WriteDatabase> EVM<'a, DB> {
    pub fn new(db: &'a mut DB) -> Self {
        Self {
            env: Env::default(),
            db,
            inspector: None,
        }
    }

    pub fn state(&mut self) -> &mut DB {
        self.db
    }

    pub fn clear_gas_used(&mut self) -> U256 {
        core::mem::take(&mut self.env.block.gas_used)
    }

    pub fn transact(&mut self) -> (ExitReason, TransactOut, u64) {
        let (exit, out, gas, state) = if let Some(inspector) = &mut self.inspector {
            inner_inspect(&self.env, self.db, inspector)
        } else {
            inner(&self.env, self.db)
        }
        .transact();
        self.db.apply(state);
        (exit, out, gas)
    }
}

/// All functions inside this impl are various setters for evn.
/// all setters are prefixed with cfg_, block_, tx_ for better readability.
impl<'a, DB: Database + WriteDatabase> EVM<'a, DB> {
    pub fn env(&mut self, env: Env) {
        self.env = env;
    }

    pub fn inspector(&mut self, inspector: &'a mut dyn Inspector) {
        self.inspector = Some(inspector);
    }

    /********** CFG *****************/

    pub fn cfg(&mut self, cfg: CfgEnv) {
        self.env.cfg = cfg;
    }
    pub fn cfg_chain_id(&mut self, chain_id: U256) {
        self.env.cfg.chain_id = chain_id;
    }
    pub fn cfg_spec_id(&mut self, spec_id: SpecId) {
        self.env.cfg.spec_id = spec_id;
    }

    /********** BLOCK **************/

    pub fn block(&mut self, block: BlockEnv) {
        self.env.block = block;
    }
    pub fn block_gas_limit(&mut self, gas_limit: U256) {
        self.env.block.gas_limit = gas_limit;
    }
    pub fn block_number(&mut self, number: U256) {
        self.env.block.number = number;
    }

    pub fn block_coinbase(&mut self, coinbase: H160) {
        self.env.block.coinbase = coinbase;
    }
    pub fn block_timestamp(&mut self, timestamp: U256) {
        self.env.block.timestamp = timestamp;
    }
    pub fn block_difficulty(&mut self, difficulty: U256) {
        self.env.block.difficulty = difficulty;
    }
    pub fn block_basefee(&mut self, basefee: U256) {
        self.env.block.basefee = basefee;
    }
    pub fn block_gas_used(&mut self, gas_used: U256) {
        self.env.block.gas_used = gas_used;
    }

    /************* TX *****************/

    pub fn tx(&mut self, tx: TxEnv) {
        self.env.tx = tx;
    }
    pub fn tx_caller(&mut self, caller: H160) {
        self.env.tx.caller = caller;
    }
    pub fn tx_gas_limit(&mut self, gas_limit: u64) {
        self.env.tx.gas_limit = gas_limit;
    }
    pub fn tx_gas_price(&mut self, gas_price: U256) {
        self.env.tx.gas_price = gas_price;
    }
    pub fn tx_gas_priority_fee(&mut self, gas_priority_fee: Option<U256>) {
        self.env.tx.gas_priority_fee = gas_priority_fee;
    }
    pub fn tx_transact_to(&mut self, transact_to: TransactTo) {
        self.env.tx.transact_to = transact_to;
    }
    pub fn tx_data(&mut self, data: Bytes) {
        self.env.tx.data = data;
    }
    pub fn tx_value(&mut self, value: U256) {
        self.env.tx.value = value;
    }
    pub fn tx_nonce(&mut self, nonce: Option<u64>) {
        self.env.tx.nonce = nonce;
    }
    pub fn tx_access_list(&mut self, access_list: Vec<(H160, Vec<H256>)>) {
        self.env.tx.access_list = access_list;
    }
}
