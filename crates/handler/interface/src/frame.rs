use crate::FrameOrResult;

/// Call frame trait
pub trait Frame: Sized {
    type Context;
    type FrameInit;
    type FrameContext;
    type FrameResult;
    type Error;

    fn init_first(
        context: &mut Self::Context,
        frame_context: &mut Self::FrameContext,
        frame_input: Self::FrameInit,
    ) -> Result<FrameOrResult<Self, Self::FrameResult>, Self::Error>;

    fn final_return(
        context: &mut Self::Context,
        frame_context: &mut Self::FrameContext,
        result: &mut Self::FrameResult,
    ) -> Result<(), Self::Error>;

    fn init(
        &self,
        context: &mut Self::Context,
        frame_context: &mut Self::FrameContext,
        frame_input: Self::FrameInit,
    ) -> Result<FrameOrResult<Self, Self::FrameResult>, Self::Error>;

    fn run(
        &mut self,
        context: &mut Self::Context,
        frame_context: &mut Self::FrameContext,
    ) -> Result<FrameOrResult<Self::FrameInit, Self::FrameResult>, Self::Error>;

    fn return_result(
        &mut self,
        context: &mut Self::Context,
        frame_context: &mut Self::FrameContext,
        result: Self::FrameResult,
    ) -> Result<(), Self::Error>;
}
