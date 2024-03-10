use crate::{
    handler::mainnet,
    primitives::{db::Database, EVMError, Env, Spec},
    Context,
};

/// Handle that validates env.
pub trait ValidateEnvTrait<DB: Database>: Send + Sync {
    fn validate_env(&self, env: &Env) -> Result<(), EVMError<DB::Error>>;
}

/// Handle that validates transaction environment against the state.
/// Second parametar is initial gas.
pub trait ValidateTxAgainstStateTrait<EXT, DB: Database>: Send + Sync {
    fn validate_tx_against_state(
        &self,
        context: &mut Context<EXT, DB>,
    ) -> Result<(), EVMError<DB::Error>>;
}

/// Initial gas calculation handle
pub trait ValidateInitialTxGasTrait<DB: Database>: Send + Sync {
    fn validate_initial_tx_gas(&self, env: &Env) -> Result<u64, EVMError<DB::Error>>;
}

/// Handles related to validation.
pub struct ValidationHandler<EXT, DB: Database> {
    /// Validate and calculate initial transaction gas.
    pub initial_tx_gas: Box<dyn ValidateInitialTxGasTrait<DB>>,
    /// Validate transactions against state data.
    pub tx_against_state: Box<dyn ValidateTxAgainstStateTrait<EXT, DB>>,
    /// Validate Env.
    pub env: Box<dyn ValidateEnvTrait<DB>>,
}

impl<EXT, DB: Database> ValidationHandler<EXT, DB> {
    /// Create new ValidationHandles
    pub fn new<SPEC: Spec>() -> Self {
        Self {
            initial_tx_gas: Box::new(mainnet::ValidateInitialTxGasImpl::<SPEC>::default()),
            //env: Arc::new(mainnet::validate_env::<SPEC, DB>),
            env: Box::new(mainnet::ValidateEnvImpl::<SPEC>::default()),
            tx_against_state: Box::new(mainnet::ValidateTxAgainstStateImpl::<SPEC>::default()),
        }
    }
}

impl<EXT, DB: Database> ValidationHandler<EXT, DB> {
    /// Validate env.
    pub fn env(&self, env: &Env) -> Result<(), EVMError<DB::Error>> {
        self.env.validate_env(env)
    }

    /// Initial gas
    pub fn initial_tx_gas(&self, env: &Env) -> Result<u64, EVMError<DB::Error>> {
        self.initial_tx_gas.validate_initial_tx_gas(env)
    }

    /// Validate ttansaction against the state.
    pub fn tx_against_state(
        &self,
        context: &mut Context<EXT, DB>,
    ) -> Result<(), EVMError<DB::Error>> {
        self.tx_against_state.validate_tx_against_state(context)
    }
}
