mod context_precompiles;
pub(crate) mod evm_context;
mod inner_evm_context;

pub use context_precompiles::{
    ContextPrecompile, ContextPrecompiles, ContextStatefulPrecompile, ContextStatefulPrecompileArc,
    ContextStatefulPrecompileBox, ContextStatefulPrecompileMut,
};
pub use evm_context::EvmContext;
pub use inner_evm_context::InnerEvmContext;

use crate::{
    db::{Database, EmptyDB},
    primitives::{ChainSpec, EthChainSpec},
};
use std::boxed::Box;

/// Main Context structure that contains both EvmContext and External context.
pub struct Context<ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Evm Context (internal context).
    pub evm: EvmContext<ChainSpecT, DB>,
    /// External contexts.
    pub external: EXT,
}

impl<ChainSpecT: ChainSpec, EXT: Clone, DB: Database + Clone> Clone for Context<ChainSpecT, EXT, DB>
where
    DB::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            evm: self.evm.clone(),
            external: self.external.clone(),
        }
    }
}

impl Default for Context<EthChainSpec, (), EmptyDB> {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl<ChainSpecT: ChainSpec> Context<ChainSpecT, (), EmptyDB> {
    /// Creates empty context. This is useful for testing.
    pub fn new_empty() -> Context<ChainSpecT, (), EmptyDB> {
        Context {
            evm: EvmContext::new(EmptyDB::new()),
            external: (),
        }
    }
}

impl<ChainSpecT: ChainSpec, DB: Database> Context<ChainSpecT, (), DB> {
    /// Creates new context with database.
    pub fn new_with_db(db: DB) -> Context<ChainSpecT, (), DB> {
        Context {
            evm: EvmContext::new_with_env(db, Box::default()),
            external: (),
        }
    }
}

impl<ChainSpecT: ChainSpec, EXT, DB: Database> Context<ChainSpecT, EXT, DB> {
    /// Creates new context with external and database.
    pub fn new(evm: EvmContext<ChainSpecT, DB>, external: EXT) -> Context<ChainSpecT, EXT, DB> {
        Context { evm, external }
    }
}

/// Context with handler configuration.
pub struct ContextWithChainSpec<ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Context of execution.
    pub context: Context<ChainSpecT, EXT, DB>,
    /// Handler configuration.
    pub spec_id: ChainSpecT::Hardfork,
}

impl<ChainSpecT: ChainSpec, EXT, DB: Database> ContextWithChainSpec<ChainSpecT, EXT, DB> {
    /// Creates new context with handler configuration.
    pub fn new(context: Context<ChainSpecT, EXT, DB>, spec_id: ChainSpecT::Hardfork) -> Self {
        Self { spec_id, context }
    }
}

impl<ChainSpecT: ChainSpec, EXT: Clone, DB: Database + Clone> Clone
    for ContextWithChainSpec<ChainSpecT, EXT, DB>
where
    DB::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            spec_id: self.spec_id,
        }
    }
}
