use crate::{
    collection::vec::Vec,
    db::{Database, WriteDatabase},
    error::ExitReason,
    evm_impl::EVMImpl,
    subroutine::State,
    BerlinSpec, BlockEnv, ByzantineSpec, CfgEnv, Env, Inspector, IstanbulSpec, LatestSpec,
    LondonSpec, NoOpInspector, Spec, SpecId, TransactOut, TransactTo, TxEnv,
};

use primitive_types::{H160, H256, U256};

use bytes::Bytes;
use revm_precompiles::Precompiles;

macro_rules! create_evm {
    ($spec:tt,$db:ident,$env:ident,$inspector:ident) => {
        Box::new(EVMImpl::<'a, $spec, DB, INSPECT>::new(
            $db,
            $env,
            $inspector,
            Precompiles::new::<{ SpecId::to_precompile_id($spec::SPEC_ID) }>(),
        )) as Box<dyn EVM + 'a>
    };
}

fn new_inner<'a, DB: Database, const INSPECT: bool>(
    specid: SpecId,
    env: &'a Env,
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

pub fn new<'a, DB: Database>(env: &'a Env, db: &'a mut DB) -> Box<dyn EVM + 'a> {
    /**** SAFETY ********
     * NOOP_INSP is not used inside EVM because INSPECTOR flag is set to false and
     * no fn is going to be called from inspector so it is just dummy placeholder.
     * And NoOpInspector is empty struct.
     */
    static mut NOOP_INSP: NoOpInspector = NoOpInspector {};
    new_inner::<DB, false>(env.cfg.spec_id, env, db, unsafe { &mut NOOP_INSP }
        as &'a mut dyn Inspector)
}

pub fn new_inspect<'a, DB: Database>(
    env: &'a Env,
    db: &'a mut DB,
    inspector: &'a mut dyn Inspector,
) -> Box<dyn EVM + 'a> {
    new_inner::<DB, true>(env.cfg.spec_id, env, db, inspector)
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

/// Struct that takes Database and enabled transact to update state dirrectly to database.
/// additionaly it allows user to partialy set config parameters.
/// Parameters that can be set are devided between Config, Block and Transaction  
pub struct StatefullEVM<'a, DB: Database + WriteDatabase> {
    env: Env,
    db: &'a mut DB,
}

impl<'a, DB: Database + WriteDatabase> StatefullEVM<'a, DB> {
    pub fn new(db: &'a mut DB) -> Self {
        Self {
            env: Env::default(),
            db,
        }
    }

    pub fn state(&mut self) -> &mut DB {
        self.db
    }

    pub fn clear_gas_used(&mut self) -> U256 {
        core::mem::take(&mut self.env.block.gas_used)
    }

    pub fn transact(
        &mut self,
        caller: H160,
        transact_to: TransactTo,
        value: U256,
        data: Bytes,
        gas_limit: u64,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> (ExitReason, TransactOut, u64) {
        let (exit, out, gas, state) = new(&self.env, self.db).transact(
            caller,
            transact_to,
            value,
            data,
            gas_limit,
            access_list,
        );
        self.db.apply(state);
        (exit, out, gas)
    }
}

/// All functions inside this impl are various setters for evn.
/// all setters are prefixed with cfg_, block_, tx_ for better readability.
impl<'a, DB: Database + WriteDatabase> StatefullEVM<'a, DB> {
    pub fn env(mut self, env: Env) -> Self {
        self.env = env;
        self
    }

    /********** CFG *****************/

    pub fn cfg(mut self, cfg: CfgEnv) -> Self {
        self.env.cfg = cfg;
        self
    }
    pub fn cfg_chain_id(mut self, chain_id: U256) -> Self {
        self.env.cfg.chain_id = chain_id;
        self
    }
    pub fn cfg_spec_id(mut self, spec_id: SpecId) -> Self {
        self.env.cfg.spec_id = spec_id;
        self
    }

    /********** BLOCK **************/

    pub fn block(mut self, block: BlockEnv) -> Self {
        self.env.block = block;
        self
    }
    pub fn block_gas_limit(mut self, gas_limit: U256) -> Self {
        self.env.block.gas_limit = gas_limit;
        self
    }
    pub fn block_number(mut self, number: U256) -> Self {
        self.env.block.number = number;
        self
    }

    pub fn block_coinbase(mut self, coinbase: H160) -> Self {
        self.env.block.coinbase = coinbase;
        self
    }
    pub fn block_timestamp(mut self, timestamp: U256) -> Self {
        self.env.block.timestamp = timestamp;
        self
    }
    pub fn block_difficulty(mut self, difficulty: U256) -> Self {
        self.env.block.difficulty = difficulty;
        self
    }
    pub fn block_basefee(mut self, basefee: U256) -> Self {
        self.env.block.basefee = basefee;
        self
    }
    pub fn block_gas_used(mut self, gas_used: U256) -> Self {
        self.env.block.gas_used = gas_used;
        self
    }

    /************* TX *****************/

    pub fn tx(mut self, tx: TxEnv) -> Self {
        self.env.tx = tx;
        self
    }
    pub fn tx_caller(mut self, caller: H160) -> Self {
        self.env.tx.caller = caller;
        self
    }
    pub fn tx_gas_limit(mut self, gas_limit: u64) -> Self {
        self.env.tx.gas_limit = gas_limit;
        self
    }
    pub fn tx_gas_price(mut self, gas_price: U256) -> Self {
        self.env.tx.gas_price = gas_price;
        self
    }
    pub fn tx_gas_priority_fee(mut self, gas_priority_fee: Option<U256>) -> Self {
        self.env.tx.gas_priority_fee = gas_priority_fee;
        self
    }
    pub fn tx_transact_to(mut self, transact_to: TransactTo) -> Self {
        self.env.tx.transact_to = transact_to;
        self
    }
    pub fn tx_value(mut self, value: U256) -> Self {
        self.env.tx.value = value;
        self
    }
    pub fn tx_nonce(mut self, nonce: Option<u64>) -> Self {
        self.env.tx.nonce = nonce;
        self
    }
    pub fn tx_access_list(mut self, access_list: Vec<(H160, Vec<H256>)>) -> Self {
        self.env.tx.access_list = access_list;
        self
    }
}
