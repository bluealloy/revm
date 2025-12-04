//! This module contains [`Evm`] struct.
use core::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use context_interface::FrameStack;

/// Main EVM structure that contains all data needed for execution.
#[derive(Debug, Clone)]
pub struct Evm<CTX, INSP, I, P, F> {
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
    /// Frame that is going to be executed.
    pub frame_stack: FrameStack<F>,
}

impl<CTX, I, P, F: Default> Evm<CTX, (), I, P, F> {
    /// Create a new EVM instance with a given context, instruction set, and precompile provider.
    ///
    /// Inspector will be set to `()`.
    pub fn new(ctx: CTX, instruction: I, precompiles: P) -> Self {
        Evm {
            ctx,
            inspector: (),
            instruction,
            precompiles,
            frame_stack: FrameStack::new_prealloc(8),
        }
    }
}

impl<CTX, I, INSP, P, F: Default> Evm<CTX, INSP, I, P, F> {
    /// Create a new EVM instance with a given context, inspector, instruction set, and precompile provider.
    pub fn new_with_inspector(ctx: CTX, inspector: INSP, instruction: I, precompiles: P) -> Self {
        Evm {
            ctx,
            inspector,
            instruction,
            precompiles,
            frame_stack: FrameStack::new_prealloc(8),
        }
    }
}

impl<CTX, INSP, I, P, F> Evm<CTX, INSP, I, P, F> {
    /// Consumed self and returns new Evm type with given Inspector.
    pub fn with_inspector<OINSP>(self, inspector: OINSP) -> Evm<CTX, OINSP, I, P, F> {
        Evm {
            ctx: self.ctx,
            inspector,

            instruction: self.instruction,
            precompiles: self.precompiles,
            frame_stack: self.frame_stack,
        }
    }

    /// Consumes self and returns new Evm type with given Precompiles.
    pub fn with_precompiles<OP>(self, precompiles: OP) -> Evm<CTX, INSP, I, OP, F> {
        Evm {
            ctx: self.ctx,
            inspector: self.inspector,
            instruction: self.instruction,
            precompiles,
            frame_stack: self.frame_stack,
        }
    }

    /// Consumes self and returns inner Inspector.
    pub fn into_inspector(self) -> INSP {
        self.inspector
    }
}

impl<CTX, INSP, I, P, F> Deref for Evm<CTX, INSP, I, P, F> {
    type Target = CTX;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<CTX, INSP, I, P, F> DerefMut for Evm<CTX, INSP, I, P, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}
