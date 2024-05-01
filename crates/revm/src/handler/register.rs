use crate::{db::Database, handler::Handler, primitives::ChainSpec, Context};
use std::boxed::Box;

/// EVM Handler
pub type EvmHandler<'a, ChainSpecT, EXT, DB> =
    Handler<'a, ChainSpecT, Context<ChainSpecT, EXT, DB>, EXT, DB>;

// Handle register
pub type HandleRegister<ChainSpecT, EXT, DB> = for<'a> fn(&mut EvmHandler<'a, ChainSpecT, EXT, DB>);

// Boxed handle register
pub type HandleRegisterBox<ChainSpecT, EXT, DB> =
    Box<dyn for<'a> Fn(&mut EvmHandler<'a, ChainSpecT, EXT, DB>)>;

pub enum HandleRegisters<ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Plain function register
    Plain(HandleRegister<ChainSpecT, EXT, DB>),
    /// Boxed function register.
    Box(HandleRegisterBox<ChainSpecT, EXT, DB>),
}

impl<ChainSpecT: ChainSpec, EXT, DB: Database> HandleRegisters<ChainSpecT, EXT, DB> {
    /// Call register function to modify EvmHandler.
    pub fn register(&self, handler: &mut EvmHandler<'_, ChainSpecT, EXT, DB>) {
        match self {
            HandleRegisters::Plain(f) => f(handler),
            HandleRegisters::Box(f) => f(handler),
        }
    }
}
