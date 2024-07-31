use crate::{Database, EvmWiring, Inspector};

/// Dummy [Inspector], helpful as standalone replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoOpInspector;

impl<EvmWiringT: EvmWiring, DB: Database> Inspector<EvmWiringT, DB> for NoOpInspector {}
