use crate::{db::Database, handler::Handler, Evm};
use std::boxed::Box;

/// EVM Handler
pub type EvmHandler<'a, EXT, DB> = Handler<'a, Evm<'a, EXT, DB>, EXT, DB>;

// Handle register
pub type HandleRegister<'a, EXT, DB> = fn(&mut EvmHandler<'a, EXT, DB>);

// Boxed handle register
pub type HandleRegisterBox<'a, EXT, DB> = Box<dyn Fn(&mut EvmHandler<'a, EXT, DB>)>;

pub enum HandleRegisters<'a, EXT, DB: Database> {
    /// Plain function register
    Plain(HandleRegister<'a, EXT, DB>),
    /// Boxed function register.
    Box(HandleRegisterBox<'a, EXT, DB>),
}

impl<'a, EXT, DB: Database> HandleRegisters<'a, EXT, DB> {
    /// Call register function to modify EvmHandler.
    pub fn register(&self, handler: &mut EvmHandler<'a, EXT, DB>) {
        match self {
            HandleRegisters::Plain(f) => f(handler),
            HandleRegisters::Box(f) => f(handler),
        }
    }
}
