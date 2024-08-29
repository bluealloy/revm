// Includes.
use super::{GenericContextHandle, GenericContextHandleRet};
use crate::{
    handler::mainnet,
    primitives::{db::Database, EVMError, Spec},
    Context, ContextPrecompiles,
};
use std::sync::Arc;

/// Loads precompiles into Evm
pub type LoadPrecompilesHandle<'a, DB> = Arc<dyn Fn() -> ContextPrecompiles<DB> + 'a>;

/// Load access list accounts and beneficiary.
/// There is no need to load Caller as it is assumed that
/// it will be loaded in DeductCallerHandle.
pub type LoadAccountsHandle<'a, EXT, DB> = GenericContextHandle<'a, EXT, DB>;

/// Deduct the caller to its limit.
pub type DeductCallerHandle<'a, EXT, DB> = GenericContextHandle<'a, EXT, DB>;

/// Load Auth list for EIP-7702, and returns number of created accounts.
pub type ApplyEIP7702AuthListHandle<'a, EXT, DB> = GenericContextHandleRet<'a, EXT, DB, u64>;

/// Handles related to pre execution before the stack loop is started.
pub struct PreExecutionHandler<'a, EXT, DB: Database> {
    /// Load precompiles
    pub load_precompiles: LoadPrecompilesHandle<'a, DB>,
    /// Main load handle
    pub load_accounts: LoadAccountsHandle<'a, EXT, DB>,
    /// Deduct max value from the caller.
    pub deduct_caller: DeductCallerHandle<'a, EXT, DB>,
    /// Apply EIP-7702 auth list
    pub apply_eip7702_auth_list: ApplyEIP7702AuthListHandle<'a, EXT, DB>,
}

impl<'a, EXT: 'a, DB: Database + 'a> PreExecutionHandler<'a, EXT, DB> {
    /// Creates mainnet MainHandles.
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            load_precompiles: Arc::new(mainnet::load_precompiles::<SPEC, DB>),
            load_accounts: Arc::new(mainnet::load_accounts::<SPEC, EXT, DB>),
            deduct_caller: Arc::new(mainnet::deduct_caller::<SPEC, EXT, DB>),
            apply_eip7702_auth_list: Arc::new(mainnet::apply_eip7702_auth_list::<SPEC, EXT, DB>),
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

    /// Apply EIP-7702 auth list and return gas refund on account that were already present.
    pub fn apply_eip7702_auth_list(
        &self,
        context: &mut Context<EXT, DB>,
    ) -> Result<u64, EVMError<DB::Error>> {
        (self.apply_eip7702_auth_list)(context)
    }

    /// Load precompiles
    pub fn load_precompiles(&self) -> ContextPrecompiles<DB> {
        (self.load_precompiles)()
    }
}
