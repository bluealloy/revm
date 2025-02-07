use crate::setters::ContextSetters;
use core::ops::{Deref, DerefMut};

pub struct Evm<CTX, INSP, I, P> {
    pub data: EvmData<CTX, INSP>,
    pub enabled_inspection: bool,
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
            enabled_inspection: false,
            instruction: (),
            precompiles: (),
        }
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
