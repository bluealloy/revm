// Includes.
use crate::{
    handler::mainnet,
    primitives::{db::Database, EVMError, EVMResultGeneric, Spec},
    Context,
};
use alloc::sync::Arc;
use revm_precompile::Precompiles;

/// Loads precompiles into Evm
pub type LoadPrecompilesHandle<'a> = Arc<dyn Fn() -> Precompiles + 'a>;

/// Load access list accounts and beneficiary.
/// There is no need to load Caller as it is assumed that
/// it will be loaded in DeductCallerHandle.
pub type LoadAccountsHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>) -> Result<(), EVMError<<DB as Database>::Error>> + 'a>;

/// Deduct the caller to its limit.
pub type DeductCallerHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>) -> EVMResultGeneric<(), <DB as Database>::Error> + 'a>;

/// Handles related to pre execution before the stack loop is started.
pub struct PreExecutionHandler<'a, EXT, DB: Database> {
    /// Load precompiles
    pub load_precompiles: LoadPrecompilesHandle<'a>,
    /// Main load handle
    pub load_accounts: LoadAccountsHandle<'a, EXT, DB>,
    /// Deduct max value from the caller.
    pub deduct_caller: DeductCallerHandle<'a, EXT, DB>,
}

impl<'a, EXT: 'a, DB: Database + 'a> PreExecutionHandler<'a, EXT, DB> {
    /// Creates mainnet MainHandles.
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            load_precompiles: Arc::new(mainnet::load_precompiles::<SPEC>),
            load_accounts: Arc::new(mainnet::load::<SPEC, EXT, DB>),
            deduct_caller: Arc::new(mainnet::deduct_caller::<SPEC, EXT, DB>),
        }
    }
}

impl<'a, EXT, DB: Database> PreExecutionHandler<'a, EXT, DB> {
    /// Deduct caller to its limit.
    pub fn deduct_caller(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>> {
        (self.deduct_caller)(context)
    }

    /// Main load
    pub fn load_accounts(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>> {
        (self.load_accounts)(context)
    }

    /// Load precompiles
    pub fn load_precompiles(&self) -> Precompiles {
        (self.load_precompiles)()
    }
}
