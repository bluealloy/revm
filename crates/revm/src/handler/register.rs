use crate::{db::Database, handler::Handler, Evm};
use std::boxed::Box;

/// EVM Handler
pub type EvmHandler<EXT, DB> = Handler<Evm<EXT, DB>, EXT, DB>;

pub type HandleRegisterFn<EXT, DB> = fn(&mut EvmHandler<EXT, DB>);

pub trait HandleRegisterTrait<EXT, DB: Database> {
    fn register(&self, handler: &mut EvmHandler<EXT, DB>);
}

// Boxed handle register
pub type HandleRegisterBox<EXT, DB> = Box<dyn HandleRegisterTrait<EXT, DB>>;
