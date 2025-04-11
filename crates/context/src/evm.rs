//! This module contains [`Evm`] struct.
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};

/// Main EVM structure that contains all data needed for execution.
#[derive(Debug, Clone)]
pub struct Evm<CTX, INSP, I, P> {
    /// [`context_interface::ContextTr`] of the EVM it is used to fetch data from database.
    pub ctx: CTX,
    /// Inspector of the EVM it is used to inspect the EVM.
    /// Its trait are defined in revm-inspector crate.
    pub inspector: INSP,
    /// Instructions provider of the EVM it is used to execute instructions.
    /// `InstructionProvider` trait is defined in revm-handler crate.
    pub instruction: I,
    /// Precompile provider of the EVM it is used to execute precompiles.
    /// `PrecompileProvider` trait is defined in revm-handler crate.
    pub precompiles: P,
}

impl<CTX, I, P> Evm<CTX, (), I, P> {
    /// Create a new EVM instance with a given context, instruction set, and precompile provider.
    ///
    /// Inspector will be set to `()`.
    pub fn new(ctx: CTX, instruction: I, precompiles: P) -> Self {
        Evm {
            ctx,
            inspector: (),
            instruction,
            precompiles,
        }
    }
}

impl<CTX, I, INSP, P> Evm<CTX, INSP, I, P> {
    /// Create a new EVM instance with a given context, inspector, instruction set, and precompile provider.
    pub fn new_with_inspector(ctx: CTX, inspector: INSP, instruction: I, precompiles: P) -> Self {
        Evm {
            ctx,
            inspector,
            instruction,
            precompiles,
        }
    }
}

impl<CTX, INSP, I, P> Evm<CTX, INSP, I, P> {
    /// Consumed self and returns new Evm type with given Inspector.
    pub fn with_inspector<OINSP>(self, inspector: OINSP) -> Evm<CTX, OINSP, I, P> {
        Evm {
            ctx: self.ctx,
            inspector,

            instruction: self.instruction,
            precompiles: self.precompiles,
        }
    }

    /// Consumes self and returns new Evm type with given Precompiles.
    pub fn with_precompiles<OP>(self, precompiles: OP) -> Evm<CTX, INSP, I, OP> {
        Evm {
            ctx: self.ctx,
            inspector: self.inspector,
            instruction: self.instruction,
            precompiles,
        }
    }

    /// Consumes self and returns inner Inspector.
    pub fn into_inspector(self) -> INSP {
        self.inspector
    }
}

impl<CTX, INSP, I, P> Deref for Evm<CTX, INSP, I, P> {
    type Target = CTX;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<CTX, INSP, I, P> DerefMut for Evm<CTX, INSP, I, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}
