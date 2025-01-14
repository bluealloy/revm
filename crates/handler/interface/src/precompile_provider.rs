use primitives::{Address, Bytes};

pub trait PrecompileProvider: Clone {
    type Context;
    type Output;
    type Error;

    /// Create a new precompile.
    fn new(context: &mut Self::Context) -> Self;

    /// Run the precompile.
    fn run(
        &mut self,
        context: &mut Self::Context,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, Self::Error>;

    /// Get the warm addresses.
    fn warm_addresses(&self) -> impl Iterator<Item = Address>;

    /// Check if the address is a precompile.
    fn contains(&self, address: &Address) -> bool;
}

pub trait PrecompileProviderGetter {
    type PrecompileProvider: PrecompileProvider;

    fn precompiles(&mut self) -> &mut Self::PrecompileProvider;
}
