use crate::handler::wires::Frame;
use context::{BlockGetter, JournalStateGetter, TransactionGetter};
use interpreter::{Host, InternalResult, NewFrameAction};

pub struct EthFrame<CTX> {
    _phantom: std::marker::PhantomData<CTX>,
}

pub trait HostTemp: TransactionGetter + BlockGetter + JournalStateGetter {}

impl<CTX: HostTemp> Frame for EthFrame<CTX> {
    type Context = CTX;

    type FrameInit = NewFrameAction;

    type FrameResult = InternalResult;

    fn init(
        &self,
        frame_action: Self::FrameInit,
        cxt: &mut Self::Context,
    ) -> crate::handler::FrameOrResult<Self, Self::FrameResult> {
        todo!()
    }

    fn run(
        &mut self,
        instructions: (),
        context: &mut Self::Context,
    ) -> crate::handler::FrameOrResult<Self::FrameInit, Self::FrameResult> {
        todo!()
    }

    fn return_result(&mut self, result: Self::FrameResult) {
        todo!()
    }
}
