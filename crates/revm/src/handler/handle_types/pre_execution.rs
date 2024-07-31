// Includes.
use crate::{
    handler::mainnet,
    primitives::{db::Database, EVMResultGeneric, Spec},
    Context, ContextPrecompiles, EvmWiring,
};
use std::sync::Arc;

/// Loads precompiles into Evm
pub type LoadPrecompilesHandle<'a, EvmWiringT, DB> =
    Arc<dyn Fn() -> ContextPrecompiles<EvmWiringT, DB> + 'a>;

/// Load access list accounts and beneficiary.
/// There is no need to load Caller as it is assumed that
/// it will be loaded in DeductCallerHandle.
pub type LoadAccountsHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
        ) -> EVMResultGeneric<(), EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Deduct the caller to its limit.
pub type DeductCallerHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
        ) -> EVMResultGeneric<(), EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Handles related to pre execution before the stack loop is started.
pub struct PreExecutionHandler<'a, EvmWiringT: EvmWiring, EXT, DB: Database> {
    /// Load precompiles
    pub load_precompiles: LoadPrecompilesHandle<'a, EvmWiringT, DB>,
    /// Main load handle
    pub load_accounts: LoadAccountsHandle<'a, EvmWiringT, EXT, DB>,
    /// Deduct max value from the caller.
    pub deduct_caller: DeductCallerHandle<'a, EvmWiringT, EXT, DB>,
}

impl<'a, EvmWiringT: EvmWiring, EXT: 'a, DB: Database + 'a>
    PreExecutionHandler<'a, EvmWiringT, EXT, DB>
{
    /// Creates mainnet MainHandles.
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            load_precompiles: Arc::new(mainnet::load_precompiles::<EvmWiringT, SPEC, DB>),
            load_accounts: Arc::new(mainnet::load_accounts::<EvmWiringT, SPEC, EXT, DB>),
            deduct_caller: Arc::new(mainnet::deduct_caller::<EvmWiringT, SPEC, EXT, DB>),
        }
    }
}

impl<'a, EvmWiringT: EvmWiring, EXT, DB: Database> PreExecutionHandler<'a, EvmWiringT, EXT, DB> {
    /// Deduct caller to its limit.
    pub fn deduct_caller(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
    ) -> EVMResultGeneric<(), EvmWiringT, DB::Error> {
        (self.deduct_caller)(context)
    }

    /// Main load
    pub fn load_accounts(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
    ) -> EVMResultGeneric<(), EvmWiringT, DB::Error> {
        (self.load_accounts)(context)
    }

    /// Load precompiles
    pub fn load_precompiles(&self) -> ContextPrecompiles<EvmWiringT, DB> {
        (self.load_precompiles)()
    }
}
