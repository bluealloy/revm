use crate::{FrameInitOrResult, FrameOrResult};

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
    ) -> Result<FrameOrResult<Self>, Self::Error>;

    fn init(
        &self,
        context: &mut Self::Context,
        frame_context: &mut Self::FrameContext,
        frame_input: Self::FrameInit,
    ) -> Result<FrameOrResult<Self>, Self::Error>;

    fn run(
        &mut self,
        context: &mut Self::Context,
        frame_context: &mut Self::FrameContext,
    ) -> Result<FrameInitOrResult<Self>, Self::Error>;

    fn return_result(
        &mut self,
        context: &mut Self::Context,
        frame_context: &mut Self::FrameContext,
        result: Self::FrameResult,
    ) -> Result<(), Self::Error>;
}
