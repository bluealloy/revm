use core::fmt::Debug;
use core::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Evm<CTX, INSP, I, P> {
    pub data: EvmData<CTX, INSP>,
    pub instruction: I,
    pub precompiles: P,
}

#[derive(Debug)]
pub struct EvmData<CTX, INSP> {
    pub ctx: CTX,
    pub inspector: INSP,
}

impl<CTX, I, P> Evm<CTX, (), I, P> {
    pub fn new(ctx: CTX, instruction: I, precompiles: P) -> Evm<CTX, (), I, P> {
        Evm {
            data: EvmData { ctx, inspector: () },
            instruction,
            precompiles,
        }
    }
}

impl<CTX, I, INSP, P> Evm<CTX, INSP, I, P> {
    pub fn new_with_inspector(ctx: CTX, inspector: INSP, instruction: I, precompiles: P) -> Self {
        Evm {
            data: EvmData { ctx, inspector },
            instruction,
            precompiles,
        }
    }
}

impl<CTX, INSP, I, P> Evm<CTX, INSP, I, P> {
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
