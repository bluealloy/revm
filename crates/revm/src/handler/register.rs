use crate::{db::Database, handler::Handler, ChainSpec, Context};
use std::boxed::Box;

/// EVM Handler
pub type EvmHandler<'a, ChainSpecT, EXT, DB> =
    Handler<'a, ChainSpecT, Context<ChainSpecT, EXT, DB>, EXT, DB>;

// Handle register
pub type HandleRegister<ChainSpecT, EXT, DB> = for<'a> fn(&mut EvmHandler<'a, ChainSpecT, EXT, DB>);

// Boxed handle register
pub type HandleRegisterBox<'a, ChainSpecT, EXT, DB> =
    Box<dyn for<'e> Fn(&mut EvmHandler<'e, ChainSpecT, EXT, DB>) + 'a>;

pub enum HandleRegisters<'a, ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Plain function register
    Plain(HandleRegister<ChainSpecT, EXT, DB>),
    /// Boxed function register.
    Box(HandleRegisterBox<'a, ChainSpecT, EXT, DB>),
}

impl<'register, ChainSpecT: ChainSpec, EXT, DB: Database>
    HandleRegisters<'register, ChainSpecT, EXT, DB>
{
    /// Call register function to modify EvmHandler.
    pub fn register<'evm>(&self, handler: &mut EvmHandler<'evm, ChainSpecT, EXT, DB>)
    where
        'evm: 'register,
    {
        match self {
            HandleRegisters::Plain(f) => f(handler),
            HandleRegisters::Box(f) => f(handler),
        }
    }
}
