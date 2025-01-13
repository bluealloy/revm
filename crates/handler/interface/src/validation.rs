pub trait ValidationHandler {
    type Context;
    type Error;

    /// Validate env.
    fn validate_env(&self, context: &Self::Context) -> Result<(), Self::Error>;

    /// Validate transactions against state.
    fn validate_tx_against_state(&self, context: &mut Self::Context) -> Result<(), Self::Error>;

    /// Validate initial gas.
    fn validate_initial_tx_gas(
        &self,
        context: &Self::Context,
    ) -> Result<InitialAndFloorGas, Self::Error>;
}

/// Init and floor gas from transaction
#[derive(Clone, Copy, Debug, Default)]
pub struct InitialAndFloorGas {
    /// Initial gas for transaction.
    pub initial_gas: u64,
    /// If transaction is a Call and Prague is enabled
    /// floor_gas is at least amount of gas that is going to be spent.
    pub floor_gas: u64,
}
