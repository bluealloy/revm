/// Returns of `Frame` trait methods
pub enum FrameOrResult<Frame, Result> {
    Frame(Frame),
    Result(Result),
}

/// Execution frame.
pub trait Frame: Sized {
    type FrameAction: Sized;
    type FrameResult: Sized;
    type Context: Sized;

    fn init(frame_action: Self::FrameAction, cxt: &mut Self::Context) -> FrameOrResult<Self, Self::FrameResult>;

    fn run(
        &mut self,
        instructions: (),
        cnt: &mut Self::Context,
    ) -> FrameOrResult<Self::FrameAction, Self::FrameResult>;

    fn return_result(&mut self, result: Self::FrameResult);
}