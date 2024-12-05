pub trait ValidationHandler {
    type Context;
    type Error;

    /// Validate env.
    fn validate_env(&self, context: &Self::Context) -> Result<(), Self::Error>;

    /// Validate transactions against state.
    fn validate_tx_against_state(&self, context: &mut Self::Context) -> Result<(), Self::Error>;

    /// Validate initial gas.
    fn validate_initial_tx_gas(&self, context: &Self::Context) -> Result<u64, Self::Error>;
}
