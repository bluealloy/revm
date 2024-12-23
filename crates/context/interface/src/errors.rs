use auto_impl::auto_impl;

// TODO : Change the name of the trait
#[auto_impl(&mut, Box)]
pub trait ErrorGetter {
    type Error;

    fn take_error(&mut self) -> Result<(), Self::Error>;
}
