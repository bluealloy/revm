use crate::inspector::Inspector;
use interpreter::InterpreterTypes;

/// Dummy [Inspector], helpful as standalone replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoOpInspector;

impl<CTX, INTR: InterpreterTypes> Inspector<CTX, INTR> for NoOpInspector {}
