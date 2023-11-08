use crate::{Database, Inspector};

/// Dummy [Inspector], helpful as standalone replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoOpInspector;

impl<EXT, DB: Database> Inspector<EXT, DB> for NoOpInspector {}
