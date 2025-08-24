use crate::inspector::Inspector;
use interpreter::InterpreterTypes;

/// Dummy [Inspector], helpful as standalone replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NoOpInspector;

impl<CTX, INTR: InterpreterTypes> Inspector<CTX, INTR> for NoOpInspector {}
