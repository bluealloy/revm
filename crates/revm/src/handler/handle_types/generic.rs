use crate::{
    primitives::{db::Database, EVMResultGeneric, EvmWiring},
    Context,
};
use std::sync::Arc;

/// Generic Handle that takes a mutable reference to the context and returns a result.
pub type GenericContextHandle<'a, EvmWiring> = GenericContextHandleRet<'a, EvmWiring, ()>;

/// Generic handle that takes a mutable reference to the context and returns a result.
pub type GenericContextHandleRet<'a, EvmWiringT, ReturnT> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT>,
        )
            -> EVMResultGeneric<ReturnT, <<EvmWiringT as EvmWiring>::Database as Database>::Error>
        + 'a,
>;
