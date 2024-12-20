use crate::Context;
use context_interface::{Block, Cfg, Database, Journal, Transaction};

/// Helper trait for wiring up the context with generic Database.
pub trait ContextWiring<DB: Database> {
    type Block: Block;
    type Tx: Transaction;
    type Cfg: Cfg;
    type Journal: Journal<Database = DB>;
    type Chain;
}

/// Type that bind [`Context`] with the [`ContextWiring`].
pub type ContextWire<DB, CTXW> = Context<
    <CTXW as ContextWiring<DB>>::Block,
    <CTXW as ContextWiring<DB>>::Tx,
    <CTXW as ContextWiring<DB>>::Cfg,
    DB,
    <CTXW as ContextWiring<DB>>::Journal,
    <CTXW as ContextWiring<DB>>::Chain,
>;
