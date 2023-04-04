//! Dummy NoOp Inspector, helpful as standalone replacement.

use crate::Inspector;

#[derive(Clone, Copy)]
pub struct NoOpInspector();

impl<E> Inspector<E> for NoOpInspector {}
