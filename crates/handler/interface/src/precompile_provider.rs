use interpreter::InterpreterResult;
use primitives::{Address, Bytes};

pub trait PrecompileProvider: Clone {
    type Context;
    type Error;

    fn new(context: &mut Self::Context) -> Self;

    fn run(
        &mut self,
        context: &mut Self::Context,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<InterpreterResult>, Self::Error>;

    fn warm_addresses(&self) -> impl Iterator<Item = Address>;
}
