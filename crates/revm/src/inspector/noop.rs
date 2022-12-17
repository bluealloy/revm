//! Dummy NoOp Inspector, helpful as standalone replacemnt.

use crate::{blockchain::Blockchain, Database, Inspector};

#[derive(Clone, Copy)]
pub struct NoOpInspector();

impl<DB: Database, BC: Blockchain<Error = DB::Error>> Inspector<DB, BC> for NoOpInspector {}
