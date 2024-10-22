use crate::{Context, JournaledState as JournaledStateImpl};
use auto_impl::auto_impl;
use database_interface::{Database, EmptyDB};
use specification::hardfork::{LatestSpec, Spec};
use std::sync::Arc;
use transaction::Transaction;
use wiring::{
    default::{CfgEnv, Env, EnvWiring},
    evm_wiring::HardforkTrait,
    journaled_state::JournaledState,
    result::{EVMError, EVMErrorWiring, EVMResultGeneric, InvalidTransaction},
    Block, Cfg, EthereumWiring, EvmWiring,
};

#[auto_impl(&, &mut, Box, Arc)]
pub trait CfgGetter {
    type Cfg: Cfg;

    fn cfg(&self) -> &Self::Cfg;
}

impl<EvmWiringT: EvmWiring> CfgGetter for Context<EvmWiringT> {
    type Cfg = CfgEnv;

    fn cfg(&self) -> &Self::Cfg {
        &self.evm.inner.env.cfg
    }
}

impl<BLOCK: Block, TX: Transaction> CfgGetter for Env<BLOCK, TX> {
    type Cfg = CfgEnv;

    fn cfg(&self) -> &Self::Cfg {
        &self.cfg
    }
}

/// Helper that extracts database error from [`JournalStateGetter`].
pub type JournalStateGetterDBError<CTX> =
    <<<CTX as JournalStateGetter>::Journal as JournaledState>::Database as Database>::Error;

#[auto_impl(&mut, Box)]
pub trait JournalStateGetter {
    type Journal: JournaledState;

    fn journal(&mut self) -> &mut Self::Journal;
}

impl<EvmWiringT: EvmWiring> JournalStateGetter for Context<EvmWiringT> {
    type Journal = JournaledStateImpl<EvmWiringT::Database>;

    fn journal(&mut self) -> &mut Self::Journal {
        &mut self.evm.journaled_state
    }
}

#[auto_impl(&mut, Box)]
pub trait DatabaseGetter {
    type Database: Database;

    fn db(&mut self) -> &mut Self::Database;
}

impl<EvmWiringT: EvmWiring> DatabaseGetter for Context<EvmWiringT> {
    type Database = EvmWiringT::Database;

    fn db(&mut self) -> &mut Self::Database {
        &mut self.evm.journaled_state.database
    }
}

/// TODO change name of the trait
pub trait ErrorGetter {
    type Error;

    fn take_error(&mut self) -> Result<(), Self::Error>;
}

impl<EvmWiringT: EvmWiring> ErrorGetter for Context<EvmWiringT> {
    type Error = EVMErrorWiring<EvmWiringT>;

    fn take_error(&mut self) -> Result<(), Self::Error> {
        self.evm.inner.take_error().map_err(EVMError::Database)
    }
}

#[auto_impl(&, &mut, Box, Arc)]
pub trait TransactionGetter {
    type Transaction: Transaction;

    fn tx(&self) -> &Self::Transaction;
}

impl<BLOCK: Block, TX: Transaction> TransactionGetter for Env<BLOCK, TX> {
    type Transaction = TX;

    fn tx(&self) -> &Self::Transaction {
        &self.tx
    }
}

impl<EvmWiringT: EvmWiring> TransactionGetter for Context<EvmWiringT> {
    type Transaction = EvmWiringT::Transaction;

    fn tx(&self) -> &Self::Transaction {
        &self.evm.env.tx
    }
}

#[auto_impl(&, &mut, Box, Arc)]
pub trait BlockGetter {
    type Block: Block;

    fn block(&self) -> &Self::Block;
}

impl<BLOCK: Block, TX: Transaction> BlockGetter for Env<BLOCK, TX> {
    type Block = BLOCK;

    fn block(&self) -> &Self::Block {
        &self.block
    }
}

impl<EvmWiringT: EvmWiring> BlockGetter for Context<EvmWiringT> {
    type Block = EvmWiringT::Block;

    fn block(&self) -> &Self::Block {
        &self.evm.env.block
    }
}

pub type EvmError<DB, TX> = EVMError<DB, TX>;

// ENV
