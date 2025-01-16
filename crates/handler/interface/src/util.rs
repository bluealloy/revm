use crate::Frame;

pub enum FrameOrResult<Frame, Result> {
    Frame(Frame),
    Result(Result),
}

impl<F, R> FrameOrResult<F, R> {
    pub fn map_frame<F2>(self, f: impl FnOnce(F) -> F2) -> FrameOrResult<F2, R> {
        match self {
            FrameOrResult::Frame(frame) => FrameOrResult::Frame(f(frame)),
            FrameOrResult::Result(result) => FrameOrResult::Result(result),
        }
    }

    pub fn map_result<R2>(self, f: impl FnOnce(R) -> R2) -> FrameOrResult<F, R2> {
        match self {
            FrameOrResult::Frame(frame) => FrameOrResult::Frame(frame),
            FrameOrResult::Result(result) => FrameOrResult::Result(f(result)),
        }
    }
}

pub type FrameOrFrameResult<FRAME> = FrameOrResult<FRAME, <FRAME as Frame>::FrameResult>;
