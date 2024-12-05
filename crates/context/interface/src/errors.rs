/// TODO change name of the trait
pub trait ErrorGetter {
    type Error;

    fn take_error(&mut self) -> Result<(), Self::Error>;
}
