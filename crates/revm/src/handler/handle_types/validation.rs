use crate::{
    handler::mainnet,
    primitives::{
        db::Database, EVMResultGeneric, Env, InvalidTransaction, Spec, TransactionValidation,
    },
    Context, EvmWiring,
};
use std::sync::Arc;

/// Handle that validates env.
pub type ValidateEnvHandle<'a, EvmWiringT, DB> =
    Arc<dyn Fn(&Env<EvmWiringT>) -> EVMResultGeneric<(), EvmWiringT, <DB as Database>::Error> + 'a>;

/// Handle that validates transaction environment against the state.
/// Second parametar is initial gas.
pub type ValidateTxEnvAgainstState<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
        ) -> EVMResultGeneric<(), EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Initial gas calculation handle
pub type ValidateInitialTxGasHandle<'a, EvmWiringT, DB> = Arc<
    dyn Fn(&Env<EvmWiringT>) -> EVMResultGeneric<u64, EvmWiringT, <DB as Database>::Error> + 'a,
>;

/// Handles related to validation.
pub struct ValidationHandler<'a, EvmWiringT: EvmWiring, EXT, DB: Database> {
    /// Validate and calculate initial transaction gas.
    pub initial_tx_gas: ValidateInitialTxGasHandle<'a, EvmWiringT, DB>,
    /// Validate transactions against state data.
    pub tx_against_state: ValidateTxEnvAgainstState<'a, EvmWiringT, EXT, DB>,
    /// Validate Env.
    pub env: ValidateEnvHandle<'a, EvmWiringT, DB>,
}

impl<'a, EvmWiringT: EvmWiring, EXT: 'a, DB: Database + 'a>
    ValidationHandler<'a, EvmWiringT, EXT, DB>
where
    <EvmWiringT::Transaction as TransactionValidation>::ValidationError: From<InvalidTransaction>,
{
    /// Create new ValidationHandles
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            initial_tx_gas: Arc::new(mainnet::validate_initial_tx_gas::<EvmWiringT, SPEC, DB>),
            env: Arc::new(mainnet::validate_env::<EvmWiringT, SPEC, DB>),
            tx_against_state: Arc::new(
                mainnet::validate_tx_against_state::<EvmWiringT, SPEC, EXT, DB>,
            ),
        }
    }
}

impl<'a, EvmWiringT: EvmWiring, EXT, DB: Database> ValidationHandler<'a, EvmWiringT, EXT, DB> {
    /// Validate env.
    pub fn env(&self, env: &Env<EvmWiringT>) -> EVMResultGeneric<(), EvmWiringT, DB::Error> {
        (self.env)(env)
    }

    /// Initial gas
    pub fn initial_tx_gas(
        &self,
        env: &Env<EvmWiringT>,
    ) -> EVMResultGeneric<u64, EvmWiringT, DB::Error> {
        (self.initial_tx_gas)(env)
    }

    /// Validate ttansaction against the state.
    pub fn tx_against_state(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
    ) -> EVMResultGeneric<(), EvmWiringT, DB::Error> {
        (self.tx_against_state)(context)
    }
}
