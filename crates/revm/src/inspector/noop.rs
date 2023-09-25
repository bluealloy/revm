//! Dummy NoOp Inspector, helpful as standalone replacement.

use crate::{Database, Inspector};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoOpInspector;

impl<DB: Database> Inspector<DB> for NoOpInspector {}
