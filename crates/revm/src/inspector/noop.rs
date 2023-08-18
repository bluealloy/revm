//! Dummy NoOp Inspector, helpful as standalone replacement.

use crate::{Database, Inspector};

#[derive(Clone, Copy)]
pub struct NoOpInspector();

impl<DB: Database> Inspector<DB> for NoOpInspector {}
