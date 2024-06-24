// Includes.
use crate::{
    handler::mainnet,
    primitives::{
        db::Database, ChainSpec, EVMError, EVMResultGeneric, Spec, TransactionValidation,
    },
    Context, ContextPrecompiles,
};
use std::sync::Arc;

/// Loads precompiles into Evm
pub type LoadPrecompilesHandle<'a, ChainSpecT, DB> =
    Arc<dyn Fn() -> ContextPrecompiles<ChainSpecT, DB> + 'a>;

/// Load access list accounts and beneficiary.
/// There is no need to load Caller as it is assumed that
/// it will be loaded in DeductCallerHandle.
pub type LoadAccountsHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
        ) -> Result<
            (),
            EVMError<
                <DB as Database>::Error,
                <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
            >,
        > + 'a,
>;

/// Deduct the caller to its limit.
pub type DeductCallerHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
        ) -> EVMResultGeneric<(), ChainSpecT, <DB as Database>::Error>
        + 'a,
>;

/// Handles related to pre execution before the stack loop is started.
pub struct PreExecutionHandler<'a, ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Load precompiles
    pub load_precompiles: LoadPrecompilesHandle<'a, ChainSpecT, DB>,
    /// Main load handle
    pub load_accounts: LoadAccountsHandle<'a, ChainSpecT, EXT, DB>,
    /// Deduct max value from the caller.
    pub deduct_caller: DeductCallerHandle<'a, ChainSpecT, EXT, DB>,
}

impl<'a, ChainSpecT: ChainSpec, EXT: 'a, DB: Database + 'a>
    PreExecutionHandler<'a, ChainSpecT, EXT, DB>
{
    /// Creates mainnet MainHandles.
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            load_precompiles: Arc::new(mainnet::load_precompiles::<ChainSpecT, SPEC, DB>),
            load_accounts: Arc::new(mainnet::load_accounts::<ChainSpecT, SPEC, EXT, DB>),
            deduct_caller: Arc::new(mainnet::deduct_caller::<ChainSpecT, SPEC, EXT, DB>),
        }
    }
}

impl<'a, ChainSpecT: ChainSpec, EXT, DB: Database> PreExecutionHandler<'a, ChainSpecT, EXT, DB> {
    /// Deduct caller to its limit.
    pub fn deduct_caller(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
    ) -> EVMResultGeneric<(), ChainSpecT, DB::Error> {
        (self.deduct_caller)(context)
    }

    /// Main load
    pub fn load_accounts(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
    ) -> EVMResultGeneric<(), ChainSpecT, DB::Error> {
        (self.load_accounts)(context)
    }

    /// Load precompiles
    pub fn load_precompiles(&self) -> ContextPrecompiles<ChainSpecT, DB> {
        (self.load_precompiles)()
    }
}
