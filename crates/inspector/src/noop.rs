use core::marker::PhantomData;

use crate::inspector::Inspector;
use derive_where::derive_where;
use interpreter::InterpreterTypes;

/// Dummy [Inspector], helpful as standalone replacement.
#[derive_where(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoOpInspector<CTX> {
    phantom: PhantomData<CTX>,
}

impl<CTX, INTR: InterpreterTypes> Inspector<INTR> for NoOpInspector<CTX> {
    type Context<'context> = CTX;
}
