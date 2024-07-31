use crate::{db::Database, handler::Handler, Context, EvmWiring};
use std::boxed::Box;

/// EVM Handler
pub type EvmHandler<'a, EvmWiringT, EXT, DB> =
    Handler<'a, EvmWiringT, Context<EvmWiringT, EXT, DB>, EXT, DB>;

// Handle register
pub type HandleRegister<EvmWiringT, EXT, DB> = for<'a> fn(&mut EvmHandler<'a, EvmWiringT, EXT, DB>);

// Boxed handle register
pub type HandleRegisterBox<'a, EvmWiringT, EXT, DB> =
    Box<dyn for<'e> Fn(&mut EvmHandler<'e, EvmWiringT, EXT, DB>) + 'a>;

pub enum HandleRegisters<'a, EvmWiringT: EvmWiring, EXT, DB: Database> {
    /// Plain function register
    Plain(HandleRegister<EvmWiringT, EXT, DB>),
    /// Boxed function register.
    Box(HandleRegisterBox<'a, EvmWiringT, EXT, DB>),
}

impl<'register, EvmWiringT: EvmWiring, EXT, DB: Database>
    HandleRegisters<'register, EvmWiringT, EXT, DB>
{
    /// Call register function to modify EvmHandler.
    pub fn register<'evm>(&self, handler: &mut EvmHandler<'evm, EvmWiringT, EXT, DB>)
    where
        'evm: 'register,
    {
        match self {
            HandleRegisters::Plain(f) => f(handler),
            HandleRegisters::Box(f) => f(handler),
        }
    }
}
