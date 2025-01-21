use auto_impl::auto_impl;
use primitives::{Address, Bytes};
use specification::hardfork::SpecId;
use std::boxed::Box;

#[auto_impl(&mut, Box)]
pub trait PrecompileProvider: Clone {
    type Context;
    type Output;
    type Error;
    type Spec: Into<SpecId>;

    fn set_spec(&mut self, spec: Self::Spec);

    /// Run the precompile.
    fn run(
        &mut self,
        context: &mut Self::Context,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, Self::Error>;

    /// Get the warm addresses.
    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address> + '_>;

    /// Check if the address is a precompile.
    fn contains(&self, address: &Address) -> bool;
}

pub trait PrecompileProviderGetter {
    type PrecompileProvider: PrecompileProvider;

    fn precompiles(&mut self) -> &mut Self::PrecompileProvider;
}
