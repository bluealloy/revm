use crate::{primitives::ChainSpec, Database, Inspector};
/// Dummy [Inspector], helpful as standalone replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoOpInspector;

impl<ChainSpecT: ChainSpec, DB: Database> Inspector<ChainSpecT, DB> for NoOpInspector {}
