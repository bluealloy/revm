use crate::{
    primitives::{db::Database, EVMResultGeneric},
    Context,
};
use std::sync::Arc;

/// Generic Handle that takes a mutable reference to the context and returns a result.
pub type GenericContextHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>) -> EVMResultGeneric<(), <DB as Database>::Error> + 'a>;
