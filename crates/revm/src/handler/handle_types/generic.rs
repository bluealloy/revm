use crate::{
    primitives::{db::Database, EVMResultGeneric},
    Context,
};
use std::sync::Arc;

/// Generic Handle that takes a mutable reference to the context and returns a result.
pub type GenericContextHandle<'a, EXT, DB> = GenericContextHandleRet<'a, EXT, DB, ()>;

/// Generic handle that takes a mutable reference to the context and returns a result.
pub type GenericContextHandleRet<'a, EXT, DB, ReturnT> =
    Arc<dyn Fn(&mut Context<EXT, DB>) -> EVMResultGeneric<ReturnT, <DB as Database>::Error> + 'a>;
