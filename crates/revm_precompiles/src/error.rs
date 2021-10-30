use alloc::borrow::Cow;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExitError {
    Exit,
    OutOfGas,
    /// Other normal errors.
    Other(Cow<'static, str>),
}
