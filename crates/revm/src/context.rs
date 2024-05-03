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
    primitives::ChainSpec,
};
use std::boxed::Box;

/// Main Context structure that contains both EvmContext and External context.
pub struct Context<EXT, DB: Database> {
    /// Evm Context (internal context).
    pub evm: EvmContext<DB>,
    /// External contexts.
    pub external: EXT,
}

impl<EXT: Clone, DB: Database + Clone> Clone for Context<EXT, DB>
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

impl Default for Context<(), EmptyDB> {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl Context<(), EmptyDB> {
    /// Creates empty context. This is useful for testing.
    pub fn new_empty() -> Context<(), EmptyDB> {
        Context {
            evm: EvmContext::new(EmptyDB::new()),
            external: (),
        }
    }
}

impl<DB: Database> Context<(), DB> {
    /// Creates new context with database.
    pub fn new_with_db(db: DB) -> Context<(), DB> {
        Context {
            evm: EvmContext::new_with_env(db, Box::default()),
            external: (),
        }
    }
}

impl<EXT, DB: Database> Context<EXT, DB> {
    /// Creates new context with external and database.
    pub fn new(evm: EvmContext<DB>, external: EXT) -> Context<EXT, DB> {
        Context { evm, external }
    }
}

/// Context with handler configuration.
pub struct ContextWithChainSpec<ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Context of execution.
    pub context: Context<EXT, DB>,
    /// Handler configuration.
    pub spec_id: ChainSpecT::Hardfork,
}

impl<ChainSpecT: ChainSpec, EXT, DB: Database> ContextWithChainSpec<ChainSpecT, EXT, DB> {
    /// Creates new context with handler configuration.
    pub fn new(context: Context<EXT, DB>, spec_id: ChainSpecT::Hardfork) -> Self {
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
