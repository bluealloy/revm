use crate::{handler::Handler, Context, EvmWiring};
use std::boxed::Box;

/// EVM Handler
pub type EvmHandler<'a, EvmWiringT> = Handler<'a, EvmWiringT, Context<EvmWiringT>>;

// Handle register
pub type HandleRegister<EvmWiringT> = for<'a> fn(&mut EvmHandler<'a, EvmWiringT>);

// Boxed handle register
pub type HandleRegisterBox<'a, EvmWiringT> =
    Box<dyn for<'e> Fn(&mut EvmHandler<'e, EvmWiringT>) + 'a>;

pub enum HandleRegisters<'a, EvmWiringT: EvmWiring> {
    /// Plain function register
    Plain(HandleRegister<EvmWiringT>),
    /// Boxed function register.
    Box(HandleRegisterBox<'a, EvmWiringT>),
}

impl<'register, EvmWiringT: EvmWiring> HandleRegisters<'register, EvmWiringT> {
    /// Call register function to modify EvmHandler.
    pub fn register<'evm>(&self, handler: &mut EvmHandler<'evm, EvmWiringT>)
    where
        'evm: 'register,
    {
        match self {
            HandleRegisters::Plain(f) => f(handler),
            HandleRegisters::Box(f) => f(handler),
        }
    }
}
