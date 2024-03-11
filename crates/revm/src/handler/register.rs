use crate::{db::Database, handler::Handler, Evm};
use std::boxed::Box;

/// EVM Handler
pub type EvmHandler<EXT, DB> = Handler<Evm<EXT, DB>, EXT, DB>;

// Handle register
pub type HandleRegister<EXT, DB> = fn(&mut EvmHandler<EXT, DB>);

// Boxed handle register
pub type HandleRegisterBox<EXT, DB> = Box<dyn Fn(&mut EvmHandler<EXT, DB>)>;

pub enum HandleRegisters<EXT, DB: Database> {
    /// Plain function register
    Plain(HandleRegister<EXT, DB>),
    /// Boxed function register.
    Box(HandleRegisterBox<EXT, DB>),
}

impl<EXT, DB: Database> HandleRegisters<EXT, DB> {
    /// Call register function to modify EvmHandler.
    pub fn register(&self, handler: &mut EvmHandler<EXT, DB>) {
        match self {
            HandleRegisters::Plain(f) => f(handler),
            HandleRegisters::Box(f) => f(handler),
        }
    }
}
