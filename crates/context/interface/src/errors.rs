use auto_impl::auto_impl;
/// Trait to get error from context.
#[auto_impl(&mut, Box)]
pub trait ErrorGetter {
    type Error;

    fn take_error(&mut self) -> Result<(), Self::Error>;
}
