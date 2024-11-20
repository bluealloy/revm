pub enum FrameOrResultGen<Frame, Result> {
    Frame(Frame),
    Result(Result),
}

impl<F, R> FrameOrResultGen<F, R> {
    pub fn map_frame<F2>(self, f: impl FnOnce(F) -> F2) -> FrameOrResultGen<F2, R> {
        match self {
            FrameOrResultGen::Frame(frame) => FrameOrResultGen::Frame(f(frame)),
            FrameOrResultGen::Result(result) => FrameOrResultGen::Result(result),
        }
    }

    pub fn map_result<R2>(self, f: impl FnOnce(R) -> R2) -> FrameOrResultGen<F, R2> {
        match self {
            FrameOrResultGen::Frame(frame) => FrameOrResultGen::Frame(frame),
            FrameOrResultGen::Result(result) => FrameOrResultGen::Result(f(result)),
        }
    }
}
