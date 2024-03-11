// Includes.
use crate::{
    handler::mainnet::PreExecutionImpl,
    primitives::{db::Database, EVMError, Spec},
    Context, ContextPrecompiles,
};

/// Loads precompiles into Evm
pub trait LoadPrecompilesTrait<DB: Database>: Send + Sync {
    fn load_precompiles(&self) -> ContextPrecompiles<DB>;
}

/// Load access list accounts and beneficiary.
/// There is no need to load Caller as it is assumed that
/// it will be loaded in DeductCallerHandle.
pub trait LoadAccountsTrait<EXT, DB: Database>: Send + Sync {
    fn load_accounts(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>>;
}

pub trait DeductCallerTrait<EXT, DB: Database>: Send + Sync {
    fn deduct_caller(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>>;
}

/// Handles related to pre execution before the stack loop is started.
pub struct PreExecutionHandler<EXT, DB: Database> {
    /// Load precompiles
    pub load_precompiles: Box<dyn LoadPrecompilesTrait<DB>>,
    /// Main load handle
    pub load_accounts: Box<dyn LoadAccountsTrait<EXT, DB>>,
    /// Deduct max value from the caller.
    pub deduct_caller: Box<dyn DeductCallerTrait<EXT, DB>>,
}

impl<EXT, DB: Database> PreExecutionHandler<EXT, DB> {
    /// Creates mainnet MainHandles.
    pub fn new<SPEC: Spec>() -> Self {
        Self {
            load_precompiles: Box::<PreExecutionImpl<SPEC>>::default(),
            load_accounts: Box::<PreExecutionImpl<SPEC>>::default(),
            deduct_caller: Box::<PreExecutionImpl<SPEC>>::default(),
        }
    }
}

impl<EXT, DB: Database> PreExecutionHandler<EXT, DB> {
    /// Deduct caller to its limit.
    pub fn deduct_caller(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>> {
        self.deduct_caller.deduct_caller(context)
    }

    /// Main load
    pub fn load_accounts(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>> {
        self.load_accounts.load_accounts(context)
    }

    /// Load precompiles
    pub fn load_precompiles(&self) -> ContextPrecompiles<DB> {
        self.load_precompiles.load_precompiles()
    }
}
