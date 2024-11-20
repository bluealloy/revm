use primitives::{Address, Bytes};
use interpreter::InterpreterResult;

pub trait PrecompileProvider: Clone {
    type Context;
    type Error;

    fn new(ctx: &mut Self::Context) -> Self;

    fn run(
        &mut self,
        ctx: &mut Self::Context,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<InterpreterResult>, Self::Error>;

    fn warm_addresses(&self) -> impl Iterator<Item = Address>;
}
