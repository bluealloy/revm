use crate::setters::ContextSetters;
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};

pub struct Evm<CTX, INSP, I, P> {
    pub data: EvmData<CTX, INSP>,
    pub instruction: I,
    pub precompiles: P,
}

pub struct EvmData<CTX, INSP> {
    pub ctx: CTX,
    pub inspector: INSP,
}

impl<CTX> Evm<CTX, (), (), ()> {
    pub fn new(ctx: CTX) -> Self {
        Evm {
            data: EvmData { ctx, inspector: () },
            instruction: (),
            precompiles: (),
        }
    }
}

impl<CTX: Debug, INSP: Debug, I: Debug, P: Debug> Debug for Evm<CTX, INSP, I, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Evm")
            .field("data", &self.data)
            .field("instruction", &self.instruction)
            .field("precompiles", &self.precompiles)
            .finish()
    }
}

impl<CTX: Debug, INSP: Debug> Debug for EvmData<CTX, INSP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EvmData")
            .field("ctx", &self.ctx)
            .field("inspector", &self.inspector)
            .finish()
    }
}

impl<CTX: ContextSetters, INSP, I, P> Evm<CTX, INSP, I, P> {
    /// Consumed self and returns new Evm type with given Inspector.
    pub fn with_inspector<OINSP>(self, inspector: OINSP) -> Evm<CTX, OINSP, I, P> {
        Evm {
            data: EvmData {
                ctx: self.data.ctx,
                inspector,
            },
            instruction: self.instruction,
            precompiles: self.precompiles,
        }
    }

    /// Consumes self and returns new Evm type with given Precompiles.
    pub fn with_precompiles<OP>(self, precompiles: OP) -> Evm<CTX, INSP, I, OP> {
        Evm {
            data: self.data,
            instruction: self.instruction,
            precompiles,
        }
    }

    /// Consumes self and returns inner Inspector.
    pub fn into_inspector(self) -> INSP {
        self.data.inspector
    }
}

impl<CTX: ContextSetters, INSP, I, P> ContextSetters for Evm<CTX, INSP, I, P> {
    type Tx = <CTX as ContextSetters>::Tx;
    type Block = <CTX as ContextSetters>::Block;

    fn set_tx(&mut self, tx: Self::Tx) {
        self.data.ctx.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.data.ctx.set_block(block);
    }
}

impl<CTX, INSP, I, P> Deref for Evm<CTX, INSP, I, P> {
    type Target = CTX;

    fn deref(&self) -> &Self::Target {
        &self.data.ctx
    }
}

impl<CTX, INSP, I, P> DerefMut for Evm<CTX, INSP, I, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data.ctx
    }
}
