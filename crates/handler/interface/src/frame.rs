use crate::FrameOrResultGen;

/// Call frame trait
pub trait Frame: Sized {
    type Context;
    type SharedContext;
    type FrameInit;
    type FrameResult;
    type Error;

    fn init_first(
        context: &mut Self::Context,
        frame_input: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error>;

    fn final_return(
        context: &mut Self::Context,
        result: &mut Self::FrameResult,
    ) -> Result<(), Self::Error>;

    fn init(
        &self,
        context: &mut Self::Context,
        frame_input: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error>;

    fn run(
        &mut self,
        context: &mut Self::Context,
    ) -> Result<FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error>;

    fn return_result(
        &mut self,
        context: &mut Self::Context,
        result: Self::FrameResult,
    ) -> Result<(), Self::Error>;
}
