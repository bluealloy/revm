use crate::{db::Database, handler::Handler, Context};
use std::boxed::Box;

/// EVM Handler
pub type EvmHandler<'a, EXT, DB> = Handler<'a, Context<EXT, DB>, EXT, DB>;

// Handle register
pub type HandleRegister<EXT, DB> = for<'a> fn(&mut EvmHandler<'a, EXT, DB>);

// Boxed handle register
pub type HandleRegisterBox<'a, EXT, DB> = Box<dyn for<'e> Fn(&mut EvmHandler<'e, EXT, DB>) + 'a>;

pub enum HandleRegisters<'a, EXT, DB: Database> {
    /// Plain function register
    Plain(HandleRegister<EXT, DB>),
    /// Boxed function register.
    Box(HandleRegisterBox<'a, EXT, DB>),
}

impl<'register, EXT, DB: Database> HandleRegisters<'register, EXT, DB> {
    /// Call register function to modify EvmHandler.
    pub fn register<'evm>(&self, handler: &mut EvmHandler<'evm, EXT, DB>)
    where
        'evm: 'register,
    {
        match self {
            HandleRegisters::Plain(f) => f(handler),
            HandleRegisters::Box(f) => f(handler),
        }
    }
}
