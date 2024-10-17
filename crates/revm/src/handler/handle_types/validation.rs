use crate::{handler::mainnet, Context, EvmWiring};
use specification::hardfork::Spec;
use std::sync::Arc;
use transaction::Transaction;
use wiring::{
    default::EnvWiring,
    result::{EVMResultGeneric, InvalidTransaction},
};

/// Handle that validates env.
pub type ValidateEnvHandle<'a, EvmWiringT> =
    Arc<dyn Fn(&EnvWiring<EvmWiringT>) -> EVMResultGeneric<(), EvmWiringT> + 'a>;

/// Handle that validates transaction environment against the state.
/// Second parametar is initial gas.
pub type ValidateTxEnvAgainstState<'a, EvmWiringT> =
    Arc<dyn Fn(&mut Context<EvmWiringT>) -> EVMResultGeneric<(), EvmWiringT> + 'a>;

/// Initial gas calculation handle
pub type ValidateInitialTxGasHandle<'a, EvmWiringT> =
    Arc<dyn Fn(&EnvWiring<EvmWiringT>) -> EVMResultGeneric<u64, EvmWiringT> + 'a>;

/// Handles related to validation.
pub struct ValidationHandler<'a, EvmWiringT: EvmWiring> {
    /// Validate and calculate initial transaction gas.
    pub initial_tx_gas: ValidateInitialTxGasHandle<'a, EvmWiringT>,
    /// Validate transactions against state data.
    pub tx_against_state: ValidateTxEnvAgainstState<'a, EvmWiringT>,
    /// Validate Env.
    pub env: ValidateEnvHandle<'a, EvmWiringT>,
}

impl<'a, EvmWiringT: EvmWiring + 'a> ValidationHandler<'a, EvmWiringT>
where
    <EvmWiringT::Transaction as Transaction>::TransactionError: From<InvalidTransaction>,
{
    /// Create new ValidationHandles
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            initial_tx_gas: Arc::new(mainnet::validate_initial_tx_gas::<EvmWiringT, SPEC>),
            env: Arc::new(mainnet::validate_env::<EvmWiringT, SPEC>),
            tx_against_state: Arc::new(mainnet::validate_tx_against_state::<EvmWiringT, SPEC>),
        }
    }
}

impl<'a, EvmWiringT: EvmWiring> ValidationHandler<'a, EvmWiringT> {
    /// Validate env.
    pub fn env(&self, env: &EnvWiring<EvmWiringT>) -> EVMResultGeneric<(), EvmWiringT> {
        (self.env)(env)
    }

    /// Initial gas
    pub fn initial_tx_gas(&self, env: &EnvWiring<EvmWiringT>) -> EVMResultGeneric<u64, EvmWiringT> {
        (self.initial_tx_gas)(env)
    }

    /// Validate ttansaction against the state.
    pub fn tx_against_state(
        &self,
        context: &mut Context<EvmWiringT>,
    ) -> EVMResultGeneric<(), EvmWiringT> {
        (self.tx_against_state)(context)
    }
}
