use crate::FrameOrResultGen;

/// Makes sense
pub trait Frame: Sized {
    type Context;
    type FrameInit;
    type FrameResult;
    type Error;

    fn init_first(
        cxt: &mut Self::Context,
        frame_action: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error>;

    fn init(
        &self,
        cxt: &mut Self::Context,
        frame_action: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error>;

    fn run(
        &mut self,
        context: &mut Self::Context,
    ) -> Result<FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error>;

    fn return_result(
        &mut self,
        cxt: &mut Self::Context,
        result: Self::FrameResult,
    ) -> Result<(), Self::Error>;
}
