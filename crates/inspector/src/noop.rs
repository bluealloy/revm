use revm::interpreter::InterpreterTypes;

use crate::Inspector;

/// Dummy [Inspector], helpful as standalone replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoOpInspector<CTX, INTR> {
    _phantom: core::marker::PhantomData<(CTX, INTR)>,
}

impl<CTX, INTR: InterpreterTypes> Inspector for NoOpInspector<CTX, INTR> {
    type Context = CTX;
    type InterpreterTypes = INTR;
}
