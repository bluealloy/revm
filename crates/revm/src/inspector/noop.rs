//! Dummy NoOp Inspector, helpful as standalone replacment.

#[derive(Clone, Copy)]
pub struct NoOpInspector();

impl<DB: Database> Inspector<DB> for NoOpInspector {}
