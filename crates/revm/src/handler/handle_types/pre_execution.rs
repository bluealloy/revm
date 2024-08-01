// Includes.
use crate::{
    handler::mainnet,
    primitives::{EVMResultGeneric, Spec},
    Context, ContextPrecompiles, EvmWiring,
};
use std::sync::Arc;

/// Loads precompiles into Evm
pub type LoadPrecompilesHandle<'a, EvmWiringT> =
    Arc<dyn Fn() -> ContextPrecompiles<EvmWiringT> + 'a>;

/// Load access list accounts and beneficiary.
/// There is no need to load Caller as it is assumed that
/// it will be loaded in DeductCallerHandle.
pub type LoadAccountsHandle<'a, EvmWiringT> =
    Arc<dyn Fn(&mut Context<EvmWiringT>) -> EVMResultGeneric<(), EvmWiringT> + 'a>;

/// Deduct the caller to its limit.
pub type DeductCallerHandle<'a, EvmWiringT> =
    Arc<dyn Fn(&mut Context<EvmWiringT>) -> EVMResultGeneric<(), EvmWiringT> + 'a>;

/// Handles related to pre execution before the stack loop is started.
pub struct PreExecutionHandler<'a, EvmWiringT: EvmWiring> {
    /// Load precompiles
    pub load_precompiles: LoadPrecompilesHandle<'a, EvmWiringT>,
    /// Main load handle
    pub load_accounts: LoadAccountsHandle<'a, EvmWiringT>,
    /// Deduct max value from the caller.
    pub deduct_caller: DeductCallerHandle<'a, EvmWiringT>,
}

impl<'a, EvmWiringT: EvmWiring + 'a> PreExecutionHandler<'a, EvmWiringT> {
    /// Creates mainnet MainHandles.
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            load_precompiles: Arc::new(mainnet::load_precompiles::<EvmWiringT, SPEC>),
            load_accounts: Arc::new(mainnet::load_accounts::<EvmWiringT, SPEC>),
            deduct_caller: Arc::new(mainnet::deduct_caller::<EvmWiringT, SPEC>),
        }
    }
}

impl<'a, EvmWiringT: EvmWiring> PreExecutionHandler<'a, EvmWiringT> {
    /// Deduct caller to its limit.
    pub fn deduct_caller(
        &self,
        context: &mut Context<EvmWiringT>,
    ) -> EVMResultGeneric<(), EvmWiringT> {
        (self.deduct_caller)(context)
    }

    /// Main load
    pub fn load_accounts(
        &self,
        context: &mut Context<EvmWiringT>,
    ) -> EVMResultGeneric<(), EvmWiringT> {
        (self.load_accounts)(context)
    }

    /// Load precompiles
    pub fn load_precompiles(&self) -> ContextPrecompiles<EvmWiringT> {
        (self.load_precompiles)()
    }
}
